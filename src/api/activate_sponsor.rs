use axum::response::IntoResponse;
use axum::Json;
use axum::Extension;
use crate::Database;
use crate::StatusCode;
use serde::{Serialize, Deserialize};
use crate::secrets::Secrets;
use base64::{engine::general_purpose, Engine as _};
use bincode;
use solana_sdk::transaction::Transaction;
use crate::solana::verify_deposit::verify_deposit;
use crate::api::ResponseData;
use crate::api::launchpad::ReturnSponsor;


#[derive(Deserialize, Serialize)]
pub struct ActivateSponsorArgs {
    pub sponsor_public_key: String,
    pub transaction: String,
}

pub async fn activate_sponsor(
    secrets: Extension<Secrets>,
    Extension(database): Extension<Database>,
    Json(sponsor_args): Json<ActivateSponsorArgs>,
) -> impl IntoResponse {

    // Decode the base64-encoded transaction
    let decoded_transaction = general_purpose::STANDARD
        .decode(&sponsor_args.transaction)
        .expect("Failed to decode transaction");

    // Deserialize the transaction
    let transaction: Transaction = bincode::deserialize(&decoded_transaction)
        .expect("Failed to deserialize transaction");


    let signature = verify_deposit(
        &secrets,
        sponsor_args.sponsor_public_key.clone(),
        transaction
    ).await
    .unwrap();  


    let sponsor = database.get_sponsor_by_public_key(sponsor_args.sponsor_public_key.clone()).await.unwrap();

    if sponsor.initial_funded == true {
        return (StatusCode::BAD_REQUEST, "Initial was already funded").into_response();
    }

    database.update_sponsor_to_active(sponsor_args.sponsor_public_key.clone()).await.unwrap();


    let return_sponsor = ReturnSponsor {
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
        initial_funded: true,
    };

    let response_data = ResponseData {
        sponsor: return_sponsor,
        signature: signature.to_string(),
    };

    let response = (StatusCode::CREATED, Json(response_data)).into_response();
    response

}