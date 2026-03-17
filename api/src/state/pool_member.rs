use serde::{Deserialize, Serialize};
use steel::*;

use crate::state::pool_member_pda;

use super::GodlAccount;

#[repr(C)]
#[derive(Clone, Copy, Debug, PartialEq, Pod, Zeroable, Serialize, Deserialize)]
pub struct PoolMember {
    /// Authority that owns this pool membership.
    pub authority: Pubkey,

    /// The round identifier this membership currently targets.
    pub round_id: u64,

    /// Amount of SOL deployed per square as part of the pool.
    pub deployed: [u64; 25],

    /// Total SOL deployed via the pool in the tracked round.
    pub total_deployed: u64,
}

impl PoolMember {
    pub fn pda(&self) -> (Pubkey, u8) {
        pool_member_pda(self.authority)
    }

    pub fn reset_for_round(&mut self, round_id: u64) {
        self.round_id = round_id;
        self.deployed = [0; 25];
        self.total_deployed = 0;
    }

    pub fn record_deploy(&mut self, square: usize, amount: u64) {
        if square >= self.deployed.len() {
            return;
        }
        self.deployed[square] = self.deployed[square].saturating_add(amount);
        self.total_deployed = self.total_deployed.saturating_add(amount);
    }
}

account!(GodlAccount, PoolMember);
