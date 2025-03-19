use axum::response::IntoResponse;
use axum::Json;
use axum::Extension;
use crate::solana::verify_payment::verify_payment;
use crate::database::Sponsor;
use crate::api::SponsorArgs;
use crate::Database;
use anyhow::Context;
use crate::StatusCode;
use serde::Serialize;
use crate::solana::keys::generate_private_key;
use solana_sdk::signer::Signer;
use crate::secrets::Secrets;
use base64::{engine::general_purpose, Engine as _};
use bincode;
use solana_sdk::transaction::Transaction;
use crate::api::ResponseData;


#[derive(Serialize)]
pub struct ReturnSponsor {
    pub id: i32,
    pub name: String,
    pub user_id: String,
    pub active: bool,
    pub background_url: String,
    pub public_key: String,
    pub token_mint: String,
    pub original_tokens: i64,
    pub available_tokens: i64,
    pub reward_tokens: i64,
    pub challenge_text: String,
    pub challenge_time: i32,
    pub start_text: String,
    pub system_instruction: String,
    pub won_text: String,
    pub lost_text: String,
    pub greeting_text: String,
    pub end_text: String,
    pub rating_threshold: i32,
    pub initial_funded: bool,
}


pub async fn launchpad(
    secrets: Extension<Secrets>,
    Extension(database): Extension<Database>,
    Json(new_sponsor): Json<SponsorArgs>,
) -> impl IntoResponse {
    let challenge: String = String::from("Lets start the game: ");


    let private_key = generate_private_key();
    let public_key = private_key.pubkey().to_string();
    let private_key_base58 = private_key.to_base58_string();

    let sponsor = Sponsor {
        id: 1,
        name: new_sponsor.name.trim().to_string(),
        user_id: new_sponsor.user_id.trim().to_string(),
        active: false,
        background_url: new_sponsor.background_url.trim().to_string(),
        private_key: private_key_base58,
        public_key: public_key.to_string(),
        token_mint: new_sponsor.token_mint.trim().to_string(),
        original_tokens: new_sponsor.original_tokens,
        available_tokens: new_sponsor.original_tokens,
        reward_tokens: new_sponsor.reward_tokens,
        challenge_time: if new_sponsor.challenge_time > 60 {
            60
        } else {
            new_sponsor.challenge_time
        },
        system_instruction: new_sponsor.system_instruction,
        greeting_text: "Welcome to Why dot Fun. Please tell me your name to start the game.".to_string(),
        challenge_text: new_sponsor.challenge.clone(),
        start_text: format!("{} {}", challenge, new_sponsor.challenge),
        end_text: "Alright, your time is up! Thank you for participating. You will receive a text message with the results of your attempt. If you are calling from the United States, visit claim.why.fun to check your result. Callers from the US will not receive a text message, please check your result on claim.why.fun. Thank you for playing today!".to_string(),
        won_text: "Congratulations {name}, you won! Claim your prize: {link}. View the video of your attempt here: {video_url} (it will be ready in around 15 minutes)".to_string(),
        lost_text: "Unfortunately, you did not win this time. Better luck next time! Check out https://x.com/whydotfun for tips and tricks to improve your chances.".to_string(),
        rating_threshold: new_sponsor.rating_threshold,
        initial_funded: false,
    };

    // Decode the base64-encoded transaction
    let decoded_transaction = general_purpose::STANDARD
        .decode(&new_sponsor.transaction)
        .expect("Failed to decode transaction");

    // Deserialize the transaction
    let transaction: Transaction = bincode::deserialize(&decoded_transaction)
        .expect("Failed to deserialize transaction");

    let signature = verify_payment(&secrets, transaction).await.expect("Failed to verify payment");

    let sponsor_entry = database
        .create_sponsor(sponsor)
        .await
        .context("Creating sponsor")
        .expect("Failed to create sponsor");

    let return_sponsor = ReturnSponsor {
        id: sponsor_entry.id,
        name: sponsor_entry.name,
        user_id: sponsor_entry.user_id,
        active: sponsor_entry.active,
        background_url: sponsor_entry.background_url,
        public_key: sponsor_entry.public_key,
        token_mint: sponsor_entry.token_mint,
        original_tokens: sponsor_entry.original_tokens,
        available_tokens: sponsor_entry.available_tokens,
        reward_tokens: sponsor_entry.reward_tokens,
        challenge_text: sponsor_entry.challenge_text,
        challenge_time: sponsor_entry.challenge_time,
        system_instruction: sponsor_entry.system_instruction,
        start_text: sponsor_entry.start_text,
        won_text: sponsor_entry.won_text,
        lost_text: sponsor_entry.lost_text,
        greeting_text: sponsor_entry.greeting_text,
        end_text: sponsor_entry.end_text,
        rating_threshold: sponsor_entry.rating_threshold,
        initial_funded: sponsor_entry.initial_funded,
    };

    let response_data = ResponseData {
        sponsor: return_sponsor,
        signature: signature.to_string(),
    };

    let response = (StatusCode::CREATED, Json(response_data)).into_response();
    response
}
