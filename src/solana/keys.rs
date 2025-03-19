use solana_client::rpc_client::RpcClient;
use solana_sdk::compute_budget::ComputeBudgetInstruction;
use solana_sdk::pubkey::Pubkey;
use solana_sdk::signature::Keypair;
use solana_sdk::signer::Signer;
use solana_sdk::transaction::Transaction;
use solana_sdk::commitment_config::CommitmentConfig;
use crate::Secrets;


pub fn generate_private_key() -> Keypair {
    log::debug!("Generate new Solana keypair");

    let keypair = Keypair::new();
    return keypair;
}

pub fn _generate_private_key_base58() -> String {
    log::debug!("Generate new Solana keypair");

    let keypair = Keypair::new();
    return keypair.to_base58_string();
}

pub fn _derive_public_key_from_private_key(private_key: &str) -> String {
    log::debug!("Derive Solana public key from private key");

    let keypair = Keypair::from_base58_string(private_key);
    return keypair.pubkey().to_string();
}

pub async fn get_or_create_ata(
    payer: &Keypair,
    wallet_address: &Pubkey,
    token_mint_address: &Pubkey,
    token_program_id: &Pubkey,
    secrets: &Secrets,
) -> Result<Pubkey, Box<dyn std::error::Error>> {

    let commitment_config = CommitmentConfig::confirmed();
    let rpc_client = RpcClient::new_with_commitment(&secrets.rpc_url, commitment_config);

    // Check if the associated token account already exists
    let ata_address = spl_associated_token_account::get_associated_token_address(
        &wallet_address,
        &token_mint_address,
    );

    if rpc_client.get_account(&ata_address).is_ok() {
        log::debug!("ATA already exists: {}", ata_address);
        return Ok(ata_address);
    }

    // Create the associated token account if it doesn't exist
    let create_ata_ix = spl_associated_token_account::instruction::create_associated_token_account(
        &payer.pubkey(),
        &wallet_address,
        &token_mint_address,
        &token_program_id,
    );

    let modify_compute_units = ComputeBudgetInstruction::set_compute_unit_limit(40000);
    let set_priority_fee = ComputeBudgetInstruction::set_compute_unit_price(1000);

    let latest_blockhash = rpc_client.get_latest_blockhash()?;

    let transaction = Transaction::new_signed_with_payer(
        &[create_ata_ix, modify_compute_units, set_priority_fee],
        Some(&payer.pubkey()),
        &[payer],
        latest_blockhash,
    );

    let signature = rpc_client.send_transaction(&transaction)?;

    println!("signature: {}", signature.to_string());

    // Return the ATA address after confirming the transaction
    Ok(ata_address)
}

// funding_address,
// wallet_address,
// token_mint_address,
// token_program_id,
