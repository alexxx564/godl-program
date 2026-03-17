use godl_api::prelude::*;
use steel::*;

/// Initializes the referrer PDA and its GODL token account.
pub fn process_initialize_referrer(accounts: &[AccountInfo<'_>], _data: &[u8]) -> ProgramResult {
    // Load accounts.
    let [authority_info, referrer_info, referrer_tokens_info, mint_info, system_program, token_program, associated_token_program] =
        accounts
    else {
        return Err(ProgramError::NotEnoughAccountKeys);
    };
    authority_info.is_signer()?;
    referrer_info.is_writable()?;
    referrer_tokens_info.is_writable()?;
    referrer_info
        .is_empty()?
        .has_seeds(&[REFERRER, &authority_info.key.to_bytes()], &godl_api::ID)?;
    mint_info.has_address(&MINT_ADDRESS)?.as_mint()?;
    system_program.is_program(&system_program::ID)?;
    token_program.is_program(&spl_token::ID)?;
    associated_token_program.is_program(&spl_associated_token_account::ID)?;

    // Create referrer PDA account.
    create_program_account::<Referrer>(
        referrer_info,
        system_program,
        authority_info,
        &godl_api::ID,
        &[REFERRER, &authority_info.key.to_bytes()],
    )?;
    let referrer = referrer_info.as_account_mut::<Referrer>(&godl_api::ID)?;
    referrer.authority = *authority_info.key;
    referrer.rewards_sol = 0;
    referrer.rewards_godl = 0;
    referrer.cumulative_rewards_sol = 0;
    referrer.cumulative_rewards_godl = 0;
    referrer.referrer_count = 0;

    // Ensure the referrer has an associated token account.
    if referrer_tokens_info.data_is_empty() {
        create_associated_token_account(
            authority_info,
            referrer_info,
            referrer_tokens_info,
            mint_info,
            system_program,
            token_program,
            associated_token_program,
        )?;
    } else {
        referrer_tokens_info.as_associated_token_account(referrer_info.key, mint_info.key)?;
    }

    Ok(())
}
