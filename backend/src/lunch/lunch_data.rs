use axum::Extension;
use axum::extract::State;
use axum::http::status::StatusCode;
use axum::response::IntoResponse;
use axum::{Json, response::Response};
use chrono::NaiveDate;
use serde::Serialize;
use uuid::Uuid;

use super::LunchData;

#[derive(Serialize, sqlx::FromRow)]
pub struct LunchOptuots {
    pub date: NaiveDate,
}

pub async fn get_lunch_data(
    Extension(user_id): Extension<Uuid>,
    State(pool): State<sqlx::Pool<sqlx::Postgres>>,
) -> Response {
    let q: &str = "SELECT date FROM lunch_optouts WHERE user_id = $1";
    let q1: &str = "SELECT date FROM holidays";
    let dates: Vec<LunchOptuots> = match sqlx::query_as::<_, LunchOptuots>(q)
        .bind(user_id)
        .fetch_all(&pool)
        .await
    {
        Ok(data) => data,
        Err(e) => {
            eprintln!("Error while trying to get lunch data: {}", e);
            return (
                StatusCode::INTERNAL_SERVER_ERROR,
                "Error while getting lunch data from db!",
            )
                .into_response();
        }
    };

    let holidays: Vec<LunchOptuots> =
        match sqlx::query_as::<_, LunchOptuots>(q1).fetch_all(&pool).await {
            Ok(data) => data,
            Err(e) => {
                eprintln!("Error while trying to get holiday data: {}", e);
                return (
                    StatusCode::INTERNAL_SERVER_ERROR,
                    "Error while getting lunch data from db!",
                )
                    .into_response();
            }
        };

    let mut response: Vec<LunchData> = dates
        .into_iter()
        .map(|row| LunchData {
            status: "cancel".to_string(),
            date: row.date,
        })
        .collect();

    let mut response1: Vec<LunchData> = holidays
        .into_iter()
        .map(|row| LunchData {
            status: "holiday".to_string(),
            date: row.date,
        })
        .collect();
    response.append(&mut response1);

    (StatusCode::OK, Json(response)).into_response()
}
