use axum::Json;
use axum::extract::State;
use axum::http::status::StatusCode;
use axum::response::IntoResponse;
use chrono::{NaiveDate, Utc};
use serde::Serialize;
use std::error::Error;
use tower_sessions::Session;
use uuid::Uuid;

#[derive(Serialize, sqlx::FromRow)]
pub struct LunchOptuots {
    pub date: NaiveDate,
}

#[derive(serde::Serialize)]
pub struct DateResponse {
    status: String,
    date: NaiveDate,
}

pub async fn lunch_handling(
    session: Session,
    State(pool): State<sqlx::Pool<sqlx::Postgres>>,
    Json(data): Json<serde_json::Value>,
) -> impl IntoResponse {
    // Lunch
    let user_id: Uuid = match session.get::<Uuid>("user_id").await {
        Ok(Some(id)) => id,
        _ => return (StatusCode::UNAUTHORIZED, "Session expired").into_response(),
    };
    match main(user_id, data, pool).await {
        Ok(_) => (StatusCode::OK, "Ok").into_response(),
        Err(e) => {
            println!("Error in lunch_handling: {}", e);
            (
                StatusCode::BAD_REQUEST,
                "Error while getting lunch data from db!",
            )
                .into_response()
        }
    }
}

pub async fn lunch_data(
    session: Session,
    State(pool): State<sqlx::Pool<sqlx::Postgres>>,
) -> impl IntoResponse {
    let user_id: Uuid = match session.get::<Uuid>("user_id").await {
        Ok(Some(id)) => id,
        _ => {
            return {
                println!("Session expired! (backend)");
                (StatusCode::UNAUTHORIZED, "Session expired").into_response()
            };
        }
    };
    match get_lunch_data(user_id, pool).await {
        Ok(response) => {
            //println!("Success on searching for user lunch optouts in db (backend)");
            response.into_response()
        }
        Err(e) => {
            println!("Error in lunch_data: {}", e);
            (
                StatusCode::BAD_REQUEST,
                "Error while getting lunch data from db!",
            )
                .into_response()
        }
    }
}

pub async fn main(
    user_id: Uuid,
    data: serde_json::Value,
    pool: sqlx::Pool<sqlx::Postgres>,
) -> Result<(), Box<dyn Error>> {
    update_database(user_id, data, pool.clone()).await?;
    Ok(())
}

pub async fn get_lunch_data(
    user_id: Uuid,
    pool: sqlx::Pool<sqlx::Postgres>,
) -> Result<Json<Vec<DateResponse>>, Box<dyn Error>> {
    // Get lunch data from database
    let q: &str = "SELECT date FROM lunch_optouts WHERE user_id = $1";
    let rows: Vec<LunchOptuots> = sqlx::query_as::<_, LunchOptuots>(q)
        .bind(user_id)
        .fetch_all(&pool)
        .await?;

    // Join rows in multiple response
    let response = rows
        .into_iter()
        .map(|row| DateResponse {
            status: "cancel".to_string(),
            date: row.date,
        })
        .collect();

    Ok(Json(response))
}

async fn update_database(
    user_id: Uuid,
    data: serde_json::Value,
    pool: sqlx::Pool<sqlx::Postgres>,
) -> Result<(), Box<dyn Error>> {
    if let Some(array) = data.as_array() {
        for item in array {
            let date_str = item
                .get("date")
                .and_then(|v| v.as_str())
                .expect("Missing date field!");
            let status = item
                .get("status")
                .and_then(|v| v.as_str())
                .expect("Missing status field!");
            let parsed_date =
                NaiveDate::parse_from_str(date_str, "%Y-%m-%d").expect("Invalid date format!");
            let issued_date = Utc::now();

            // INSERT or DELETE
            if status == "cancel" {
                let q: &str =
                    "INSERT INTO lunch_optouts (user_id, date, issued_date) VALUES ($1, $2, $3)";

                let _row = sqlx::query(q)
                    .bind(user_id)
                    .bind(parsed_date)
                    .bind(issued_date)
                    .execute(&pool)
                    .await?;
                println!("Added in db for user {} and date {}", user_id, date_str);
            } else if status == "ok" {
                let q: &str = "DELETE FROM lunch_optouts WHERE user_id = $1 and date = $2";
                let _row = sqlx::query(q)
                    .bind(user_id)
                    .bind(parsed_date)
                    .execute(&pool)
                    .await?;
                println!("Removed in db for user {} and date {}", user_id, date_str);
            }
        }
    }
    Ok(())
}
