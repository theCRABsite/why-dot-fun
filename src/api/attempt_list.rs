use axum::response::IntoResponse;
use axum::Json;
use crate::api::{Attempt, AttemptReturn};
use axum::Extension;
use crate::Database;

// Implement the From trait for AttemptReturn
impl From<Attempt> for AttemptReturn {
    fn from(attempt: Attempt) -> Self {
        AttemptReturn {
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
        }
    }
}

pub async fn attempt_list(
    Extension(database): Extension<Database>,
) -> impl IntoResponse {

    let attempt_list: Vec<AttemptReturn> = database
        .get_all_attempts_last_14_days()
        .await
        .unwrap_or(vec![])
        .into_iter()
        .map(AttemptReturn::from)
        .filter(|attempt| attempt.video_url.as_ref().map_or(false, |url| !url.is_empty()))
        .collect();

    Json(attempt_list).into_response()
}