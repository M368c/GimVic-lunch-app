use axum::extract::State;
use axum::http::status::StatusCode;
use axum::response::IntoResponse;
use axum::{Json, response::Response};
use serde::Serialize;
use std::error::Error;
use tower_sessions::Session;
use uuid::Uuid;

extern crate bcrypt;
use bcrypt::{DEFAULT_COST, hash, verify};

#[derive(Debug, Serialize, sqlx::FromRow)]
struct LoginData {
    id: Uuid, // for lunch management
    first_name: String,
    last_name: String,
    username: String,
    password: String,
}

impl Default for LoginData {
    fn default() -> Self {
        return LoginData {
            id: Uuid::new_v4(),
            first_name: "first_name".to_string(),
            last_name: "last_name".to_string(),
            username: "username".to_string(),
            password: "$2a$12$bWReyWZj/TPlHYssI112muTp0UNFZSl2BRc4C6qkPmgDBVZJFyIr.".to_string(),
        };
    }
}

#[derive(serde::Deserialize, sqlx::FromRow)]
pub struct Data {
    username: String,
    password: String,
}

#[derive(Debug, Serialize, sqlx::FromRow)]
pub struct User {
    pub first_name: String,
    pub last_name: String,
    username: String,
}

#[derive(Debug, Serialize)]
pub struct LoginResponse {
    user: User,
}

#[derive(serde::Deserialize, sqlx::FromRow)]
pub struct ChangePassword {
    password: String,
    new_password: String,
}

async fn read(
    input_username: &str,
    pool: sqlx::Pool<sqlx::Postgres>,
) -> Result<(LoginData, bool), Box<dyn Error + Send + Sync>> {
    let q = "SELECT id, first_name, last_name, username, password FROM users WHERE username = $1";
    let row = sqlx::query_as::<_, LoginData>(q)
        .bind(input_username)
        .fetch_optional(&pool)
        .await?;

    match row {
        Some(user) => Ok((user, true)),
        None => {
            let garbage_user = Default::default();
            Ok((garbage_user, false))
        }
    }
}

pub async fn login(
    session: Session,
    State(pool): State<sqlx::Pool<sqlx::Postgres>>,
    Json(data): Json<Data>,
) -> Response {
    let user_data: (LoginData, bool) = match read(data.username.as_str(), pool.clone()).await {
        Ok(t) => t,
        Err(_) => return (StatusCode::UNAUTHORIZED, "Invalid login!").into_response(),
    };
    let valid: bool = match verify(data.password.as_str(), &user_data.0.password) {
        Ok(t) => t,
        Err(_) => {
            return (
                StatusCode::INTERNAL_SERVER_ERROR,
                "Verifying password failed",
            )
                .into_response();
        }
    };

    if valid && user_data.1 {
        let response = user_data_frontend(&user_data.0);
        match session.insert("user_id", user_data.0.id).await {
            Ok(_) => (StatusCode::OK, response).into_response(),
            Err(_) => {
                (StatusCode::INTERNAL_SERVER_ERROR, "Session creation failed").into_response()
            }
        }
    } else {
        (StatusCode::UNAUTHORIZED, "Invalid login!").into_response()
    }
}

pub async fn auth_status() -> StatusCode {
    StatusCode::OK
}

pub async fn logout(session: Session) -> StatusCode {
    match session.flush().await {
        Ok(_) => StatusCode::OK,
        Err(_) => {
            eprintln!("Couldn't remove the session id!");
            StatusCode::NOT_ACCEPTABLE
        }
    }
}

pub async fn change_password(
    session: Session,
    State(pool): State<sqlx::Pool<sqlx::Postgres>>,
    Json(data): Json<ChangePassword>,
) -> StatusCode {
    let user_id: Uuid = match session.get::<Uuid>("user_id").await {
        Ok(Some(id)) => id,
        _ => return StatusCode::UNAUTHORIZED,
    };

    let q: &str = "SELECT username, password FROM users WHERE id = $1";
    let row = sqlx::query_as::<_, Data>(q)
        .bind(user_id)
        .fetch_optional(&pool)
        .await;

    let hashed_new_password = hash(data.new_password, DEFAULT_COST).unwrap();

    match row {
        Ok(Some(user)) => {
            let is_valid = verify(data.password.as_str(), &user.password);
            match is_valid {
                Ok(true) => {
                    let q: &str = "UPDATE users SET password = $1 WHERE id = $2";
                    let new_row = sqlx::query(q)
                        .bind(hashed_new_password)
                        .bind(user_id)
                        .execute(&pool)
                        .await;
                    match new_row {
                        Ok(_) => StatusCode::OK,
                        Err(_e) => StatusCode::BAD_REQUEST,
                    }
                }
                Ok(false) => StatusCode::BAD_REQUEST,
                Err(_e) => StatusCode::BAD_REQUEST,
            }
        }
        Ok(None) => {
            logout(session).await;
            StatusCode::NOT_FOUND
        }
        Err(_e) => StatusCode::BAD_REQUEST,
    }
}

fn user_data_frontend(data: &LoginData) -> Json<LoginResponse> {
    // Get data for frontend
    Json(LoginResponse {
        user: User {
            first_name: data.first_name.clone(),
            last_name: data.last_name.clone(),
            username: data.username.clone(),
        },
    })
}
