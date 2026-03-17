use godl_api::prelude::*;
use steel::*;

/// Initializes the SolMotherlode PDA.
pub fn process_initialize_sol_motherlode(
    accounts: &[AccountInfo<'_>],
    _data: &[u8],
) -> ProgramResult {
    // Load accounts.
    let [signer_info, sol_motherlode_info, system_program] = accounts else {
        return Err(ProgramError::NotEnoughAccountKeys);
    };
    signer_info.is_signer()?;
    signer_info.has_address(&DEPLOYER_ADDRESS)?;
    system_program.is_program(&system_program::ID)?;

    sol_motherlode_info
        .is_empty()?
        .is_writable()?
        .has_seeds(&[SOL_MOTHERLODE], &godl_api::ID)?;

    // Create account.
    create_program_account::<SolMotherlode>(
        sol_motherlode_info,
        system_program,
        signer_info,
        &godl_api::ID,
        &[SOL_MOTHERLODE],
    )?;

    // Initialize sol motherlode.
    let sol_motherlode = sol_motherlode_info.as_account_mut::<SolMotherlode>(&godl_api::ID)?;
    sol_motherlode.amount = 0;

    Ok(())
}
