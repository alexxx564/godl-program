use anyhow::Result;
use godl_api::prelude::*;
use solana_client::nonblocking::rpc_client::RpcClient;
use solana_sdk::signature::Signer;
use spl_associated_token_account::get_associated_token_address;
use steel::{AccountMeta, Pubkey};

use crate::transaction::submit_transaction;

/// Create a lookup table with common addresses
pub async fn lut(rpc: &RpcClient, payer: &solana_sdk::signer::keypair::Keypair) -> Result<()> {
    let recent_slot = rpc.get_slot().await? - 4;
    let (ix, lut_address) = solana_address_lookup_table_interface::instruction::create_lookup_table(
        payer.pubkey(),
        payer.pubkey(),
        recent_slot,
    );
    let board_address = godl_api::state::board_pda().0;
    let config_address = godl_api::state::config_pda().0;
    let treasury_address = godl_api::state::treasury_pda().0;
    let treasury_tokens_address = godl_api::state::treasury_tokens_address(treasury_address);
    let treasury_sol_address = get_associated_token_address(&treasury_address, &SOL_MINT);
    let mint_address = MINT_ADDRESS;
    let godl_program_address = godl_api::ID;
    let ex_ix = solana_address_lookup_table_interface::instruction::extend_lookup_table(
        lut_address,
        payer.pubkey(),
        Some(payer.pubkey()),
        vec![
            board_address,
            config_address,
            treasury_address,
            treasury_tokens_address,
            treasury_sol_address,
            mint_address,
            godl_program_address,
        ],
    );
    let ix_1 = steel::Instruction {
        program_id: ix.program_id,
        accounts: ix
            .accounts
            .iter()
            .map(|a| AccountMeta::new(a.pubkey, a.is_signer))
            .collect(),
        data: ix.data,
    };
    let ix_2 = steel::Instruction {
        program_id: ex_ix.program_id,
        accounts: ex_ix
            .accounts
            .iter()
            .map(|a| AccountMeta::new(a.pubkey, a.is_signer))
            .collect(),
        data: ex_ix.data,
    };
    submit_transaction(rpc, payer, &[ix_1, ix_2]).await?;
    println!("LUT address: {}", lut_address);
    Ok(())
}

/// Create an associated token account (debug utility)
pub async fn ata(
    rpc: &RpcClient,
    payer: &solana_sdk::signer::keypair::Keypair,
    user: Pubkey,
) -> Result<()> {
    let ata = get_associated_token_address(&user, &MINT_ADDRESS);
    let ix = spl_associated_token_account::instruction::create_associated_token_account(
        &payer.pubkey(),
        &user,
        &MINT_ADDRESS,
        &spl_token::ID,
    );
    submit_transaction(rpc, payer, &[ix]).await?;
    let account = rpc.get_account(&ata).await?;
    println!("ATA: {}", ata);
    println!("Account: {:?}", account);
    Ok(())
}
