use axum::response::IntoResponse;
use axum::Json;
use axum::Extension;
use crate::Database;
use serde::Deserialize;
use solana_sdk::signature::Signature;
use solana_sdk::pubkey::Pubkey;
use std::str::FromStr;
use crate::StatusCode;
use crate::api::ReturnSponsor;


#[derive(Deserialize, Clone, Debug)]
pub struct SponsorListArgs {
    public_key: String,
    signature: String,
}

pub async fn sponsor_list(
    Extension(database): Extension<Database>,
    Json(request): Json<SponsorListArgs>,
) -> impl IntoResponse {

    let signature = request.signature;
    let public_key = request.public_key;

    // Convert the signature and public key from strings to their respective types
    let signature = Signature::from_str(&signature).expect("Invalid signature format");
    let public_key = Pubkey::from_str(&public_key).expect("Invalid public key format");

    let message = chrono::Utc::now().format("%Y-%m-%d %H:00:00").to_string();

    // Verify the signature
    if !signature.verify(&public_key.to_bytes(), message.as_bytes()) {
        return (StatusCode::BAD_REQUEST, Json("Invalid signature")).into_response();
    }

    let sponsor_list = database
        .get_sponsor_by_user_id(public_key.to_string())
        .await
        .expect("Failed to get sponsor");

    // Transform each sponsor into a ReturnSponsor object
    let return_sponsor_list: Vec<ReturnSponsor> = sponsor_list.into_iter().map(|sponsor| {
        ReturnSponsor {
            id: sponsor.id,
            name: sponsor.name,
            user_id: sponsor.user_id,
            active: sponsor.active,
            background_url: sponsor.background_url,
            public_key: sponsor.public_key,
            token_mint: sponsor.token_mint,
            original_tokens: sponsor.original_tokens,
            available_tokens: sponsor.available_tokens,
            reward_tokens: sponsor.reward_tokens,
            challenge_text: sponsor.challenge_text,
            challenge_time: sponsor.challenge_time,
            system_instruction: sponsor.system_instruction,
            start_text: sponsor.start_text,
            won_text: sponsor.won_text,
            lost_text: sponsor.lost_text,
            greeting_text: sponsor.greeting_text,
            end_text: sponsor.end_text,
            rating_threshold: sponsor.rating_threshold,
            initial_funded: sponsor.initial_funded,
        }
    }).collect();

    (StatusCode::OK, Json(return_sponsor_list)).into_response()
}
