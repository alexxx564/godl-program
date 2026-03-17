use anyhow::Result;
use godl_api::prelude::*;
use solana_client::nonblocking::rpc_client::RpcClient;
use solana_sdk::{pubkey::Pubkey, signature::Signer};

use crate::{
    display::*,
    rpc::{
        get_automations, get_board, get_clock, get_config, get_miner, get_miners_participating,
        get_round, get_stake, get_treasury,
    },
};

/// Log board information
pub async fn log_board(rpc: &RpcClient) -> Result<()> {
    let board = get_board(rpc).await?;
    let clock = get_clock(rpc).await?;
    print_board(board, &clock);
    Ok(())
}

/// Log clock information
pub async fn log_clock(rpc: &RpcClient) -> Result<()> {
    let clock = get_clock(rpc).await?;
    print_clock(&clock);
    Ok(())
}

/// Log config information
pub async fn log_config(rpc: &RpcClient) -> Result<()> {
    let config = get_config(rpc).await?;
    print_config(&config);
    Ok(())
}

/// Log treasury information
pub async fn log_treasury(rpc: &RpcClient) -> Result<()> {
    let treasury_address = godl_api::state::treasury_pda().0;
    let treasury = get_treasury(rpc).await?;
    print_treasury(&treasury, treasury_address);
    Ok(())
}

/// Log round information
pub async fn log_round(rpc: &RpcClient, id: u64) -> Result<()> {
    let round_address = round_pda(id).0;
    let round = get_round(rpc, id).await?;
    print_round(&round, round_address);
    Ok(())
}

/// Log miner information
pub async fn log_miner(
    rpc: &RpcClient,
    payer: &solana_sdk::signer::keypair::Keypair,
    authority: Option<Pubkey>,
) -> Result<()> {
    let authority = authority.unwrap_or_else(|| payer.pubkey());
    let miner_address = godl_api::state::miner_pda(authority).0;
    let miner = get_miner(rpc, authority).await?;
    print_miner(rpc, &miner, authority, miner_address).await;
    Ok(())
}

/// Log stake information
pub async fn log_stake(
    rpc: &RpcClient,
    payer: &solana_sdk::signer::keypair::Keypair,
    authority: Option<Pubkey>,
) -> Result<()> {
    let authority = authority.unwrap_or_else(|| payer.pubkey());
    let staker_address = godl_api::state::stake_pda(authority).0;
    let stake = get_stake(rpc, authority).await?;
    print_stake(&stake, authority, staker_address);
    Ok(())
}

/// Log automations information
pub async fn log_automations(rpc: &RpcClient) -> Result<()> {
    let automations = get_automations(rpc).await?;
    print_automations(&automations);
    Ok(())
}

/// Log participating miners
pub async fn participating_miners(rpc: &RpcClient, round_id: u64) -> Result<()> {
    let miners = get_miners_participating(rpc, round_id).await?;
    print_participating_miners(&miners);
    Ok(())
}

/// Print key addresses (debug utility)
pub async fn keys(rpc: &RpcClient, authority: Pubkey) -> Result<()> {
    let treasury_address = godl_api::state::treasury_pda().0;
    let config_address = godl_api::state::config_pda().0;
    let board_address = godl_api::state::board_pda().0;
    let miner_address = godl_api::state::miner_pda(authority).0;
    let board = get_board(rpc).await?;
    let round = round_pda(board.round_id).0;
    println!("Mint: {}", MINT_ADDRESS);
    println!("Round: {}", round);
    println!("Treasury: {}", treasury_address);
    println!("Config: {}", config_address);
    println!("Board: {}", board_address);
    println!("Miner: {}", miner_address);
    Ok(())
}
