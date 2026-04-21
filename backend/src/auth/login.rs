use std::error::Error;
use uuid::Uuid;
use axum::{Json, response::Response};
use axum::http::status::StatusCode;
use axum::response::IntoResponse;
use axum::extract::State;
use serde::Serialize;
use tower_sessions::Session;

extern crate bcrypt;
use bcrypt::verify;

#[derive(Debug, Serialize, sqlx::FromRow)]
struct LoginData {
    id: Uuid, // for lunch management
    first_name: String,
    last_name: String,
    username: String,
    password: String,
}

#[derive(serde::Deserialize)]
pub struct Data {
    username: String,
    password: String,
}

#[derive(Debug, Serialize)]
pub struct User {
    first_name: String,
    last_name: String,
    username: String,
}

#[derive(Debug, Serialize)]
pub struct LoginResponse {
    user: User,
}

async fn read(input_username: &str, pool: sqlx::Pool<sqlx::Postgres>) -> Result<LoginData, Box<dyn Error + Send + Sync>> {
    let q = "SELECT id, first_name, last_name, username, password FROM users WHERE username = $1";
    let row = sqlx::query_as::<_, LoginData>(q)
        .bind(input_username)
        .fetch_optional(&pool)
        .await?;

    match row {
        Some(user) => Ok(user),
        None => {
            println!("User not found in database");
            Err("User not found in database".into())
        }
    }
}

pub async fn login(session: Session, State(pool): State<sqlx::Pool<sqlx::Postgres>>, Json(data): Json<Data>) -> Response {
    let user_data: LoginData = match read(data.username.as_str(), pool.clone()).await {
        Ok(t) => t,
        Err(_) => return (StatusCode::INTERNAL_SERVER_ERROR, "Reading db in login failed").into_response()
    };
    let valid: bool = match verify(data.password.as_str(), &user_data.password) {
        Ok(t) => t,
        Err(_) => return (StatusCode::INTERNAL_SERVER_ERROR, "Verifying password failed").into_response()
    };

    if valid {
        let response = user_data_frontend(&user_data);
        //Ok((data.id, response))
        println!("Login successful!");
        println!("User id is {:?}", user_data.id);
        match session.insert("user_id", user_data.id).await {
            Ok(_) => {
                (StatusCode::OK, response).into_response()
            }
            Err(_) => {
                (StatusCode::INTERNAL_SERVER_ERROR, "Session creation failed").into_response()
            }
        }
    } else {
        println!("Login failed!");
        (StatusCode::UNAUTHORIZED, "Invalid login!").into_response()
        //Err("Wrong login data!".into())
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

pub async fn change_password() {
    println!("Changing password!");
}

fn user_data_frontend(data: &LoginData) -> Json<LoginResponse> {
    // Get data for frontend
    Json(LoginResponse {
        user: User {
            first_name: data.first_name.clone(),
            last_name: data.last_name.clone(),
            username: data.username.clone(),
        }
    })
}