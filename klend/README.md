# klend — Lending/Borrowing Protocol

Aave V2/Compound V2-style lending protocol on Solana. Users deposit assets to earn interest, borrow against collateral, and face liquidation if undercollateralized.

## Instructions

| Instruction | Description |
|---|---|
| `init_market` | Create a lending market (admin-controlled). |
| `init_mock_oracle` | Create a price oracle for a token mint (mock for testing). |
| `update_mock_oracle` | Update oracle price and timestamp. |
| `init_reserve` | Create a reserve for an asset with rate model, LTV, caps, etc. |
| `refresh_reserve` | Accrue interest, update borrow index and protocol fees. Must be called before deposit/withdraw/borrow/repay. |
| `deposit` | Deposit tokens into a reserve, receive shares tracked in an obligation. Creates obligation on first deposit. |
| `withdraw` | Burn shares, receive underlying tokens from the vault. |
| `borrow` | Borrow tokens against deposited collateral. Health factor must stay >= 1.0. |
| `repay` | Repay borrowed tokens. Caps at current debt. Can repay on behalf of others. |
| `liquidate` | Repay another user's debt and seize their collateral at a bonus. 50% close factor, 5% liquidation bonus. |

## Account Architecture

```
LendingMarket PDA    ["lending_market", admin]                      Global config
Reserve PDA          ["reserve", market, token_mint]                Per-asset pool state
Reserve Authority    ["reserve_authority", reserve]                  Signs vault transfers
Obligation PDA       ["obligation", market, user]                   Per-user position (deposits + borrows)
MockOracle PDA       ["mock_oracle", token_mint]                    Price feed
Token Vault          ATA(reserve_authority, token_mint)              Holds deposited tokens
```

## Accounting Model

The reserve uses a Compound-style accounting model where `deposited_liquidity` tracks physical cash in the vault:

```
deposited_liquidity  = tokens physically in the vault
borrowed_liquidity   = outstanding borrow debt (grows with accrued interest)
accumulated_fees     = protocol fees (portion of interest)

total_assets()       = deposited + borrowed - fees   (total value of the pool)
available_liquidity() = deposited                     (cash available to borrow/withdraw)
```

Token flow:
- **Deposit**: deposited += amount
- **Withdraw**: deposited -= amount
- **Borrow**: deposited -= amount, borrowed += amount
- **Repay**: deposited += amount, borrowed -= amount
- **Refresh**: borrowed += interest, fees += protocol_share
- **Liquidate**: debt_reserve.deposited += repay, debt_reserve.borrowed -= repay, collateral_reserve.deposited -= seized

## Interest Rate Model

Kinked rate model (Aave V2-style):

```
If utilization <= U_optimal:
  borrow_rate = r_base + (U / U_optimal) * r_slope1

If utilization > U_optimal:
  borrow_rate = r_base + r_slope1 + ((U - U_optimal) / (1 - U_optimal)) * r_slope2
```

Interest accrues on `refresh_reserve` via a cumulative borrow index (1e18 scaled). Borrower debt = `scaled_amount * current_index / SCALE`.

## Share/Exchange Rate

Depositors receive shares (Compound cToken-style) with a virtual offset for inflation attack defense:

```
shares     = amount * (total_shares + 1) / (total_assets + 1)     // rounds down
underlying = shares * (total_assets + 1) / (total_shares + 1)     // rounds down
```

## Key Design Decisions

| Decision | Choice | Rationale |
|---|---|---|
| Interest model | Kinked (base + slope1 + slope2) | Standard Aave V2 pattern, incentivizes optimal utilization |
| Scaling | 1e18 fixed-point | Sufficient precision for interest compounding |
| Obligation model | Single PDA per user per market | Tracks deposits + borrows across reserves |
| Oracle | Mock PDA (swappable for Pyth/Switchboard) | Testable locally, production-ready interface |
| Liquidation | 50% close factor, 5% bonus | Standard parameters, partial liquidation reduces cascading risk |
| Rounding | Debt rounds up, shares round down | Always favor the protocol |
| Reserve freshness | 2-second staleness check | Forces `refresh_reserve` before state-changing operations |

## Build & Test

```bash
anchor build
cd tests-litesvm && cargo test
```

18 tests covering market/reserve init, deposits, withdrawals, borrows, repays, interest accrual, exchange rates, liquidation, close factor enforcement, oracle staleness, supply/borrow caps, and a full lifecycle test.

## Project Layout

```
programs/klend/src/
  lib.rs                10 instruction dispatchers
  constants.rs          PDA seeds, scaling, liquidation params
  errors.rs             KlendError enum
  math.rs               Interest rates, share conversion, health factor, collateral valuation
  state.rs              LendingMarket, Reserve, Obligation, MockOracle
  instructions/
    init_market.rs      Create lending market
    init_mock_oracle.rs Create mock price oracle
    update_mock_oracle.rs Update oracle price
    init_reserve.rs     Create reserve with config validation
    refresh_reserve.rs  Accrue interest, update index
    deposit.rs          Deposit tokens, mint shares
    withdraw.rs         Burn shares, withdraw tokens
    borrow.rs           Borrow against collateral (health factor check)
    repay.rs            Repay debt
    liquidate.rs        Seize collateral from unhealthy positions
tests-litesvm/src/
  lib.rs                18 LiteSVM integration tests
```
