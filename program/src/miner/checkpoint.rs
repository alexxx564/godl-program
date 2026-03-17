use steel::*;

/// Checkpoints a miner's rewards.
///
/// Deprecated: Use [`CheckpointV3`] instead.
pub fn process_checkpoint(_accounts: &[AccountInfo<'_>], _data: &[u8]) -> ProgramResult {
    Err(ProgramError::InvalidInstructionData)
}
