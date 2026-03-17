use anyhow::Result;
use jup_swap::{
    quote::QuoteRequest,
    swap::SwapRequest,
    transaction_config::{DynamicSlippageSettings, TransactionConfig},
    JupiterSwapApiClient,
};
use solana_client::nonblocking::rpc_client::RpcClient;
use solana_sdk::{native_token::LAMPORTS_PER_SOL, pubkey, pubkey::Pubkey, signature::Signer};
use spl_associated_token_account::get_associated_token_address;
use tokio::time::{sleep, Duration};

use crate::transaction::{
    get_address_lookup_table_accounts, submit_transaction,
    submit_transaction_with_address_lookup_tables,
};

const CHEST_AMOUNT_BPS: u64 = 2500; // 25%
const ADMIN_AMOUNT_BPS: u64 = 500; // 5%

const GODL_LUT: Pubkey = pubkey!("CWD8mcpi4QFPZfhgG46cmcytShfEMXWF2gHDjVKaYFce");

/// Manually bury a fixed amount of GODL from the treasury.
pub async fn manual_bury(
    rpc: &RpcClient,
    payer: &solana_sdk::signer::keypair::Keypair,
    amount_godl: f64,
    no_burn: bool,
) -> Result<()> {
    // Convert GODL (decimal) to base units.
    let amount_units = (amount_godl * godl_api::consts::ONE_GODL as f64) as u64;

    if amount_units == 0 {
        println!("Amount too small after conversion; nothing to bury.");
        return Ok(());
    }

    let ix = godl_api::sdk::bury_tokens(payer.pubkey(), amount_units, no_burn);
    submit_transaction(rpc, payer, &[ix]).await?;

    println!(
        "Submitted manual-bury for {} GODL ({} base units)",
        amount_godl, amount_units
    );

    Ok(())
}

/// Bury (swap SOL for GODL via Jupiter)
pub async fn bury(
    rpc: &RpcClient,
    payer: &solana_sdk::signer::keypair::Keypair,
    amount_sol: f64,
    api_base_url: Option<String>,
    no_burn: bool,
) -> Result<()> {
    // Convert SOL to lamports.
    let amount = (amount_sol * LAMPORTS_PER_SOL as f64) as u64;

    let chest_amount = (amount * CHEST_AMOUNT_BPS / 10000) as u64;
    let admin_amount = (amount * ADMIN_AMOUNT_BPS / 10000) as u64;
    let bury_amount = amount - chest_amount - admin_amount;

    // Build quote request.
    const INPUT_MINT: Pubkey = pubkey!("So11111111111111111111111111111111111111112");
    const OUTPUT_MINT: Pubkey = pubkey!("GodL6KZ9uuUoQwELggtVzQkKmU1LfqmDokPibPeDKkhF");
    let api_base_url = api_base_url.unwrap_or_else(|| "https://lite-api.jup.ag/swap/v1".into());
    let jupiter_swap_api_client = JupiterSwapApiClient::new(api_base_url);
    let quote_request = QuoteRequest {
        amount: bury_amount,
        input_mint: INPUT_MINT,
        output_mint: OUTPUT_MINT,
        max_accounts: Some(55),
        ..QuoteRequest::default()
    };

    // GET /quote
    let quote_response = match jupiter_swap_api_client.quote(&quote_request).await {
        Ok(quote_response) => quote_response,
        Err(e) => {
            println!("quote failed: {e:#?}");
            return Err(anyhow::anyhow!("quote failed: {e:#?}"));
        }
    };

    // GET /swap/instructions
    let treasury_address = godl_api::state::treasury_pda().0;
    let response = jupiter_swap_api_client
        .swap_instructions(&SwapRequest {
            user_public_key: treasury_address,
            quote_response,
            config: TransactionConfig {
                skip_user_accounts_rpc_calls: false,
                wrap_and_unwrap_sol: false,
                dynamic_compute_unit_limit: true,
                dynamic_slippage: Some(DynamicSlippageSettings {
                    min_bps: Some(50),
                    max_bps: Some(1000),
                }),
                ..TransactionConfig::default()
            },
        })
        .await
        .unwrap();

    let mut lut_addresses = vec![GODL_LUT];
    lut_addresses.extend(response.address_lookup_table_addresses);

    let address_lookup_table_accounts = get_address_lookup_table_accounts(rpc, lut_addresses)
        .await
        .unwrap();

    // Build transaction.
    let pre_bury_ix = godl_api::sdk::pre_bury(payer.pubkey(), bury_amount, chest_amount, admin_amount);
    let bury_ix = godl_api::sdk::bury(
        payer.pubkey(),
        &response.swap_instruction.accounts,
        &response.swap_instruction.data,
        no_burn,
    );
    submit_transaction_with_address_lookup_tables(
        rpc,
        payer,
        &[pre_bury_ix, bury_ix],
        address_lookup_table_accounts,
    )
    .await?;

    Ok(())
}

