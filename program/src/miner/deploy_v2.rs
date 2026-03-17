use steel::*;

/// Deploys capital to prospect on a square.
///
/// Deprecated: Use [`DeployV3`] instead.
pub fn process_deploy_v2(_accounts: &[AccountInfo<'_>], _data: &[u8]) -> ProgramResult {
    Err(ProgramError::InvalidInstructionData)
}
