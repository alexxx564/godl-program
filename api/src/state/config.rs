use serde::{Deserialize, Serialize};
use steel::*;

use crate::state::config_pda;

use super::GodlAccount;

#[repr(C)]
#[derive(Clone, Copy, Debug, PartialEq, Pod, Zeroable, Serialize, Deserialize)]
pub struct Config {
    /// The address that can update the config.
    pub admin: Pubkey,

    /// The amount of GODL minted per round.
    pub godl_per_round: u64,

    /// The adress with authority to call bury.
    pub bury_authority: Pubkey,

    /// The address that receives admin fees.
    pub fee_collector: Pubkey,

    /// The program id for the protocol swaps.
    pub swap_program: Pubkey,

    /// The address of the entropy var account.
    pub var_address: Pubkey,

    /// The denominator to calculate the motherlode.
    pub motherlode_denominator: u64,
}

impl Config {
    pub fn pda() -> (Pubkey, u8) {
        config_pda()
    }
}

account!(GodlAccount, Config);
