use godl_api::prelude::*;
use solana_program::{log::sol_log, native_token::lamports_to_sol};
use steel::*;

/// Claims a block reward and funds an automation account.
pub fn process_claim_sol_and_fund_automation(
    accounts: &[AccountInfo<'_>],
    _data: &[u8],
) -> ProgramResult {
    // Load accounts.
    let clock = Clock::get()?;
    let [signer_info, miner_info, automation_v2_info, system_program, rest @ ..] = accounts else {
        return Err(ProgramError::NotEnoughAccountKeys);
    };
    signer_info.is_signer()?;

    let automation_v2 = automation_v2_info
        .as_account_mut::<AutomationV2>(&godl_api::ID)?
        .assert_mut(|a| a.executor == *signer_info.key)?
        .assert_mut(|a| a.get_claim_and_fund() == true)?;
    automation_v2_info.is_writable()?.has_seeds(
        &[AUTOMATION_V2, &automation_v2.authority.to_bytes()],
        &godl_api::ID,
    )?;
    let authority = automation_v2.authority;
    let miner = miner_info
        .as_account_mut::<Miner>(&godl_api::ID)?
        .assert_mut(|m| m.authority == authority)?;
    system_program.is_program(&system_program::ID)?;

    // Normalize amount.
    let amount = miner.claim_sol(&clock);
    let referrer_pda = miner.referrer_account();
    let referral_amount = if referrer_pda.is_some() {
        amount
            .checked_mul(REFERRAL_BPS / 10)
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
        automation_v2.balance += miner_amount;
        miner_info.send(miner_amount, automation_v2_info);
    }

    Ok(())
}
