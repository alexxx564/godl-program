# GODL

GODL is a crypto mining protocol.

## API

- [`Consts`](api/src/consts.rs) – Program constants.
- [`Error`](api/src/error.rs) – Custom program errors.
- [`Event`](api/src/error.rs) – Custom program events.
- [`Instruction`](api/src/instruction.rs) – Declared instructions and arguments.

## Instructions

#### Mining

- [`Automate`](program/src/automate.rs) - Configures a new automation.
- [`AutomateV2`](program/src/automate_v2.rs) - Configures the v2 automation strategy.
- [`AutomateV3`](program/src/automate_v3.rs) - Configures automation with pooled deployment metadata.
- [`Checkpoint`](program/src/checkpoint.rs) - Checkpoints rewards from a prior round.
- [`CheckpointV3`](program/src/checkpoint_v3.rs) - Checkpoints rewards with pooled top-miner sharing.
- [`ClaimGODL`](program/src/claim_godl.rs) - Claims GODL mining rewards.
- [`ClaimSOL`](program/src/claim_sol.rs) - Claims SOL mining rewards.
- [`Deploy`](program/src/deploy.rs) – Deploys SOL to claim space on the board.
- [`DeployV2`](program/src/deploy_v2.rs) – Deploys SOL with automation hooks.
- [`DeployV3`](program/src/deploy_v3.rs) – Deploys SOL with optional pooling metadata.
- [`Initialize`](program/src/initialize.rs) - Initializes program variables.
- [`Log`](program/src/log.rs) – Logs non-truncatable event data.
- [`Reset`](program/src/reset.rs) - Resets the board for a new round.
- [`ResetV2`](program/src/reset_v2.rs) - Resets the board with entropy sampling.
- [`ResetV3`](program/src/reset_v3.rs) - Resets the board and flags pooled top-miner wins.

#### Staking

- [`Deposit`](program/src/deposit.rs) - Deposits GODL into a stake account.
- [`Withdraw`](program/src/withdraw.rs) - Withdraws GODL from a stake account.
- [`ClaimSeeker`](program/src/claim_seeker.rs) - Claims a Seeker genesis token.
- [`ClaimYield`](program/src/claim_yield.rs) - Claims staking yield.

#### Admin

- [`Bury`](program/src/bury.rs) - Executes a buy-and-bury transaction.
- [`Wrap`](program/src/wrap.rs) - Wraps SOL in the treasury for swap transactions.
- [`SetAdmin`](program/src/set_admin.rs) - Re-assigns the admin authority.
- [`SetFeeCollector`](program/src/set_admin.rs) - Updates the fee collection address.
- [`SetFeeRate`](program/src/set_admin.rs) - Updates the fee charged per swap.

## State

- [`Automation`](api/src/state/automation.rs) - Tracks automation configs.
- [`Board`](api/src/state/board.rs) - Tracks the current round number and timestamps.
- [`Config`](api/src/state/config.rs) - Global program configs.
- [`Miner`](api/src/state/miner.rs) - Tracks a miner's game state.
- [`PoolMember`](api/src/state/pool_member.rs) - Records an authority's pooled deployments per round.
- [`PoolRound`](api/src/state/pool_round.rs) - Aggregates pooled SOL per square for a round.
- [`Round`](api/src/state/round.rs) - Tracks the game state of a given round.
- [`Seeker`](api/src/state/seeker.rs) - Tracks whether a Seeker token has been claimed.
- [`Stake`](api/src/state/stake.rs) - Manages a user's staking activity.
- [`Treasury`](api/src/state/treasury.rs) - Mints, burns, and escrows GODL tokens.

## Mining Pool

- Use [`AutomateV3`](program/src/automate_v3.rs) to configure automation strategies that deploy via the pool without manual toggles.
- [`DeployV3`](program/src/deploy_v3.rs) (exposed via the CLI `--pooled` flag) opts miners into the shared pool for a round and provisions the `PoolMember`/`PoolRound` accounts.
- [`CheckpointV3`](program/src/checkpoint_v3.rs) distributes non-split top-miner rewards proportionally between pooled members whenever `ResetV3` flagged the round as a pool win.
- [`ResetV3`](program/src/reset_v3.rs) verifies the provided top miner and corresponding pool membership before marking `round.top_miner = POOL_ADDRESS`, enabling proportional payouts.

## CLI

- `close-rounds` – Closes expired round accounts that belong to the current payer, batching up to 20 closures per transaction to reclaim rent efficiently.

## Tests

To run the test suite, use the Solana toolchain:

```
cargo test-sbf
```

For line coverage, use llvm-cov:

```
cargo llvm-cov
```
