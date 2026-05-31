use axum::Json;
use axum::extract::State;
use axum::http::status::StatusCode;
use axum::response::IntoResponse;
use chrono::{Datelike, NaiveDate, TimeZone, Utc};
use core::result::Result;
use lettre::message::{Mailbox, header::ContentType};
use lettre::{Address, Message, SmtpTransport, Transport};
use serde::Serialize;
use sqlx::PgPool;
use std::error::Error;
use tower_sessions::Session;
use uuid::Uuid;

use crate::auth::login;

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
    State(mailer): State<SmtpTransport>,
    lunch_update: Json<serde_json::Value>,
) -> StatusCode {
    let user_id: Uuid = match session.get::<Uuid>("user_id").await {
        Ok(Some(id)) => id,
        _ => return StatusCode::UNAUTHORIZED,
    };

    let q: &str = "SELECT first_name, last_name, username FROM users WHERE id = $1";
    let row = sqlx::query_as::<_, login::User>(q)
        .bind(user_id)
        .fetch_optional(&pool)
        .await;

    match row {
        Ok(Some(user)) => {
            // Pass the user in update_database
            match update_database(user_id, user, lunch_update, pool.clone(), mailer).await {
                Ok(_) => StatusCode::OK,
                Err(e) => {
                    println!("{}", e);
                    StatusCode::BAD_REQUEST
                }
            }
        }
        _ => {
            println!(
                "Error while getting user data for sending emails! User: {}.",
                user_id
            );
            StatusCode::BAD_REQUEST
        }
    }
}

async fn update_database(
    user_id: Uuid,
    user: login::User,
    lunch_update: Json<serde_json::Value>,
    pool: sqlx::Pool<sqlx::Postgres>,
    mailer: SmtpTransport,
) -> Result<(), Box<dyn Error>> {
    // Set up emails
    let from_email: Address = match std::env::var("FROM_EMAIL") {
        Ok(t) => t.parse().expect("Error parsing FROM_EMAIL var"),
        Err(e) => {
            eprintln!("FROM_EMAIL env var must be set! {}", e);
            std::process::exit(1);
        }
    };

    let reply_to_email: Address = match std::env::var("REPLY_TO_EMAIL") {
        Ok(t) => t.parse().expect("Error parsing REPLY_TO_EMAIL var"),
        Err(e) => {
            eprintln!("REPLY_TO_EMAIL env var must be set! {}", e);
            std::process::exit(1);
        }
    };

    let to_email: Address = match std::env::var("TO_EMAIL") {
        Ok(t) => t.parse().expect("Error parsing TO_EMAIL var"),
        Err(e) => {
            eprintln!("TO_EMAIL env var must be set! {}", e);
            std::process::exit(1);
        }
    };

    // Email build
    let email_base = Message::builder()
        .from(Mailbox::new(
            Some("Gimvič lunch app".to_owned()),
            from_email,
        ))
        .reply_to(Mailbox::new(Some("User".to_owned()), reply_to_email))
        .to(Mailbox::new(
            Some("Prehrana Gimnazija Vič".to_owned()),
            to_email,
        ));

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

    let mut subject = String::new();
    let mut email_message = String::new();

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
        subject = String::from("Odjava kosila");
        email_message = format!(
            "<html lang='sl'><body>Odjava od kosila za dan {date_str}, dijak {} {}.
            <p style='display:none'>Sporočilo je bilo uspešno poslano in samodejno ustvarjeno. Hvala za vašo pozornost.
            </p></body></html>",
            user.first_name, user.last_name
        );
    } else if status == "ok" {
        let q: &str = "DELETE FROM lunch_optouts WHERE user_id = $1 and date = $2";
        let _row = sqlx::query(q)
            .bind(user_id)
            .bind(cancel_date)
            .execute(&pool)
            .await?;
        println!("Removed in db for user {} and date {}", user_id, date_str);
        subject = String::from("Prijava kosila");
        email_message = format!(
            "<html lang='sl'><body>Prijava na kosilo za dan {date_str}, dijak {} {}.
            <p style='display:none'>Sporočilo je bilo uspešno poslano in samodejno ustvarjeno. Hvala za vašo pozornost.
            </p></body></html>",
            user.first_name, user.last_name
        );
    }

    let mut email = email_base.clone();
    email = email.subject(subject);

    let final_email = email
        .header(ContentType::TEXT_HTML)
        .body(email_message)
        .expect("Failed to build email");

    match mailer.send(&final_email) {
        Ok(_) => println!("Email sent successfully!"),
        Err(e) => {
            println!("Could not send email: {e:?}");
            std::process::exit(1);
        }
    }
    Ok(())
}
