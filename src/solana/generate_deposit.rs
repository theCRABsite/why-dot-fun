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
use solana_sdk::message::Message;
use crate::secrets::Secrets;
use crate::database::Database;
use spl_token::instruction::transfer;
use crate::solana::keys::get_or_create_ata;


pub async fn generate_deposit(
    secrets: &Secrets,
    database: &Database,
    sender_pubkey: String, 
    sponsor_pubkey: String,
) -> Result<Transaction, Box<dyn std::error::Error>> {
    log::debug!("Generate deposit transaction");


    let sponsor = database
        .get_sponsor_by_public_key(sponsor_pubkey)
        .await
        .expect("Sponsor not found");

    // Initialize the RPC client
    let commitment_config = CommitmentConfig::confirmed();
    let rpc_client = RpcClient::new_with_commitment(&secrets.rpc_url, commitment_config);

    let sender_pubkey: Pubkey = Pubkey::from_str(&sender_pubkey).expect("Invalid sender pubkey address");
    let receiver_pubkey: Pubkey = Pubkey::from_str(&sponsor.public_key).expect("Invalid receiver pubkey address");

    let signer_private_key = &secrets.treasury_private_key;
    let whydotfun_treasury_keypair = Keypair::from_base58_string(signer_private_key);


    let token_mint: Pubkey = Pubkey::from_str(&sponsor.token_mint).expect("Invalid token mint address");
    let account_info = rpc_client.get_account(&token_mint).expect("Failed to fetch account info for token mint");
    let token_program_id = account_info.owner;


    let sender_token_account = get_or_create_ata(
        &whydotfun_treasury_keypair,
        &sender_pubkey, 
        &token_mint,
        &token_program_id,
        &secrets
    ).await.expect("Failed to get or create sender token account");

    let receiver_token_account = get_or_create_ata(
        &whydotfun_treasury_keypair,
        &receiver_pubkey, 
        &token_mint,
        &token_program_id,
        &secrets
    ).await.expect("Failed to get or create sender token account");


    // Create the transfer instruction
    let transfer_ix = transfer(
        &token_program_id,
        &sender_token_account,
        &receiver_token_account,
        &sender_pubkey,
        &[&sender_pubkey],
        sponsor.original_tokens as u64,
    ).expect("Failed to create transfer instruction");



    let modify_compute_units = ComputeBudgetInstruction::set_compute_unit_limit(40000);
    let set_priority_fee = ComputeBudgetInstruction::set_compute_unit_price(1000);

    let latest_blockhash = rpc_client.get_latest_blockhash()?;
    
    // Create a message from the instructions
    let message = Message::new(
        &[
            transfer_ix, 
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
