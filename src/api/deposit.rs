use axum::response::IntoResponse;
use axum::Json;
use axum::Extension;
use crate::StatusCode;
use serde::{Serialize, Deserialize};
use crate::secrets::Secrets;
use crate::solana::generate_deposit::generate_deposit;
use base64::{engine::general_purpose, Engine as _};
use bincode;
use crate::database::Database;


#[derive(Serialize, Deserialize)]
pub struct DepositArgs {
    pub sender_public_key: String,
    pub sponsor_public_key: String,
}

pub async fn deposit(
    secrets: Extension<Secrets>,
    Extension(database): Extension<Database>,
    Json(payment_args): Json<DepositArgs>
) -> impl IntoResponse {

    let sender_public_key = payment_args.sender_public_key;
    let sponsor_public_key = payment_args.sponsor_public_key;

    let deposit_transaction = generate_deposit(
        &secrets,
        &database,
        sender_public_key,
        sponsor_public_key
    ).await
    .unwrap();

    let serialized_transaction = bincode::serialize(&deposit_transaction).expect("Failed to serialize transaction");
    let encoded_transaction = general_purpose::STANDARD.encode(serialized_transaction);

    let response = (
        StatusCode::OK, 
        Json(encoded_transaction)
    ).into_response();

    response

}