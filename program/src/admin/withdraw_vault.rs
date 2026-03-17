use godl_api::prelude::*;
use solana_program::rent::Rent;
use steel::*;

/// Withdraw SOL from the treasury vault back to the treasury account.
/// This transfers SOL from the treasury PDA to reduce the vault balance.
pub fn process_withdraw_vault(accounts: &[AccountInfo<'_>], data: &[u8]) -> ProgramResult {
    // Parse args.
    let args = WithdrawVault::try_from_bytes(data)?;
    let amount = u64::from_le_bytes(args.amount);

    // Load accounts.
    let [signer_info, config_info, treasury_info, system_program] = accounts else {
        return Err(ProgramError::NotEnoughAccountKeys);
    };
    signer_info.is_signer()?;
    config_info
        .as_account::<Config>(&godl_api::ID)?
        .assert(|c| c.bury_authority == *signer_info.key)?;
    let treasury = treasury_info.as_account_mut::<Treasury>(&godl_api::ID)?;
    system_program.is_program(&system_program::ID)?;

    // Validate amount.
    assert!(amount > 0, "Amount must be greater than zero");
    assert!(
        amount <= treasury.balance,
        "Insufficient vault balance: requested {}, available {}",
        amount,
        treasury.balance
    );

    // Check that after withdrawal, treasury still has minimum rent balance.
    let min_balance = Rent::get()?.minimum_balance(std::mem::size_of::<Treasury>());
    let treasury_lamports = treasury_info.lamports();
    assert!(
        treasury_lamports >= min_balance + amount,
        "Insufficient SOL balance: treasury has {} lamports, needs {} (min rent) + {} (withdrawal)",
        treasury_lamports,
        min_balance,
        amount
    );

    // Transfer SOL from treasury (reducing its lamports).
    // The lamports decrease but we're just withdrawing from the vault tracking.
    // Actually, we need to send it somewhere. Let me send to signer since
    // this is meant to extract SOL from the vault.
    treasury_info.send(amount, signer_info);

    // Update treasury vault balance.
    treasury.balance -= amount;

    Ok(())
}
