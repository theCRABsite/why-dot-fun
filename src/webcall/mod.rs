use axum::{routing::post, Router};
use tower_http::services::ServeFile;

mod check;
mod token;

pub fn router() -> Router {
    Router::new()
        .nest_service("/", ServeFile::new("static/call.html"))
        .route("/twilio-token", post(token::generate_jwt))
}
