# Xaver

An [ink!](https://use.ink/) smart contract for node staking on [Substrate](https://substrate.io/)-based blockchains. Xaver allows an operator to manage staked positions, distribute income to stakers, and process unstaking once a cessation block is reached.

---

## Overview

Xaver is a staking contract where:

- An **owner** deploys and configures the contract.
- An **operator** manages day-to-day operations: opening/closing staking, registering stakers, crediting income, and processing unstakes.
- **Stakers** lock a position for a fixed duration (tracked in block numbers). Upon cessation, the operator can unstake and transfer accumulated income to the staker in a stable asset (e.g., USDT).
- A **receipt token** (e.g., XAV) is minted and transferred to the staker upon staking as proof of participation.

---

## Contract Storage

| Field | Type | Description |
|---|---|---|
| `asset_id` | `u128` | Receipt token asset ID (e.g., XAV) |
| `stable_asset_id` | `u128` | Stable asset ID for income payouts (e.g., USDT) |
| `owner` | `AccountId` | Contract deployer; can run `setup` |
| `operator` | `AccountId` | Manages staking operations |
| `price` | `u16` | Price of a stake slot |
| `share` | `u16` | Share percentage for income distribution |
| `maximum_stakes` | `u16` | Maximum number of concurrent stakers |
| `duration` | `u128` | Stake duration in blocks |
| `stakes` | `Vec<Stake>` | List of active stakes |
| `status` | `u8` | `0` = Open, `1` = Closed |

### Stake Struct

| Field | Type | Description |
|---|---|---|
| `account` | `AccountId` | Staker's account address |
| `tx_hash` | `Vec<u8>` | Off-chain payment tx hash (USDT proof) |
| `accumulated_income` | `u128` | Total income credited so far |
| `cessation_block` | `u128` | Block number when the stake expires |
| `status` | `u8` | `0` = Frozen, `1` = Liquid |

---

## Messages

### `new(asset_id, stable_asset_id, maximum_stakes)` — Constructor
Instantiates the contract. The caller becomes both `owner` and `operator`.

### `default()` — Constructor
Instantiates with zeroed defaults.

### `setup(...)` — Owner only
Reconfigures the contract. **This clears all existing stakes.** Only callable by the `owner`.

### `get() → (...)` 
Returns all contract configuration fields.

### `open()` — Operator only
Sets `status = 0`, allowing new stakes to be added.

### `close()` — Operator only
Sets `status = 1`, halting new staking activity.

### `stake(account, tx_hash)` — Operator only
Registers a new staker after verifying their off-chain payment via `tx_hash`. Transfers the receipt token (e.g., XAV) to the staker. Fails if:
- The contract is closed
- The account is already staked
- Maximum stake capacity is reached

### `unstake(account)` — Operator only
Transfers accumulated income (stable asset) to the staker and removes the stake. Fails if:
- The contract is closed
- The stake's cessation block has not been reached
- The account is not found in stakes

### `income(amount)` — Operator only
Credits income to all **liquid** (`status = 1`) stakers proportionally:
```
income_per_staker = (amount × share) / maximum_stakes
```

### `get_staker(account) → Option<Stake>`
Returns staker information for a given account, or `None` if not found.

---

## Events

All operations emit a `XaverEvent`:

```rust
pub struct XaverEvent {
    operator: AccountId,
    status: XaverTransactionStatus,  // EmitSuccess(...) or EmitError(...)
}
```

### Success variants
| Variant | Emitted by |
|---|---|
| `XaverSetupSuccess` | `setup` |
| `XaverOpenSuccess` | `open` |
| `XaverCloseSuccess` | `close` |
| `StakingSuccess` | `stake` |
| `UnstakingSuccess` | `unstake` |
| `IncomeCreditSuccess` | `income` |

### Error variants
| Variant | Meaning |
|---|---|
| `BadOrigin` | Caller is not the authorized owner/operator |
| `XaverIsClose` | Contract is closed for staking |
| `XaverStakeAlreadyExist` | Account is already staked |
| `XaverStakingMaxOut` | Maximum stake capacity reached |
| `XaverStakeNotFound` | No stake found for the given account |
| `XaverStakeNotCeased` | Cessation block has not been reached yet |

---

## Getting Started

### Prerequisites

- [Rust](https://www.rust-lang.org/tools/install) with `wasm32-unknown-unknown` target
- [ink! CLI (`cargo-contract`)](https://github.com/paritytech/cargo-contract)

```bash
rustup target add wasm32-unknown-unknown
cargo install cargo-contract
```

### Build

```bash
cargo contract build
```

### Test

```bash
cargo test
```

### Deploy

Upload the compiled `.contract` file to a Substrate node with `pallet-contracts` and `pallet-assets` enabled (e.g., via [Contracts UI](https://contracts-ui.substrate.io/)).

---

## Architecture Notes

- **`pallet_assets` integration** — Xaver calls `pallet_assets::transfer` at runtime via `call_runtime` to handle both receipt token minting (on stake) and stable asset payouts (on unstake).
- **Block-based duration** — Stake expiry is measured in block numbers. At a 6-second block time, `5_256_000` blocks ≈ 1 year.
- **Operator model** — All sensitive mutations are gated behind the `operator` account rather than requiring stakers to call the contract directly. This allows off-chain verification (e.g., checking a USDT tx hash) before on-chain registration.

---

## License

MIT
