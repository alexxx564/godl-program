use serde::{Deserialize, Serialize};
use steel::*;

use crate::state::pool_round_pda;

use super::GodlAccount;

#[repr(C)]
#[derive(Clone, Copy, Debug, PartialEq, Pod, Zeroable, Serialize, Deserialize)]
pub struct PoolRound {
    /// The identifier of the underlying game round.
    pub id: u64,

    /// Aggregate SOL deployed by the pool on each square.
    pub deployed: [u64; 25],

    /// Count of pool members that deployed on each square.
    pub count: [u64; 25],

    /// Total SOL deployed by the pool in the round.
    pub total_deployed: u64,

    /// Account that will receive rent refunds when the pool round closes.
    pub rent_payer: Pubkey,
}

impl PoolRound {
    pub fn pda(&self) -> (Pubkey, u8) {
        pool_round_pda(self.id)
    }

    pub fn reset(&mut self, id: u64, rent_payer: Pubkey) {
        self.id = id;
        self.deployed = [0; 25];
        self.count = [0; 25];
        self.total_deployed = 0;
        self.rent_payer = rent_payer;
    }

    pub fn record_deploy(&mut self, square: usize, amount: u64, is_new_member: bool) {
        if square >= self.deployed.len() {
            return;
        }
        self.deployed[square] = self.deployed[square].saturating_add(amount);
        if is_new_member {
            self.count[square] = self.count[square].saturating_add(1);
        }
        self.total_deployed = self.total_deployed.saturating_add(amount);
    }
}

account!(GodlAccount, PoolRound);
