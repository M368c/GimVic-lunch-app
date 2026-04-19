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
        None => Err("User not found in database".into()), // panic! ?
    }
}

pub async fn login(session: Session, State(pool): State<sqlx::Pool<sqlx::Postgres>>, Json(data): Json<Data>) -> Response {
    let user_data: LoginData = read(data.username.as_str(), pool.clone()).await.unwrap();
    let valid: bool = verify(data.password.as_str(), &user_data.password).unwrap();

    if valid {
        let response = user_data_frontend(&user_data);
        //Ok((data.id, response))
        println!("Login successful!");
        println!("User id is {:?}", user_data.id);
        session.insert("user_id", user_data.id).await.unwrap();
        (StatusCode::OK, response).into_response()
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
    match session.get::<Uuid>("user_id").await {
        Ok(Some(_)) => {
            session.flush().await.unwrap();
            StatusCode::OK}
        Ok(None) => {StatusCode::OK}
        Err(e) => {println!("{}", e); StatusCode::OK}
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