/// Bury-listen (monitor treasury balance and auto-bury)
pub async fn bury_listen(
    rpc: &RpcClient,
    payer: &solana_sdk::signer::keypair::Keypair,
    amount_sol: f64,
    api_base_url: Option<String>,
    no_burn: bool,
) -> Result<()> {
    // Convert SOL to lamports.
    let amount = (amount_sol * LAMPORTS_PER_SOL as f64) as u64;
    let threshold = amount + (LAMPORTS_PER_SOL / 10); // amount + 0.1 SOL
    let treasury_address = godl_api::state::treasury_pda().0;

    println!("Starting bury-listen...");
    println!("Treasury address: {}", treasury_address);
    println!("Amount to bury: {} SOL", amount_sol);
    println!(
        "Threshold: {} SOL",
        threshold as f64 / LAMPORTS_PER_SOL as f64
    );
    println!("Checking treasury balance every 60 seconds...\n");

    loop {
        // Check treasury balance
        match rpc.get_balance(&treasury_address).await {
            Ok(balance) => {
                let balance_sol = balance as f64 / LAMPORTS_PER_SOL as f64;
                let threshold_sol = threshold as f64 / LAMPORTS_PER_SOL as f64;

                println!(
                    "[{}] Treasury balance: {} SOL (threshold: {} SOL)",
                    chrono::Local::now().format("%Y-%m-%d %H:%M:%S"),
                    balance_sol,
                    threshold_sol
                );

                if balance >= threshold {
                    println!("Balance exceeds threshold! Sending bury transaction...");

                    match execute_bury(rpc, payer, amount, api_base_url.clone(), no_burn).await {
                        Ok(()) => {
                            println!("✓ Bury transaction successful!\n");
                        }
                        Err(e) => {
                            println!("✗ Bury transaction failed: {:#?}\n", e);
                        }
                    }
                }
            }
            Err(e) => {
                println!("Failed to get treasury balance: {:#?}", e);
            }
        }

        // Wait 1 minute before next check
        sleep(Duration::from_secs(30)).await;
    }
}

