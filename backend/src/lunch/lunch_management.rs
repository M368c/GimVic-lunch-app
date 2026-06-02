use axum::Json;
use axum::extract::State;
use axum::http::status::StatusCode;
use axum::response::IntoResponse;
use chrono::{Datelike, NaiveDate, TimeZone, Utc};
use core::result::Result;
use serde::Serialize;
use sqlx::PgPool;
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

pub async fn lunch_data(
    session: Session,
    State(pool): State<sqlx::Pool<sqlx::Postgres>>,
) -> impl IntoResponse {
    let user_id: Uuid = match session.get::<Uuid>("user_id").await {
        Ok(Some(id)) => id,
        _ => {
            return {
                println!("Session expired!");
                (StatusCode::UNAUTHORIZED, "Session expired").into_response()
            };
        }
    };
    match get_lunch_data(user_id, pool).await {
        Ok(response) => response.into_response(),
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

pub async fn lunch_handling(
    session: Session,
    State(pool): State<PgPool>,
    lunch_update: Json<serde_json::Value>,
) -> StatusCode {
    let user_id: Uuid = match session.get::<Uuid>("user_id").await {
        Ok(Some(id)) => id,
        _ => return StatusCode::UNAUTHORIZED,
    };

    match update_database(user_id, lunch_update, pool.clone()).await {
        Ok(_) => StatusCode::OK,
        Err(e) => {
            println!("{}", e);
            StatusCode::BAD_REQUEST
        }
    }
}

async fn update_database(
    user_id: Uuid,
    lunch_update: Json<serde_json::Value>,
    pool: sqlx::Pool<sqlx::Postgres>,
) -> Result<(), Box<dyn Error>> {
    let date_str = lunch_update
        .get("date")
        .and_then(|v| v.as_str())
        .expect("Invalid date format!");
    let status = lunch_update
        .get("status")
        .and_then(|v| v.as_str())
        .expect("Invalid date format!");
    let cancel_date =
        NaiveDate::parse_from_str(date_str, "%Y-%m-%d").expect("Invalid date format!");
    let current_datetime = Utc::now();

    let cancel_datetime = Utc
        .with_ymd_and_hms(
            cancel_date.year(),
            cancel_date.month(),
            cancel_date.day(),
            8,
            0,
            0,
        )
        .unwrap();

    if (cancel_datetime - current_datetime).num_hours() <= 24 {
        println!("Too late for manage lunch for that date!");
        return Ok(());
    }

    // INSERT or DELETE
    if status == "cancel" {
        let q: &str = "INSERT INTO lunch_optouts (user_id, date, issued_date) VALUES ($1, $2, $3)";

        let _row = sqlx::query(q)
            .bind(user_id)
            .bind(cancel_date)
            .bind(current_datetime)
            .execute(&pool)
            .await?;
        println!("Added in db for user {} and date {}", user_id, date_str);
    } else if status == "ok" {
        let q: &str = "DELETE FROM lunch_optouts WHERE user_id = $1 and date = $2";
        let _row = sqlx::query(q)
            .bind(user_id)
            .bind(cancel_date)
            .execute(&pool)
            .await?;
        println!("Removed in db for user {} and date {}", user_id, date_str);
    }

    Ok(())
}
