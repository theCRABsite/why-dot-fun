use axum::response::IntoResponse;
use axum::Json;
use axum::Extension;
use crate::Database;
use serde::{Serialize, Deserialize};
use crate::StatusCode;


#[derive(Deserialize)]
pub struct WinnerRequest {
    phone_number: String,
}


#[derive(Serialize)]
pub struct WinnerResponse {
    is_winner: Option<bool>,
    winner_url: Option<String>,
}


pub async fn verify_winner(
    Extension(database): Extension<Database>,
    Json(request): Json<WinnerRequest>,
) -> impl IntoResponse {

    let result = database
        .get_attempt_result_by_phone_number(request.phone_number)
        .await
        .unwrap();


    let is_winner = result.clone().unwrap().is_winner;
    let winner_url = result.clone().unwrap().winner_url;

    let response = WinnerResponse {
        is_winner: is_winner,
        winner_url: Some(winner_url),
    };

    (StatusCode::OK, Json(response)).into_response()
}