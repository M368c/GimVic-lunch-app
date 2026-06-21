use axum::extract::{Json, State};
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

#[derive(serde::Serialize, serde::Deserialize, Debug)]
pub struct LunchData {
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
) -> Result<Json<Vec<LunchData>>, Box<dyn Error>> {
    // Get lunch data from database
    let q: &str = "SELECT date FROM lunch_optouts WHERE user_id = $1";
    let q1: &str = "SELECT date FROM holidays";
    let rows: Vec<LunchOptuots> = sqlx::query_as::<_, LunchOptuots>(q)
        .bind(user_id)
        .fetch_all(&pool)
        .await?;
    let rows1: Vec<LunchOptuots> = sqlx::query_as::<_, LunchOptuots>(q1)
        .fetch_all(&pool)
        .await?;

    // Join rows in multiple response
    let mut response: Vec<LunchData> = rows
        .into_iter()
        .map(|row| LunchData {
            status: "cancel".to_string(),
            date: row.date,
        })
        .collect();

    let mut response1: Vec<LunchData> = rows1
        .into_iter()
        .map(|row| LunchData {
            status: "holiday".to_string(),
            date: row.date,
        })
        .collect();
    response.append(&mut response1);

    Ok(Json(response))
}

#[axum::debug_handler]
pub async fn lunch_handling(
    session: Session,
    State(pool): State<PgPool>,
    Json(lunch_data): Json<Vec<LunchData>>,
) -> StatusCode {
    let user_id: Uuid = match session.get::<Uuid>("user_id").await {
        Ok(Some(id)) => id,
        _ => return StatusCode::UNAUTHORIZED,
    };

    match update_database(user_id, lunch_data, pool.clone()).await {
        Ok(_) => StatusCode::OK,
        Err(e) => {
            println!("{}", e);
            StatusCode::BAD_REQUEST
        }
    }
}

async fn update_database(
    user_id: Uuid,
    lunch_data: Vec<LunchData>,
    pool: sqlx::Pool<sqlx::Postgres>,
) -> Result<(), Box<dyn std::error::Error>> {
    for item in lunch_data {
        let lunch_date = item.date;
        let status = item.status;

        let current_datetime = Utc::now();

        let cancel_datetime = Utc
            .with_ymd_and_hms(
                lunch_date.year(),
                lunch_date.month(),
                lunch_date.day(),
                8,
                0,
                0,
            )
            .unwrap();

        if (cancel_datetime - current_datetime).num_hours() <= 24 {
            println!("Too late for manage lunch for that date!");
        } else {
            // INSERT or DELETE
            if status == "cancel" {
                let q: &str =
                    "INSERT INTO lunch_optouts (user_id, date, issued_date) VALUES ($1, $2, $3)";

                let _row = sqlx::query(q)
                    .bind(user_id)
                    .bind(lunch_date)
                    .bind(current_datetime)
                    .execute(&pool)
                    .await?;
                println!("Added in db for user {} and date {}", user_id, lunch_date);
            } else if status == "ok" {
                let q: &str = "DELETE FROM lunch_optouts WHERE user_id = $1 and date = $2";
                let _row = sqlx::query(q)
                    .bind(user_id)
                    .bind(lunch_date)
                    .execute(&pool)
                    .await?;
                println!("Removed in db for user {} and date {}", user_id, lunch_date);
            }
        }
    }
    Ok(())
}