/// Helper function to execute the bury transaction (extracted from bury function)
async fn execute_bury(
    rpc: &RpcClient,
    payer: &solana_sdk::signer::keypair::Keypair,
    amount: u64,
    api_base_url: Option<String>,
    no_burn: bool,
) -> Result<()> {
    let chest_amount = (amount * CHEST_AMOUNT_BPS / 10000) as u64;
    let admin_amount = (amount * ADMIN_AMOUNT_BPS / 10000) as u64;
    let bury_amount = amount - chest_amount - admin_amount;

    // Build quote request.
    const INPUT_MINT: Pubkey = pubkey!("So11111111111111111111111111111111111111112");
    const OUTPUT_MINT: Pubkey = pubkey!("GodL6KZ9uuUoQwELggtVzQkKmU1LfqmDokPibPeDKkhF");
    let api_base_url = api_base_url.unwrap_or_else(|| "https://lite-api.jup.ag/swap/v1".into());
    let jupiter_swap_api_client = JupiterSwapApiClient::new(api_base_url);
    let quote_request = QuoteRequest {
        amount: bury_amount,
        input_mint: INPUT_MINT,
        output_mint: OUTPUT_MINT,
        max_accounts: Some(55),
        ..QuoteRequest::default()
    };

    // GET /quote
    let quote_response = match jupiter_swap_api_client.quote(&quote_request).await {
        Ok(quote_response) => quote_response,
        Err(e) => {
            return Err(anyhow::anyhow!("quote failed: {e:#?}"));
        }
    };

    // GET /swap/instructions
    let treasury_address = godl_api::state::treasury_pda().0;
    let response = jupiter_swap_api_client
        .swap_instructions(&SwapRequest {
            user_public_key: treasury_address,
            quote_response,
            config: TransactionConfig {
                skip_user_accounts_rpc_calls: false,
                wrap_and_unwrap_sol: false,
                dynamic_compute_unit_limit: true,
                dynamic_slippage: Some(DynamicSlippageSettings {
                    min_bps: Some(50),
                    max_bps: Some(1000),
                }),
                ..TransactionConfig::default()
            },
        })
        .await?;

    let mut lut_addresses = vec![GODL_LUT];
    lut_addresses.extend(response.address_lookup_table_addresses);

    let address_lookup_table_accounts =
        get_address_lookup_table_accounts(rpc, lut_addresses).await?;

    // Build transaction.
    let pre_bury_ix = godl_api::sdk::pre_bury(payer.pubkey(), bury_amount, chest_amount, admin_amount);
    let bury_ix = godl_api::sdk::bury(
        payer.pubkey(),
        &response.swap_instruction.accounts,
        &response.swap_instruction.data,
        no_burn,
    );
    submit_transaction_with_address_lookup_tables(
        rpc,
        payer,
        &[pre_bury_ix, bury_ix],
        address_lookup_table_accounts,
    )
    .await?;

    Ok(())
}

/// Manually bury when the treasury GODL balance exceeds a threshold.
pub async fn manual_bury_listen(
    rpc: &RpcClient,
    payer: &solana_sdk::signer::keypair::Keypair,
    amount_godl: f64,
    no_burn: bool,
) -> Result<()> {
    // Convert GODL (decimal) to base units for the threshold.
    let threshold_units = (amount_godl * godl_api::consts::ONE_GODL as f64) as u64;

    if threshold_units == 0 {
        println!("Threshold too small after conversion; nothing to do.");
        return Ok(());
    }

    let treasury_address = godl_api::state::treasury_pda().0;
    let treasury_godl_address =
        get_associated_token_address(&treasury_address, &godl_api::consts::MINT_ADDRESS);

    println!("Starting manual-bury-listen...");
    println!("Treasury address: {}", treasury_address);
    println!("Treasury GODL ATA: {}", treasury_godl_address);
    println!("Threshold: {} GODL", amount_godl);
    println!("Checking treasury GODL balance every 60 seconds...\n");

    loop {
        match rpc.get_token_account_balance(&treasury_godl_address).await {
            Ok(balance) => {
                let balance_units: u64 = balance.amount.parse().unwrap_or(0);
                let balance_godl = balance_units as f64 / godl_api::consts::ONE_GODL as f64;

                println!(
                    "[{}] Treasury GODL balance: {} GODL ({} units, threshold: {} units)",
                    chrono::Local::now().format("%Y-%m-%d %H:%M:%S"),
                    balance_godl,
                    balance_units,
                    threshold_units,
                );

                if balance_units >= threshold_units {
                    println!("Balance exceeds threshold! Sending manual-bury transaction...");

                    match execute_manual_bury(rpc, payer, threshold_units, no_burn).await {
                        Ok(()) => println!("✓ Manual-bury transaction successful!\n"),
                        Err(e) => println!("✗ Manual-bury transaction failed: {:#?}\n", e),
                    }
                }
            }
            Err(e) => {
                println!("Failed to get treasury GODL balance: {:#?}", e);
            }
        }

        // Wait 1 minute before next check
        sleep(Duration::from_secs(60)).await;
    }
}

/// Helper function to execute the manual-bury transaction.
async fn execute_manual_bury(
    rpc: &RpcClient,
    payer: &solana_sdk::signer::keypair::Keypair,
    amount_units: u64,
    no_burn: bool,
) -> Result<()> {
    if amount_units == 0 {
        return Ok(());
    }

    let ix = godl_api::sdk::bury_tokens(payer.pubkey(), amount_units, no_burn);
    submit_transaction(rpc, payer, &[ix]).await?;

    Ok(())
}
