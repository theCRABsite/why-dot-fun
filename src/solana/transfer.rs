use solana_sdk::{
    signature::{Keypair, Signer},
    transaction::Transaction,
    pubkey::Pubkey
};
use solana_client::rpc_client::RpcClient;
use solana_sdk::commitment_config::CommitmentConfig;
use spl_token::instruction::transfer;
use std::str::FromStr;
use crate::solana::keys::get_or_create_ata;
use solana_sdk::compute_budget::ComputeBudgetInstruction;
use solana_sdk::signature::Signature;
use crate::Secrets;


pub async fn transfer_solana_token(
    secrets: &Secrets,
    sender_private_key: String, 
    receiver_pubkey: Pubkey, 
    token_mint: String,
    amount: u64
) -> Result<Signature, Box<dyn std::error::Error>> {
    log::debug!("Transfer Solana token");

    // Initialize the RPC client
    let commitment_config = CommitmentConfig::confirmed();
    let rpc_client = RpcClient::new_with_commitment(&secrets.rpc_url, commitment_config);

    // Initialize accounts needed for the transfer
    let sender_keypair: Keypair = Keypair::from_base58_string(&sender_private_key);

    let token_mint: Pubkey = Pubkey::from_str(&token_mint).expect("Invalid token mint address");

    let account_info = rpc_client.get_account(&token_mint).expect("Failed to fetch account info for token mint");
    let token_program_id = account_info.owner;

    let sender_token_account = get_or_create_ata(
        &sender_keypair,
        &sender_keypair.pubkey(), 
        &token_mint,
        &token_program_id,
        &secrets
    ).await.expect("Failed to get or create sender token account");

    let receiver_token_account = get_or_create_ata(
        &sender_keypair,
        &receiver_pubkey, 
        &token_mint,
        &token_program_id,
        &secrets
    ).await.expect("Failed to get or create receiver token account");


    // let amount_to_transfer: u64 = amount * 1000000000;

    // Create the transfer instruction
    let transfer_ix = transfer(
        &token_program_id,
        &sender_token_account,
        &receiver_token_account,
        &sender_keypair.pubkey(),
        &[&sender_keypair.pubkey()],
        amount
        // amount_to_transfer,
    )
    .expect("Failed to create transfer instruction");
    


    let modify_compute_units = ComputeBudgetInstruction::set_compute_unit_limit(40000);
    let set_priority_fee = ComputeBudgetInstruction::set_compute_unit_price(1000);

    let latest_blockhash = rpc_client.get_latest_blockhash()?;
    
    let transaction = Transaction::new_signed_with_payer(
        &[transfer_ix, modify_compute_units, set_priority_fee], 
        Some(&sender_keypair.pubkey()),
        &[sender_keypair],
        latest_blockhash
    );
    
    let signature = rpc_client.send_and_confirm_transaction(&transaction).expect("Failed to send and confirm transaction");


    Ok(signature)
}
