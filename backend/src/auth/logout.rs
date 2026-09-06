use axum::http::status::StatusCode;
use tower_sessions::Session;

pub async fn logout(session: Session) -> StatusCode {
    match session.flush().await {
        Ok(_) => StatusCode::OK,
        Err(e) => {
            eprintln!("Couldn't remove the session id! {}", e);
            StatusCode::INTERNAL_SERVER_ERROR
        }
    }
}
