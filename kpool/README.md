# kpool — Constant Product AMM

Uniswap V2-style automated market maker on Solana. Liquidity providers deposit token pairs, traders swap against the pool, and fees accrue to LPs through the constant product invariant (`x * y = k`).

## Instructions

| Instruction | Description |
|---|---|
| `initialize_pool` | Create pool, vaults, LP mint, and locked-LP vault. Mints are sorted canonically (`mint_a < mint_b`). |
| `add_liquidity` | Deposit tokens proportional to current reserves, receive LP tokens. First deposit uses `sqrt(a * b)` with minimum liquidity locked. |
| `remove_liquidity` | Burn LP tokens, receive proportional share of both reserves. Slippage protection via `min_amount_a` / `min_amount_b`. |
| `swap` | Swap one token for the other. 0.30% fee applied to input. Slippage protection via `minimum_amount_out`. |

## Account Architecture

```
Pool PDA          ["pool", mint_a, mint_b]           Pool state (reserves, bumps, keys)
Pool Authority    ["pool_authority", pool]            Signs vault transfers and LP minting
LP Mint           ["lp_mint", pool]                   Fungible LP token (6 decimals)
Vault A / B       ATA(pool_authority, mint_a/b)       Hold deposited tokens
Locked LP Vault   ATA(pool_authority, lp_mint)        Holds minimum locked liquidity
```

## Key Design Decisions

| Decision | Choice | Rationale |
|---|---|---|
| AMM formula | `x * y = k` | Uniswap V2 — simple, proven, no oracle dependency |
| Fee model | 0.30% on input (30/10000) | Standard DEX fee, accrues to LPs via reserve growth |
| LP token | SPL mint PDA (6 decimals) | Fungible, transferable, composable |
| Minimum liquidity | 1,000 LP tokens locked on first deposit | Prevents inflation/donation attacks on empty pools |
| Mint ordering | `mint_a < mint_b` by pubkey | Canonical ordering prevents duplicate pools |
| Reserves | Tracked in Pool state (not vault balance) | Avoids re-reading vault accounts, keeps constant-product invariant tight |

## Math

```
Swap output:
  amount_out = (reserve_out * amount_in * 9970) / (reserve_in * 10000 + amount_in * 9970)

First deposit LP tokens:
  lp = sqrt(amount_a * amount_b) - MINIMUM_LIQUIDITY

Subsequent deposit LP tokens:
  lp = min(amount_a * supply / reserve_a, amount_b * supply / reserve_b)

Remove liquidity:
  amount_a = lp_burn * reserve_a / supply
  amount_b = lp_burn * reserve_b / supply
```

## Build & Test

```bash
anchor build
cd tests-litesvm && cargo test
```

13 tests covering initialization, deposits, swaps (both directions), slippage protection, remove liquidity, price impact, fee accrual, and a full lifecycle test.

## Project Layout

```
programs/cpamm/src/
  lib.rs              4 instruction dispatchers
  constants.rs        PDA seeds, fee params, minimum liquidity
  errors.rs           CpammError enum
  state.rs            Pool struct
  instructions/
    initialize.rs     Pool + vault + LP mint creation
    add_liquidity.rs  Proportional deposit + first-deposit sqrt logic
    remove_liquidity.rs  Burn LP + proportional withdrawal
    swap.rs           Fee-adjusted constant product swap
tests-litesvm/src/
  lib.rs              13 LiteSVM integration tests
```
