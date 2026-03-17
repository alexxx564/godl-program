use godl_api::prelude::*;
use solana_program::{log::sol_log, native_token::lamports_to_sol};
use spl_token::amount_to_ui_amount;
use steel::*;

/// Pays out pending referral rewards to the referrer authority.
pub fn process_claim_referral(accounts: &[AccountInfo<'_>], _data: &[u8]) -> ProgramResult {
    // Load accounts.
    let [authority_info, referrer_info, referrer_tokens_info, authority_tokens_info, mint_info, system_program, token_program, associated_token_program] =
        accounts
    else {
        return Err(ProgramError::NotEnoughAccountKeys);
    };
    authority_info.is_signer()?;
    referrer_info.is_writable()?;
    referrer_tokens_info.is_writable()?;
    authority_tokens_info.is_writable()?;
    mint_info.has_address(&MINT_ADDRESS)?.as_mint()?;
    system_program.is_program(&system_program::ID)?;
    token_program.is_program(&spl_token::ID)?;
    associated_token_program.is_program(&spl_associated_token_account::ID)?;

    // Load referrer account.
    referrer_info.has_seeds(&[REFERRER, &authority_info.key.to_bytes()], &godl_api::ID)?;
    let referrer = referrer_info
        .as_account_mut::<Referrer>(&godl_api::ID)?
        .assert_mut_err(
            |r| r.authority == *authority_info.key,
            GodlError::NotAuthorized.into(),
        )?;

    // Ensure PDA token account exists.
    referrer_tokens_info.as_associated_token_account(referrer_info.key, mint_info.key)?;

    // Ensure authority token account exists or create it.
    if authority_tokens_info.data_is_empty() {
        create_associated_token_account(
            authority_info,
            authority_info,
            authority_tokens_info,
            mint_info,
            system_program,
            token_program,
            associated_token_program,
        )?;
    } else {
        authority_tokens_info.as_associated_token_account(authority_info.key, mint_info.key)?;
    }

    let godl_amount = referrer.rewards_godl;
    let sol_amount = referrer.rewards_sol;

    if godl_amount == 0 && sol_amount == 0 {
        sol_log("No referral rewards to claim");
        return Ok(());
    }

    referrer.rewards_godl = 0;
    referrer.rewards_sol = 0;

    if godl_amount > 0 {
        let authority_bytes = authority_info.key.to_bytes();
        let signer_seeds = &[REFERRER, authority_bytes.as_ref()];

        transfer_signed(
            referrer_info,
            referrer_tokens_info,
            authority_tokens_info,
            token_program,
            godl_amount,
            signer_seeds,
        )?;

        sol_log(
            &format!(
                "Claiming {} GODL from referrals",
                amount_to_ui_amount(godl_amount, TOKEN_DECIMALS)
            )
            .as_str(),
        );
    }

    if sol_amount > 0 {
        referrer_info.send(sol_amount, authority_info);
        sol_log(
            &format!(
                "Claiming {} SOL from referrals",
                lamports_to_sol(sol_amount)
            )
            .as_str(),
        );
    }

    Ok(())
}
