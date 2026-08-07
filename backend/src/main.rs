mod auth;
mod lunch;
use crate::header::HeaderValue;
use auth::login;
use axum::middleware;
use axum::middleware::Next;
use axum::{
    Router,
    extract::Request,
    http::{Method, header, status::StatusCode},
    response::IntoResponse,
    routing::{get, post},
};
use axum_extra::extract::cookie::SameSite;
use axum_governor::{GovernorConfigBuilder, GovernorLayer, Quota, extractor::PeerIp, nz};
use lunch::lunch_management;
use std::net::SocketAddr;
use tower_http::cors::CorsLayer;
use tower_sessions::{Expiry, Session, SessionManagerLayer};
use tower_sessions_sqlx_store_chrono::PostgresStore;
use uuid::Uuid;

#[tokio::main]
async fn main() {
    dotenvy::dotenv().ok(); // Load .env first
    let addr: &str = "0.0.0.0:3000";
    let listener = match tokio::net::TcpListener::bind(addr).await {
        Ok(t) => t,
        Err(e) => {
            eprintln!("Failed to bind to {}: {}", addr, e);
            std::process::exit(1);
        }
    };
    println!("Server running on {}", addr);

    let strict_cfg = GovernorConfigBuilder::default()
        .with_extractor(PeerIp::default())
        .expect_connect_info()
        .quota_default(Quota::requests_per_minute(nz!(5u32)))
        .finish()
        .unwrap();

    let normal_cfg = GovernorConfigBuilder::default()
        .with_extractor(PeerIp::default())
        .expect_connect_info()
        .quota_default(Quota::requests_per_second(nz!(50u32)))
        .finish()
        .unwrap();

    let database_connection = match std::env::var("DATABASE_URL") {
        Ok(t) => t,
        Err(_) => {
            eprintln!("Couldn't find db info in .env file!");
            std::process::exit(1);
        }
    };
    let pool = match sqlx::postgres::PgPoolOptions::new()
        .max_connections(10)
        .acquire_timeout(std::time::Duration::from_secs(3))
        .idle_timeout(std::time::Duration::from_secs(600))
        .connect(&database_connection)
        .await
    {
        Ok(t) => t,
        Err(e) => {
            eprintln!("Could not connect to database! {}", e);
            std::process::exit(1);
        }
    };

    match sqlx::migrate!("./migrations").run(&pool).await {
        Ok(t) => t,
        Err(e) => {
            eprintln!("Failed to run migrations: {}", e);
            std::process::exit(1);
        }
    };

    let session_store: PostgresStore = PostgresStore::new(pool.clone());
    match session_store.migrate().await {
        Ok(t) => t,
        Err(e) => {
            eprintln!("Failed to run session store: {}", e);
            std::process::exit(1);
        }
    };

    let session_layer = SessionManagerLayer::new(session_store)
        .with_path("/".to_string())
        .with_http_only(true)
        .with_same_site(SameSite::Lax)
        .with_secure(false) // true in production
        .with_expiry(Expiry::OnInactivity(
            tower_sessions::cookie::time::Duration::days(14),
        )); // 14 days - maybe change for production

    // CORS policy
    let cors = CorsLayer::new()
        .allow_origin("http://127.0.0.1:5500".parse::<HeaderValue>().unwrap()) // For production replace with domain name
        .allow_methods([Method::GET, Method::POST, Method::OPTIONS])
        .allow_headers([header::CONTENT_TYPE, header::AUTHORIZATION])
        .allow_credentials(true);

    let protected_routes = Router::new()
        .route("/api/auth_status", get(login::auth_status))
        .route("/api/lunch_data", get(lunch_management::lunch_data))
        .route(
            "/api/update_lunch_data",
            post(lunch_management::lunch_handling),
        )
        .route("/api/logout", post(login::logout))
        .route("/api/change_password", post(login::change_password))
        .route_layer(middleware::from_fn(auth))
        .layer(GovernorLayer::new(normal_cfg));

    let app = Router::new()
        .route("/api/login", post(login::login))
        .layer(GovernorLayer::new(strict_cfg))
        .merge(protected_routes)
        .with_state(pool)
        .layer(session_layer)
        .layer(cors);

    match axum::serve(
        listener,
        app.into_make_service_with_connect_info::<SocketAddr>(),
    )
    .await
    {
        Ok(t) => t,
        Err(e) => {
            eprintln!("{}", e);
            std::process::exit(1);
        }
    };
}

async fn auth(session: Session, req: Request, next: Next) -> impl IntoResponse {
    if req.method() == Method::OPTIONS {
        return next.run(req).await;
    }
    let user_id: Option<Uuid> = session.get("user_id").await.unwrap_or(None);
    if let Some(_id) = user_id {
        next.run(req).await
    } else {
        println!("Session id not correct!");
        (StatusCode::UNAUTHORIZED, "Not authenticated").into_response()
    }
}
