use solana_sdk::transaction::Transaction;
use solana_client::rpc_client::RpcClient;
use solana_sdk::commitment_config::CommitmentConfig;
use solana_sdk::signature::Signature;
use crate::Secrets;
use solana_sdk::pubkey::Pubkey;
use std::str::FromStr;


pub async fn verify_payment(
    secrets: &Secrets,
    transaction: Transaction,
) -> Result<Signature, Box<dyn std::error::Error>> {

    // Initialize the RPC client
    let commitment_config = CommitmentConfig::confirmed();
    let rpc_client = RpcClient::new_with_commitment(&secrets.rpc_url, commitment_config);

    let receiver_pubkey: Pubkey = Pubkey::from_str(&secrets.treasury_public_key).expect("Invalid receiver pubkey address");

    // Check if receiver_pubkey is among the signers
    let is_receiver_signer = transaction
        .signatures
        .iter()
        .any(|sig| transaction.message.account_keys.iter().any(|key| key == &receiver_pubkey));

    if !is_receiver_signer {
        return Err("Receiver pubkey is not among the signers".into());
    }

    let signature = rpc_client.send_transaction(&transaction)?;

    return Ok(signature);

}
