use axum::response::IntoResponse;
use axum::extract::Path;
use axum::Json;
use crate::api::AttemptReturn;
use axum::Extension;
use crate::Database;

pub async fn attempt_single(
    Extension(database): Extension<Database>,
    Path(public_key): Path<String>,
) -> impl IntoResponse {

    if let Some(attempt) = database.get_attempt_by_pubkey(public_key).await.unwrap_or(None) {
        let attempt_return = AttemptReturn {
            id: attempt.id,
            created_at: attempt.created_at,
            updated_at: attempt.updated_at,
            video_url: attempt.video_url,
            twitter_url: attempt.twitter_url,
            is_winner: attempt.is_winner,
            sponsor_question: attempt.sponsor_question,
            sponsor_name: attempt.sponsor_name,
            sponsor_token_mint: attempt.sponsor_token_mint,
            sponsor_background_url: attempt.sponsor_background_url,
            sponsor_challenge_time: attempt.sponsor_challenge_time,
            challenge_transcript: attempt.challenge_transcript,
            challenge_status: attempt.challenge_status,
        };

        Json(attempt_return).into_response()
    } else {
        (axum::http::StatusCode::NOT_FOUND, "Attempt not found").into_response()
    }
}