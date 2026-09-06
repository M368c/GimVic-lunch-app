use axum::Extension;
use axum::extract::{Json, State};
use axum::http::status::StatusCode;
use chrono::{Datelike, TimeZone, Utc, offset::LocalResult};
use sqlx::PgPool;
use uuid::Uuid;

use super::LunchData;

pub async fn update_lunch_data(
    Extension(user_id): Extension<Uuid>,
    State(pool): State<PgPool>,
    Json(data): Json<Vec<LunchData>>,
) -> StatusCode {
    for item in data {
        let lunch_date = item.date;
        let status = item.status;

        let current_datetime = Utc::now();
        let cancel_datetime = match Utc.with_ymd_and_hms(
            lunch_date.year(),
            lunch_date.month(),
            lunch_date.day(),
            7,
            55,
            0,
        ) {
            LocalResult::Single(v) => v,
            LocalResult::Ambiguous(a, _) => a,
            LocalResult::None => return StatusCode::INTERNAL_SERVER_ERROR,
        };

        if (cancel_datetime - current_datetime).num_hours() <= 24 {
            println!("Too late for manage lunch for that date!");
            continue;
        }

        // INSERT or DELETE
        if status == "cancel" {
            let q: &str = "INSERT INTO lunch_optouts (user_id, date, issued_date) VALUES ($1, $2, $3) ON CONFLICT DO NOTHING";

            let row = sqlx::query(q)
                .bind(user_id)
                .bind(lunch_date)
                .bind(current_datetime)
                .execute(&pool)
                .await;
            match row {
                Ok(_) => println!("Added in db for user {} and date {}", user_id, lunch_date),
                Err(e) => {
                    eprintln!("Error while inserting lunch data in db: {}", e);
                    return StatusCode::INTERNAL_SERVER_ERROR;
                }
            }
        } else if status == "ok" {
            let q: &str = "DELETE FROM lunch_optouts WHERE user_id = $1 and date = $2";
            let row = sqlx::query(q)
                .bind(user_id)
                .bind(lunch_date)
                .execute(&pool)
                .await;
            match row {
                Ok(_) => println!("Removed in db for user {} and date {}", user_id, lunch_date),
                Err(e) => {
                    eprintln!("Error while deleting lunch data in db: {}", e);
                    return StatusCode::INTERNAL_SERVER_ERROR;
                }
            }
        }
    }
    StatusCode::OK
}
