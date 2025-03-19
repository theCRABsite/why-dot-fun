use solana_sdk::transaction::Transaction;
use solana_client::rpc_client::RpcClient;
use solana_sdk::commitment_config::CommitmentConfig;
use solana_sdk::signature::Signature;
use crate::Secrets;
use solana_sdk::pubkey::Pubkey;
use std::str::FromStr;


pub async fn verify_deposit(
    secrets: &Secrets,
    _sponsor_public_key: String,
    transaction: Transaction,
) -> Result<Signature, Box<dyn std::error::Error>> {

    // Initialize the RPC client
    let commitment_config = CommitmentConfig::confirmed();
    let rpc_client = RpcClient::new_with_commitment(&secrets.rpc_url, commitment_config);

    // let is_sponsor_in_account_list = transaction
    //     .message
    //     .account_keys
    //     .iter()
    //     .any(|key| key.to_string() == sponsor_public_key);

    // if !is_sponsor_in_account_list {
    //     return Err("sponsor public key is not among the account keys".into());
    // }


    let whydotfun_treasury: Pubkey = Pubkey::from_str(&secrets.treasury_public_key).expect("Invalid receiver pubkey address");

    // Check if receiver_pubkey is among the signers
    let is_whydotfun_treasury_signer = transaction
        .message
        .account_keys
        .iter()
        .enumerate()
        .any(|(i, key)| key == &whydotfun_treasury && transaction.signatures[i] != Signature::default());

    if !is_whydotfun_treasury_signer {
        return Err("Whydotfun treasury pubkey is not among the signers".into());
    }

    let signature = rpc_client.send_transaction(&transaction)?;

    Ok(signature)
}
