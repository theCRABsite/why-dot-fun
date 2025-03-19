use solana_sdk::signer::Signer;
use solana_sdk::{
    signature::Keypair,
    transaction::Transaction,
    pubkey::Pubkey
};
use solana_client::rpc_client::RpcClient;
use solana_sdk::commitment_config::CommitmentConfig;
use std::str::FromStr;
use solana_sdk::compute_budget::ComputeBudgetInstruction;
use solana_sdk::system_instruction;
use solana_sdk::message::Message;
use crate::secrets::Secrets;


pub async fn generate_payment(
    secrets: &Secrets,
    sender_pubkey: String, 
    amount: u64
) -> Result<Transaction, Box<dyn std::error::Error>> {
    log::debug!("Generate payment transaction");

    // Initialize the RPC client
    let commitment_config = CommitmentConfig::confirmed();
    let rpc_client = RpcClient::new_with_commitment(&secrets.rpc_url, commitment_config);

    let sender_pubkey: Pubkey = Pubkey::from_str(&sender_pubkey).expect("Invalid sender pubkey address");
    let receiver_pubkey: Pubkey = Pubkey::from_str(&secrets.treasury_public_key).expect("Invalid receiver pubkey address");

    let signer_private_key = &secrets.treasury_private_key;

    let whydotfun_treasury_keypair = Keypair::from_base58_string(signer_private_key);


    let transfer_sol_ix = system_instruction::transfer(
        &sender_pubkey,
        &receiver_pubkey,
        amount,
    );

    let modify_compute_units = ComputeBudgetInstruction::set_compute_unit_limit(40000);
    let set_priority_fee = ComputeBudgetInstruction::set_compute_unit_price(1000);

    let latest_blockhash = rpc_client.get_latest_blockhash()?;
    
    // Create a message from the instructions
    let message = Message::new(
        &[
            transfer_sol_ix, 
            modify_compute_units, 
            set_priority_fee
        ],
        Some(&whydotfun_treasury_keypair.pubkey()),
    );

    // Create a partially signed transaction
    let mut transaction = Transaction::new_unsigned(message);

    // Sign the transaction with the sender's keypair
    transaction.partial_sign(&[&whydotfun_treasury_keypair], latest_blockhash);

    Ok(transaction)
}
