use godl_api::prelude::*;
use steel::*;

/// Allows the stake authority to set or update the delegated executor for an existing lock.
pub fn process_set_stake_executor_v2(accounts: &[AccountInfo<'_>], data: &[u8]) -> ProgramResult {
    // Parse data.
    let args = SetStakeExecutorV2::try_from_bytes(data)?;
    let id = u64::from_le_bytes(args.id);
    let executor = Pubkey::new_from_array(args.executor);

    // Load accounts.
    let [signer_info, stake_info] = accounts else {
        return Err(ProgramError::NotEnoughAccountKeys);
    };
    signer_info.is_signer()?;
    stake_info.is_writable()?;

    // Validate and mutate stake.
    let stake = stake_info
        .as_account_mut::<StakeV2>(&godl_api::ID)?
        .assert_mut(|s| s.id == id)?
        .assert_mut(|s| s.authority == *signer_info.key)?;

    stake.executor = executor;

    Ok(())
}
