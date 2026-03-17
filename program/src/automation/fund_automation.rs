use godl_api::prelude::*;
use steel::*;

/// Funds an existing automation_v2 account so it can run more rounds.
pub fn process_fund_automation(accounts: &[AccountInfo<'_>], data: &[u8]) -> ProgramResult {
    // Parse data.
    let args = FundAutomation::try_from_bytes(data)?;
    let deposit = u64::from_le_bytes(args.deposit);

    // Load accounts.
    // [signer_info, automation_v2_info, system_program]
    let [signer_info, automation_v2_info, system_program] = accounts else {
        return Err(ProgramError::NotEnoughAccountKeys);
    };
    signer_info.is_signer()?;
    automation_v2_info.is_writable()?;
    system_program.is_program(&system_program::ID)?;

    // Load and validate automation_v2 account.
    let automation_v2 = automation_v2_info
        .as_account_mut::<AutomationV2>(&godl_api::ID)?
        .assert_mut(|a| a.authority == *signer_info.key)?;
    automation_v2_info.has_seeds(
        &[AUTOMATION_V2, &automation_v2.authority.to_bytes()],
        &godl_api::ID,
    )?;

    // Update on-chain balance and transfer lamports from the signer.
    automation_v2.balance += deposit;
    automation_v2_info.collect(deposit, signer_info)?;

    Ok(())
}
