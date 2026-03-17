use godl_api::prelude::*;
use solana_program::{log::sol_log, native_token::lamports_to_sol};
use steel::*;

/// Claims a block reward.
pub fn process_claim_sol(accounts: &[AccountInfo<'_>], _data: &[u8]) -> ProgramResult {
    // Load accounts.
    let clock = Clock::get()?;
    let [signer_info, miner_info, system_program, rest @ ..] = accounts else {
        return Err(ProgramError::NotEnoughAccountKeys);
    };
    signer_info.is_signer()?;
    let miner = miner_info
        .as_account_mut::<Miner>(&godl_api::ID)?
        .assert_mut(|m| m.authority == *signer_info.key)?;
    system_program.is_program(&system_program::ID)?;

    // Normalize amount.
    let amount = miner.claim_sol(&clock);
    let referrer_pda = miner.referrer_account();
    let referral_amount = if referrer_pda.is_some() {
        amount
            .checked_mul(REFERRAL_BPS)
            .ok_or(ProgramError::InvalidInstructionData)?
            / DENOMINATOR_BPS
    } else {
        0
    };
    let miner_amount = amount
        .checked_sub(referral_amount)
        .ok_or(ProgramError::InvalidInstructionData)?;

    sol_log(&format!("Claiming {} SOL", lamports_to_sol(miner_amount)).as_str());
    if referral_amount > 0 {
        sol_log(&format!("Referral amount: {} SOL", lamports_to_sol(referral_amount)).as_str());
    }

    if let Some(referrer_pubkey) = referrer_pda {
        let [referrer_info] = rest else {
            return Err(ProgramError::NotEnoughAccountKeys);
        };
        let referrer = referrer_info.as_account_mut::<Referrer>(&godl_api::ID)?;
        referrer_info
            .is_writable()?
            .has_address(&referrer_pubkey)?
            .has_seeds(&[REFERRER, &referrer.authority.to_bytes()], &godl_api::ID)?;

        referrer.rewards_sol = referrer
            .rewards_sol
            .checked_add(referral_amount)
            .ok_or(ProgramError::InvalidInstructionData)?;
        referrer.cumulative_rewards_sol = referrer
            .cumulative_rewards_sol
            .checked_add(referral_amount)
            .ok_or(ProgramError::InvalidInstructionData)?;

        miner_info.send(referral_amount, referrer_info);
    } else if !rest.is_empty() {
        return Err(trace(
            "Unexpected referrer accounts",
            GodlError::InvalidReferrerAccount.into(),
        ));
    }

    if miner_amount > 0 {
        miner_info.send(miner_amount, signer_info);
    }

    Ok(())
}
