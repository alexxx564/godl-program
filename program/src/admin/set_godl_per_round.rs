use godl_api::prelude::*;
use steel::*;

/// Sets the amount of GODL minted per round.
pub fn process_set_godl_per_round(accounts: &[AccountInfo<'_>], data: &[u8]) -> ProgramResult {
    // Parse data.
    let args = SetGodlPerRound::try_from_bytes(data)?;
    let new_godl_per_round = u64::from_le_bytes(args.godl_per_round);

    // Load accounts.
    let [signer_info, config_info, system_program] = accounts else {
        return Err(ProgramError::NotEnoughAccountKeys);
    };
    signer_info.is_signer()?;
    let config = config_info
        .as_account_mut::<Config>(&godl_api::ID)?
        .assert_mut_err(
            |c| c.admin == *signer_info.key,
            GodlError::NotAuthorized.into(),
        )?;
    system_program.is_program(&system_program::ID)?;

    // Sanity check.
    assert!(
        new_godl_per_round <= MAX_GODL_PER_ROUND,
        "GODL per round cannot be greater than MAX_GODL_PER_ROUND"
    );

    // Set amount of GODL minted per round.
    config.godl_per_round = new_godl_per_round;

    Ok(())
}
