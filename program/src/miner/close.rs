use steel::*;

/// Closes an expired round account.
///
/// Deprecated: Use [`CloseV2`] instead.
pub fn process_close(_accounts: &[AccountInfo<'_>], _data: &[u8]) -> ProgramResult {
    Err(ProgramError::InvalidInstructionData)
}
