use godl_api::prelude::*;
use solana_program::log::sol_log;
use steel::*;

/// Locks in a miner's referrer selection.
pub fn process_set_referrer(accounts: &[AccountInfo<'_>], data: &[u8]) -> ProgramResult {
    // Parse data.
    let args = SetReferrer::try_from_bytes(data)?;
    let desired_referrer = Pubkey::new_from_array(args.referrer);

    // Load accounts.
    let [authority_info, miner_info, rest @ ..] = accounts else {
        return Err(ProgramError::NotEnoughAccountKeys);
    };
    authority_info.is_signer()?;

    let miner = miner_info
        .as_account_mut::<Miner>(&godl_api::ID)?
        .assert_mut_err(
            |m| m.authority == *authority_info.key,
            GodlError::NotAuthorized.into(),
        )?;

    if miner.referrer != Pubkey::default() {
        sol_log(&format!("Referrer already set: {:?}", miner.referrer).as_str());
        return Ok(());
    }

    if desired_referrer == Pubkey::default() {
        miner.referrer = REFERRER_LOCKED_SENTINEL;
        return Ok(());
    }

    if desired_referrer == REFERRER_LOCKED_SENTINEL {
        return Err(trace(
            "Invalid referrer sentinel",
            GodlError::InvalidReferrerAccount.into(),
        ));
    }

    let [referrer_info] = rest else {
        return Err(ProgramError::NotEnoughAccountKeys);
    };
    let referrer = referrer_info.as_account_mut::<Referrer>(&godl_api::ID)?;
    referrer_info
        .is_writable()?
        .has_address(&desired_referrer)?
        .has_seeds(&[REFERRER, &referrer.authority.to_bytes()], &godl_api::ID)?;

    if referrer.authority == miner.authority {
        return Err(trace(
            "Self referral not allowed",
            GodlError::InvalidReferrerAccount.into(),
        ));
    }

    referrer.referrer_count = referrer
        .referrer_count
        .checked_add(1)
        .ok_or(ProgramError::InvalidArgument)?;

    miner.referrer = *referrer_info.key;

    Ok(())
}
