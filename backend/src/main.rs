mod auth;
mod lunch;
use crate::header::HeaderValue;
use auth::login;
use axum::extract::FromRef;
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
use lettre::SmtpTransport;
use lettre::transport::smtp::authentication::Credentials;
use lunch::lunch_management;
use tower_http::cors::CorsLayer;
use tower_sessions::{Expiry, Session, SessionManagerLayer};
use tower_sessions_sqlx_store_chrono::PostgresStore;
use uuid::Uuid;

#[derive(Clone, FromRef)]
struct LunchState {
    pool: sqlx::PgPool,
    mailer: SmtpTransport,
}

#[tokio::main]
async fn main() {
    dotenvy::dotenv().ok(); // Load .env first
    let addr: &str = "127.0.0.1:3000"; // Change with correct URL for production
    let listener = match tokio::net::TcpListener::bind(addr).await {
        Ok(t) => t,
        Err(e) => {
            eprintln!("Failed to bind to {}: {}", addr, e);
            std::process::exit(1);
        }
    };
    println!("Server running on {}", addr);

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

    let creds = Credentials::new(
        match std::env::var("SMTP_USERNAME").to_owned() {
            Ok(t) => t,
            Err(e) => {
                eprintln!("Could not get username for email services: {}", e);
                std::process::exit(1);
            }
        },
        match std::env::var("SMTP_KEY").to_owned() {
            Ok(t) => t,
            Err(e) => {
                eprintln!("Could not get api key for email services: {}", e);
                std::process::exit(1);
            }
        },
    );

    let mailer = match SmtpTransport::relay("smtp-relay.brevo.com") {
        Ok(t) => t,
        Err(e) => {
            eprintln!("Failed to set connection for email: {}", e);
            std::process::exit(1);
        }
    }
    .credentials(creds)
    .build();

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
        .route("/auth_status", get(login::auth_status))
        .route("/lunch_data", get(lunch_management::lunch_data))
        .route("/logout", post(login::logout))
        .route("/change_password", post(login::change_password))
        .route_layer(middleware::from_fn(auth));

    let lunch_state = LunchState {
        pool: pool.clone(),
        mailer: mailer,
    };

    let lunch_route = Router::new()
        .route("/update_lunch_data", post(lunch_management::lunch_handling))
        .with_state(lunch_state)
        .route_layer(middleware::from_fn(auth));

    let app = Router::new()
        .merge(protected_routes)
        .merge(lunch_route)
        .route("/login", post(login::login))
        .with_state(pool)
        .layer(session_layer)
        .layer(cors);

    match axum::serve(listener, app).await {
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
