use godl_api::prelude::*;
use steel::*;

/// Sets the motherlode denominator.
pub fn process_set_motherlode_denominator(
    accounts: &[AccountInfo<'_>],
    data: &[u8],
) -> ProgramResult {
    // Parse data.
    let args = SetMotherlodeDenominator::try_from_bytes(data)?;
    let new_motherlode_denominator = u64::from_le_bytes(args.motherlode_denominator);

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

    // Set motherlode denominator.
    config.motherlode_denominator = new_motherlode_denominator;

    Ok(())
}
