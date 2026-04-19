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
    dotenvy::dotenv().ok();
    let addr: &str = "127.0.0.1:3000"; // Change with correct URL for production
    let listener = tokio::net::TcpListener::bind(addr).await.unwrap();
    println!("Server running on {}", addr);

    let url: &str = "postgres://admin:dev123@localhost:5432/myapp"; // Database
    let pool: sqlx::Pool<sqlx::Postgres> = match sqlx::PgPool::connect(url).await
    {
        Ok(p) => {println!("Sucessfuly conected to db! You can view saved data on `http://127.0.0.1:8888/`"); p}
        Err(e) => {panic!("Could not connect to database: {}", e)}
    };
    sqlx::migrate!("./migrations").run(&pool).await.unwrap();

    let session_store: PostgresStore = PostgresStore::new(pool.clone());
    session_store.migrate().await.unwrap();

    let session_layer = SessionManagerLayer::new(session_store)
        .with_path("/".to_string())
        .with_http_only(true)
        .with_same_site(SameSite::Lax)
        .with_secure(false) // true in production
        .with_expiry(Expiry::OnInactivity(tower_sessions::cookie::time::Duration::days(14))); // 14 days - maybe change for production

    // CORS policy
    let cors = CorsLayer::new()
        .allow_origin("http://127.0.0.1:5500".parse::<HeaderValue>().unwrap()) // For production replace 'Any' with frontend URL
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
        .layer(cors)
        .layer(session_layer);

    axum::serve(listener, app).await.unwrap();
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
