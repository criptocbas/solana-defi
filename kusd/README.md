# kusd — CDP Stablecoin Protocol

MakerDAO/Liquity-style CDP stablecoin on Solana. Users deposit SOL collateral, mint kUSD stablecoins against it, and face liquidation if undercollateralized.

## Design

- **CdpVault**: One per collateral type. Stores config (max LTV, liquidation threshold, bonus, stability fee, debt ceiling), accounting (total collateral, total debt shares, cumulative fee index).
- **CdpPosition**: Per user per vault. Tracks collateral deposited and debt shares.
- **MockOracle**: Self-contained price oracle (identical to klend's).

### Key Mechanics

- **Stability Fee**: Annual fee accrued via cumulative fee index. Debt = shares × fee_index / SCALE.
- **Mint/Burn**: kUSD is minted from nothing when users borrow, burned when they repay.
- **Liquidation**: Unhealthy positions (HF < 1.0) can be partially liquidated (50% close factor) with a 5% bonus.

## Instructions

| # | Instruction | Purpose |
|---|---|---|
| 1 | `init_mock_oracle` | Create price oracle for a token |
| 2 | `update_mock_oracle` | Update oracle price |
| 3 | `init_vault` | Create CDP vault for a collateral type |
| 4 | `open_position` | Create user's CDP position |
| 5 | `deposit_collateral` | Deposit collateral into position |
| 6 | `mint_kusd` | Mint kUSD against collateral (LTV check) |
| 7 | `repay_kusd` | Burn kUSD to reduce debt |
| 8 | `withdraw_collateral` | Withdraw collateral (LTV check if debt) |
| 9 | `liquidate` | Liquidate unhealthy position |
| 10 | `accrue_fees` | Public crank to update fee index |
| 11 | `set_halt` | Admin toggle to block minting |

## Build & Test

```bash
cd kusd && anchor build
cd kusd/tests-litesvm && cargo test
```

## Program ID

`4niQV7cydNxRakwi4hM2jhSkp6dwg4abuzx5HsAwDz95`
