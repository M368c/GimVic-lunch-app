mod auth;
mod lunch;
use auth::login;
use lunch::lunch_management;
use crate::header::HeaderValue;
use axum::{Router, http::{header, Method, status::StatusCode}, extract::Request, response::{IntoResponse}, routing::{post, get}};
use axum::middleware;
use axum::middleware::Next;
use axum_extra::extract::cookie::SameSite;
use tower_http::cors::{CorsLayer};
use tower_sessions_sqlx_store_chrono::PostgresStore;
use tower_sessions::{Session, Expiry, SessionManagerLayer};
use uuid::Uuid;


#[tokio::main]
async fn main() {
    dotenvy::dotenv().ok(); // Load .env first
    let addr: &str = "0.0.0.0:3000"; // Change with correct URL for production
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
    let pool = sqlx::postgres::PgPoolOptions::new()
        .max_connections(10)
        .acquire_timeout(std::time::Duration::from_secs(3))
        .idle_timeout(std::time::Duration::from_secs(600))
        .connect(&database_connection)
        .await
        .expect("Could not connect to database");
    
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
        .with_expiry(Expiry::OnInactivity(tower_sessions::cookie::time::Duration::days(14))); // 14 days - maybe change for production

    // CORS policy
    let cors = CorsLayer::new()
        .allow_origin("http://127.0.0.1:5500".parse::<HeaderValue>().unwrap()) // For production replace with domain name
        .allow_methods([Method::GET, Method::POST, Method::OPTIONS])
        .allow_headers([header::CONTENT_TYPE, header::AUTHORIZATION])
        .allow_credentials(true);

    let protected_routes = Router::new()
        .route("/auth_status", get(login::auth_status))
        .route("/lunch_data", get(lunch_management::lunch_data))
        .route("/update_lunch_data", post(lunch_management::lunch_handling))
        .route("/logout", post(login::logout))
        .route("/change_password", post(login::change_password))
        .route_layer(middleware::from_fn(auth));

    let app = Router::new()
        .merge(protected_routes)
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
        //println!("Session id CORRECT! (backend)");
        next.run(req).await
    } else {
        println!("Session id not correct! (backend)");
        (StatusCode::UNAUTHORIZED, "Not authenticated").into_response()
    }
}
