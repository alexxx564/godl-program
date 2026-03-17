use std::io::{stdout, Write};

use anyhow::Result;
use godl_api::consts::TOKEN_DECIMALS;
use solana_client::nonblocking::rpc_client::RpcClient;
use solana_sdk::{native_token::lamports_to_sol, pubkey::Pubkey};
use spl_token::amount_to_ui_amount;
use tokio::io::{self as tokio_io, AsyncBufReadExt, BufReader};

use crate::rpc::{get_miner, get_stake};

/// Interactive explorer for miner and stake rewards
pub async fn explore(rpc: &RpcClient) -> Result<()> {
    println!("Interactive explorer — enter a wallet authority (blank or 'exit' to quit)");
    let stdin = BufReader::new(tokio_io::stdin());
    let mut lines = stdin.lines();

    while let Some(line) = {
        print!("Authority: ");
        let _ = stdout().flush();
        lines.next_line().await?
    } {
        let trimmed = line.trim();
        if trimmed.is_empty() || trimmed.eq_ignore_ascii_case("exit") {
            break;
        }

        match trimmed.parse::<Pubkey>() {
            Ok(authority) => {
                print_rewards(rpc, authority).await;
            }
            Err(err) => {
                println!("Invalid pubkey '{trimmed}': {err}");
            }
        }
    }

    println!("Exiting explore mode");
    Ok(())
}

async fn print_rewards(rpc: &RpcClient, authority: Pubkey) {
    match get_miner(rpc, authority).await {
        Ok(miner) => {
            println!("Miner rewards for {authority}:");
            println!(
                "  rewards_godl: {} GODL",
                amount_to_ui_amount(miner.rewards_godl, TOKEN_DECIMALS)
            );
            println!("  rewards_sol: {} SOL", lamports_to_sol(miner.rewards_sol));
        }
        Err(err) => {
            println!("  Unable to load miner rewards: {err}");
        }
    }

    match get_stake(rpc, authority).await {
        Ok(stake) => {
            println!("Stake info:");
            println!(
                "  balance: {} GODL",
                amount_to_ui_amount(stake.balance, TOKEN_DECIMALS)
            );
            println!(
                "  rewards: {} GODL",
                amount_to_ui_amount(stake.rewards, TOKEN_DECIMALS)
            );
        }
        Err(err) => {
            println!("  Unable to load stake info: {err}");
        }
    }

    println!();
}
