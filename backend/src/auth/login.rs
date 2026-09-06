use axum::extract::State;
use axum::http::status::StatusCode;
use axum::response::IntoResponse;
use axum::{Json, response::Response};
use serde::Serialize;
use tower_sessions::Session;
use uuid::Uuid;

extern crate bcrypt;
use bcrypt::verify;

#[derive(Debug, Serialize, sqlx::FromRow)]
struct UserData {
    id: Uuid,
    first_name: String,
    last_name: String,
    username: String,
    password: String,
}

#[derive(serde::Deserialize, sqlx::FromRow)]
pub struct LoginData {
    username: String,
    pub password: String,
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

pub async fn auth_status() -> StatusCode {
    StatusCode::OK
}

impl Default for UserData {
    fn default() -> Self {
        return UserData {
            id: Uuid::new_v4(),
            first_name: "first_name".to_string(),
            last_name: "last_name".to_string(),
            username: "username".to_string(),
            password: "$2a$12$bWReyWZj/TPlHYssI112muTp0UNFZSl2BRc4C6qkPmgDBVZJFyIr.".to_string(),
        };
    }
}

pub async fn login(
    session: Session,
    State(pool): State<sqlx::Pool<sqlx::Postgres>>,
    Json(data): Json<LoginData>,
) -> Response {
    let q = "SELECT id, first_name, last_name, username, password FROM users WHERE username = $1";
    let row = sqlx::query_as::<_, UserData>(q)
        .bind(data.username.as_str())
        .fetch_optional(&pool)
        .await;

    let mut user_data: (UserData, bool) = (Default::default(), false);

    match row {
        Ok(Some(user)) => user_data = (user, true),
        Ok(None) => (),
        Err(_) => (),
    }

    let is_valid: bool = verify(data.password.as_str(), &user_data.0.password).unwrap_or(false);

    if is_valid && user_data.1 == true {
        let response = user_data_frontend(&user_data.0);
        if let Err(e) = session.insert("user_id", &user_data.0.id).await {
            eprintln!("Session creation failed: {}", e);
            return (StatusCode::INTERNAL_SERVER_ERROR, "Session creation failed").into_response();
        }
        if let Err(e) = session.cycle_id().await {
            eprintln!("Session cycle failed: {}", e);
            return (StatusCode::INTERNAL_SERVER_ERROR, "Session cycle failed").into_response();
        }

        (StatusCode::OK, response).into_response()
    } else {
        (StatusCode::UNAUTHORIZED, "Invalid login!").into_response()
    }
}

fn user_data_frontend(data: &UserData) -> Json<LoginResponse> {
    Json(LoginResponse {
        user: User {
            first_name: data.first_name.clone(),
            last_name: data.last_name.clone(),
            username: data.username.clone(),
        },
    })
}
