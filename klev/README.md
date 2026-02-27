# klev — Leveraged Yield Vault

Kamino Multiply-style leveraged yield vault on Solana. Composes klend (lending) and kpool (CPAMM) via CPI. Users deposit SOL and receive share tokens. An admin loops: deposit SOL into klend as collateral → borrow USDC → swap USDC→SOL via kpool → deposit SOL back, amplifying SOL exposure and supply yield by 2-3x.

## Instructions

| Instruction | Description |
|---|---|
| `init_vault` | Create vault PDA, share mint, vault authority, collateral + debt token accounts. Funds authority PDA with 0.1 SOL for klend obligation rent. Stores klend + cpamm references. |
| `deposit` | User deposits collateral SOL, receives share tokens. `total_assets = idle + cached_net_equity`. Respects deposit cap and halt flag. |
| `withdraw` | User burns shares, receives SOL from idle balance. Rounds down (favors protocol). Always allowed even when halted. Fails if idle < withdrawal amount (admin must deleverage first). |
| `leverage_up` | 4 sequential CPIs: klend::deposit → klend::borrow → cpamm::swap (USDC→SOL) → klend::deposit. Balance reload for swap output measurement. Post-check: leverage ratio ≤ max_leverage_bps. |
| `deleverage` | 3 sequential CPIs: klend::withdraw → cpamm::swap (SOL→USDC) → klend::repay. Reverse of leverage_up. Repay capped to swap output. |
| `harvest` | Reads klend state (reserves + obligation + oracles). Computes current net equity, yield, and fees. Mints dilutive fee shares for performance + management fees. Updates cached values. |
| `set_halt` | Admin toggles halt flag. Blocks deposits; withdrawals always allowed. |

## Account Architecture

```
LeveragedVault PDA  ["leveraged_vault", collateral_mint, debt_mint]   Vault state
Vault Authority     ["lev_vault_authority", vault]                     Signs CPI + transfers
Share Mint          ["lev_share_mint", vault]                          Fungible share tokens
Collateral ATA      ATA(vault_authority, collateral_mint)              Idle SOL
Debt ATA            ATA(vault_authority, debt_mint)                    Intermediate USDC
```

## Leverage Loop

```
User deposits SOL
       │
       ▼
┌─────────────────── leverage_up (admin, repeatable) ──────────────────┐
│                                                                      │
│  idle SOL ──deposit──▶ klend (collateral)                            │
│                              │                                       │
│                          borrow USDC                                 │
│                              │                                       │
│                         cpamm::swap                                  │
│                         USDC → SOL                                   │
│                              │                                       │
│                   deposit swapped SOL ──▶ klend (more collateral)    │
│                                                                      │
│  Result: SOL exposure amplified, USDC debt outstanding               │
└──────────────────────────────────────────────────────────────────────┘

deleverage reverses: klend::withdraw → cpamm::swap SOL→USDC → klend::repay
```

## Math

### Share Accounting (ERC-4626)

```
total_assets = idle_collateral + cached_net_equity_collateral

deposit:   shares = amount * (supply + 1) / (total_assets + 1)      rounds DOWN
withdraw:  amount = shares * (total_assets + 1) / (supply + 1)      rounds DOWN

Virtual +1 offset prevents inflation attack (first depositor can't manipulate exchange rate).
```

### Net Equity & Leverage

```
debt_in_collateral = debt_amount * debt_price * 10^coll_decimals / (coll_price * 10^debt_decimals)
net_equity         = collateral_underlying - debt_in_collateral     (saturating)
leverage_ratio     = total_collateral * 10000 / net_equity          (in bps: 20000 = 2x)
```

### Fee Extraction (Yearn V3 Dilutive Minting)

```
performance_fee = yield * performance_fee_bps / 10000
management_fee  = total_assets * management_fee_bps * elapsed_secs / (10000 * 31536000)

fee_shares = fee_underlying * (supply + 1) / (total_assets + 1 - fee_underlying)
```

Minting new shares dilutes existing holders proportionally — no token transfers needed.

### klend State Reads

```
collateral_underlying = klend_shares * (reserve_total_assets + 1) / (reserve_total_shares + 1)
current_debt          = borrowed_amount_scaled * cumulative_borrow_index / 1e18    (rounded up)
```

## CPI Account Mappings

| CPI Call | vault_authority maps to | Token accounts |
|---|---|---|
| `klend::deposit` | `user` | collateral_token_account → `user_token_account` |
| `klend::borrow` | `user`, `owner` | debt_token_account → `user_token_account` |
| `klend::withdraw` | `user`, `owner` | collateral_token_account → `user_token_account` |
| `klend::repay` | `user` | debt_token_account → `user_token_account` |
| `cpamm::swap` | `user` | direction-dependent: in=debt/out=collateral or vice versa |

## Key Design Decisions

| Decision | Choice | Rationale |
|---|---|---|
| Vault struct | `Box<Account>` | LeveragedVault is 510 bytes — heap-allocating avoids BPF 4KB stack overflow |
| Token accounts | `UncheckedAccount` + `address =` | Avoids `Account<TokenAccount>` (165 bytes each) on stack |
| klend state reads | `#[inline(never)]` helper | Isolates large deserialization from caller's stack frame |
| Balance measurement | Raw byte read at offset 64 | `read_token_balance()` reads SPL amount without Account overhead |
| Cached state | Updated at leverage/deleverage/harvest | Avoids klend deserialization on every deposit/withdraw |
| Withdraw source | Idle balance only | Simple, avoids CPI during user withdrawal. Admin deleverages to refill. |
| Share math | Virtual +1 offset | ERC-4626 inflation attack defense |
| Fee model | Dilutive minting | No fee token transfers needed, Yearn V3 pattern |
| Leverage check | Post-operation in leverage_up | klend enforces health factor; klev adds max leverage cap |
| Halt | Blocks deposits only | Withdrawals always available for user safety |

## Build & Test

```bash
cd ../klend && anchor build       # CPI dependency
cd ../kpool && anchor build       # CPI dependency
anchor build
cd tests-litesvm && cargo test
```

18 tests covering vault init (2), deposit/withdraw (5), leverage_up (4), deleverage (1), harvest (2), halt (2), lifecycle (1), and slippage protection (1).

## Project Layout

```
programs/klev/src/
  lib.rs                7 instruction dispatchers
  constants.rs          PDA seeds, BPS_SCALE, SECONDS_PER_YEAR, KLEND_SCALE, ORACLE_PRICE_SCALE
  errors.rs             KlevError enum (19 variants)
  math.rs               Share math, fee shares, klend share/debt conversion,
                        debt-to-collateral price conversion, net equity, leverage ratio
  state.rs              LeveragedVault (14 Pubkeys, cached values, fee/leverage params, bumps)
  instructions/
    init_vault.rs       Vault + authority + share mint + token accounts creation
    deposit.rs          User deposits SOL, receives shares
    withdraw.rs         User burns shares, receives SOL from idle
    leverage_up.rs      4 CPIs: klend deposit → borrow → cpamm swap → klend deposit
    deleverage.rs       3 CPIs: klend withdraw → cpamm swap → klend repay
    harvest.rs          klend state deserialization, net equity, fee minting
    set_halt.rs         Toggle halt flag
tests-litesvm/src/
  lib.rs                18 LiteSVM integration tests (loads klend + cpamm + klev)
```
