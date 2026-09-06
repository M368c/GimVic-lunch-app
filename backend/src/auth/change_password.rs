use crate::login;
use axum::Extension;
use axum::Json;
use axum::extract::State;
use axum::http::status::StatusCode;
use tokio;
use tower_sessions::Session;
use uuid::Uuid;

extern crate bcrypt;
use bcrypt::{DEFAULT_COST, hash, verify};

#[derive(serde::Deserialize, sqlx::FromRow)]
pub struct ChangePassword {
    password: String,
    new_password: String,
}

pub async fn change_password(
    session: Session,
    Extension(user_id): Extension<Uuid>,
    State(pool): State<sqlx::Pool<sqlx::Postgres>>,
    Json(data): Json<ChangePassword>,
) -> StatusCode {
    let q: &str = "SELECT username, password FROM users WHERE id = $1";
    let row = sqlx::query_as::<_, login::LoginData>(q)
        .bind(user_id)
        .fetch_optional(&pool)
        .await;

    let user = match row {
        Ok(Some(user)) => user,
        Ok(None) => return StatusCode::NOT_FOUND,
        Err(e) => {
            eprintln!("Error when changing password! {}", e);
            return StatusCode::INTERNAL_SERVER_ERROR;
        }
    };

    let is_password_ok = verify(data.password.as_str(), &user.password).unwrap_or(false);

    if !is_password_ok {
        return StatusCode::UNAUTHORIZED;
    }

    let hashed_new_password =
        match tokio::task::spawn_blocking(move || hash(data.new_password, DEFAULT_COST)).await {
            Ok(Ok(hash)) => hash,
            Ok(Err(_)) => return StatusCode::INTERNAL_SERVER_ERROR,
            Err(e) => {
                eprintln!("Spawn blocking task panicked: {}", e);
                return StatusCode::INTERNAL_SERVER_ERROR;
            }
        };

    let q: &str = "UPDATE users SET password = $1 WHERE id = $2";
    let new_row = sqlx::query(q)
        .bind(hashed_new_password)
        .bind(user_id)
        .execute(&pool)
        .await;

    match new_row {
        Ok(_) => {
            if let Err(e) = session.cycle_id().await {
                eprintln!("Session cycle failed: {}", e);
                return StatusCode::INTERNAL_SERVER_ERROR;
            }
            StatusCode::OK
        }
        Err(e) => {
            eprintln!("Error when changing password! {}", e);
            StatusCode::INTERNAL_SERVER_ERROR
        }
    }
}
