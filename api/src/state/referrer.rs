use serde::{Deserialize, Serialize};
use steel::*;

use crate::state::referrer_pda;

use super::GodlAccount;

#[repr(C)]
#[derive(Clone, Copy, Debug, PartialEq, Pod, Zeroable, Serialize, Deserialize)]
pub struct Referrer {
    /// The authority that can claim accrued referral rewards.
    pub authority: Pubkey,

    /// Pending SOL rewards denominated in lamports.
    pub rewards_sol: u64,

    /// Pending GODL rewards denominated in grams.
    pub rewards_godl: u64,

    /// Total SOL ever accrued for analytics purposes.
    pub cumulative_rewards_sol: u64,

    /// Total GODL ever accrued for analytics purposes.
    pub cumulative_rewards_godl: u64,

    /// Number of miners currently referring to this authority.
    pub referrer_count: u64,
}

impl Referrer {
    pub fn pda(&self) -> (Pubkey, u8) {
        referrer_pda(self.authority)
    }
}

account!(GodlAccount, Referrer);
