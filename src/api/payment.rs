use axum::response::IntoResponse;
use axum::Json;
use axum::Extension;
use crate::StatusCode;
use serde::{Serialize, Deserialize};
use crate::secrets::Secrets;
use crate::solana::generate_payment::generate_payment;
use base64::{engine::general_purpose, Engine as _};
use bincode;


#[derive(Serialize, Deserialize)]
pub struct PaymentArgs {
    pub sender: String,
}

#[axum::debug_handler]
pub async fn payment(
    secrets: Extension<Secrets>,
    Json(payment_args): Json<PaymentArgs>
) -> impl IntoResponse {

    let sender = payment_args.sender;
    let amount = 1000000000;

    let transaction = generate_payment(
        &secrets,
        sender, 
        amount
    ).await
    .unwrap();

    let serialized_transaction = bincode::serialize(&transaction).expect("Failed to serialize transaction");
    let encoded_transaction = general_purpose::STANDARD.encode(serialized_transaction);

    let response = (
        StatusCode::OK, 
        Json(encoded_transaction)
    ).into_response();

    response

}