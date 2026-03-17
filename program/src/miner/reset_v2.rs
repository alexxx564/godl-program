use steel::*;

/// Pays out the winners and block reward.
///
/// Deprecated: Use [`ResetV3`] instead.
pub fn process_reset_v2(_accounts: &[AccountInfo<'_>], _data: &[u8]) -> ProgramResult {
    Err(ProgramError::InvalidInstructionData)
}
