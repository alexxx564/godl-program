use serde::{Deserialize, Serialize};
use steel::*;

use super::GodlAccount;

/// Treasury is a singleton account which is the mint authority for the GODL token and the authority of
/// the program's global token account.
#[repr(C)]
#[derive(Clone, Copy, Debug, PartialEq, Pod, Zeroable, Serialize, Deserialize)]
pub struct Treasury {
    // The amount of SOL collected for buy-bury operations.
    pub balance: u64,

    /// The amount of GODL in the motherlode rewards pool.
    pub motherlode: u64,

    /// The cumulative GODL distributed to miners, divided by the total unclaimed GODL at the time of distribution.
    pub miner_rewards_factor: Numeric,

    /// The cumulative GODL distributed to stakers, divided by the total stake at the time of distribution.
    pub stake_rewards_factor: Numeric,

    /// The current total amount of GODL staking deposits.
    pub total_staked: u64,

    /// The current total amount of unclaimed GODL mining rewards.
    pub total_unclaimed: u64,

    /// The current total amount of refined GODL mining rewards.
    pub total_refined: u64,
}

account!(GodlAccount, Treasury);
