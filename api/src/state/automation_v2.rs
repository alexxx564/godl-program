use serde::{Deserialize, Serialize};
use steel::*;

use crate::state::automation_v2_pda;

use super::GodlAccount;

#[repr(C)]
#[derive(Clone, Copy, Debug, PartialEq, Pod, Zeroable, Serialize, Deserialize)]
pub struct AutomationV2 {
    /// The amount of SOL to deploy on each territory per round.
    pub amount: u64,

    /// The authority of this automation account.
    pub authority: Pubkey,

    /// The amount of SOL this automation has left.
    pub balance: u64,

    /// The executor of this automation account.
    pub executor: Pubkey,

    /// The amount of SOL the executor should receive in fees.
    pub fee: u64,

    /// Packed config for this automation:
    /// - bits 0..7   : strategy (AutomationV2Strategy)
    /// - bits 8..15  : claim_and_fund flag
    /// - bits 16..23 : is_pooled flag
    /// - bits 24..63 : reserved for future use
    pub config: u64,

    /// The mask of squares this automation should deploy to if preferred strategy.
    pub mask: u64,
}

#[repr(u8)]
#[derive(Clone, Copy, Debug, PartialEq, IntoPrimitive, TryFromPrimitive)]
pub enum AutomationV2Strategy {
    Random = 0,
    Preferred = 1,
}

impl AutomationV2Strategy {
    pub fn from_config(config: u64) -> Self {
        let strat_byte = (config & 0xFF) as u8;
        Self::try_from(strat_byte).unwrap()
    }

    pub fn from_u64(value: u64) -> Self {
        Self::from_config(value)
    }
}

impl AutomationV2 {
    pub fn pda(&self) -> (Pubkey, u8) {
        automation_v2_pda(self.authority)
    }

    /// Extract a specific 8-bit slot from the config.
    pub fn config_byte(config: u64, index: u8) -> u8 {
        let shift = (index as u64) * 8;
        ((config >> shift) & 0xFF) as u8
    }

    /// Internal helper to write an 8-bit value into a specific slot.
    fn set_config_byte(&mut self, index: u8, value: u8) {
        let shift = (index as u64) * 8;
        let clear_mask = !(0xFFu64 << shift);
        self.config = (self.config & clear_mask) | ((value as u64) << shift);
    }

    /// Convenience: strategy (byte 0)
    pub fn get_strategy(&self) -> AutomationV2Strategy {
        AutomationV2Strategy::from_config(self.config)
    }

    /// Set strategy in byte 0
    pub fn set_strategy(&mut self, strategy: AutomationV2Strategy) {
        let value: u8 = strategy.into();
        self.set_config_byte(0, value);
    }

    /// claim_and_fund flag (byte 1)
    pub fn get_claim_and_fund(&self) -> bool {
        Self::config_byte(self.config, 1) != 0
    }

    /// Set claim_and_fund flag in byte 1
    pub fn set_claim_and_fund(&mut self, enabled: bool) {
        let value: u8 = if enabled { 1 } else { 0 };
        self.set_config_byte(1, value);
    }

    pub fn get_is_pooled(&self) -> bool {
        Self::config_byte(self.config, 2) != 0
    }

    pub fn set_is_pooled(&mut self, enabled: bool) {
        let value: u8 = if enabled { 1 } else { 0 };
        self.set_config_byte(2, value);
    }
}

account!(GodlAccount, AutomationV2);
