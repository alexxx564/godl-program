use anyhow::Result;
use solana_client::nonblocking::rpc_client::RpcClient;
use solana_sdk::{pubkey::Pubkey, signature::Signer};

use crate::{rpc::get_referrer_by_address, transaction::submit_transaction};

/// Initialize a referrer account
pub async fn initialize_referrer_cmd(
    rpc: &RpcClient,
    payer: &solana_sdk::signer::keypair::Keypair,
) -> Result<()> {
    let ix = godl_api::sdk::initialize_referrer(payer.pubkey());
    submit_transaction(rpc, payer, &[ix]).await?;
    Ok(())
}

/// Set the referrer for the user's miner account
pub async fn set_referrer_cmd(
    rpc: &RpcClient,
    payer: &solana_sdk::signer::keypair::Keypair,
    referrer_authority: Option<Pubkey>,
) -> Result<()> {
    let referrer_pda = if let Some(authority) = referrer_authority {
        let (address, _) = godl_api::state::referrer_pda(authority);
        let _ = get_referrer_by_address(rpc, address).await?;
        Some(address)
    } else {
        None
    };
    let ix = godl_api::sdk::set_referrer(payer.pubkey(), referrer_pda);
    submit_transaction(rpc, payer, &[ix]).await?;
    Ok(())
}

/// Claim referral rewards
pub async fn claim_referral_cmd(
    rpc: &RpcClient,
    payer: &solana_sdk::signer::keypair::Keypair,
) -> Result<()> {
    let ix = godl_api::sdk::claim_referral(payer.pubkey());
    submit_transaction(rpc, payer, &[ix]).await?;
    Ok(())
}
