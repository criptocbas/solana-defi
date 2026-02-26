# kclmm — Concentrated Liquidity AMM

Uniswap V3/Orca Whirlpool-style concentrated liquidity AMM on Solana. Liquidity providers deposit tokens in custom price ranges for dramatically higher capital efficiency. Swaps traverse tick boundaries, updating active liquidity at each crossing. Fees accrue per unit of liquidity and are tracked per-position.

## Instructions

| Instruction | Description |
|---|---|
| `init_pool` | Create pool PDA, pool authority PDA, and vault ATAs. Mints sorted canonically (`mint_a < mint_b`). Derives tick spacing from fee tier. |
| `init_tick_array` | Lazy-init a tick array for a range of 64 ticks. Start index must be aligned to `tick_spacing * 64`. |
| `open_position` | Create an empty position PDA for a given tick range. Validates alignment and bounds. |
| `add_liquidity` | Add liquidity to a position. Accrues pending fees, updates ticks (net/gross, bitmap, fee outside), updates pool active liquidity if in range, transfers tokens from user. |
| `remove_liquidity` | Remove liquidity from a position. Mirror of add — subtracts from ticks, clears tick if gross reaches 0. Transfers tokens to user via pool authority. |
| `collect_fees` | Claim accumulated trading fees from a position. Recomputes fee growth inside, transfers owed tokens. |
| `swap` | Swap with tick traversal loop. Finds next initialized tick via bitmap, computes step, accrues fees, crosses tick if reached. Up to 3 tick arrays via `remaining_accounts`, max 20 tick crossings. |
| `close_position` | Close an empty position (liquidity == 0, tokens owed == 0), reclaim rent to owner. |

## Account Architecture

```
Pool PDA           ["pool", mint_a, mint_b, fee_rate_le]      Pool state (price, liquidity, fees)
Pool Authority     ["pool_authority", pool]                     Signs vault transfers
Vault A / B        ATA(pool_authority, mint_a/b)                Hold deposited tokens
TickArray PDA      ["tick_array", pool, start_tick_le]          64 ticks + u64 bitmap (zero-copy)
Position PDA       ["position", pool, owner, lower_le, upper_le]  Per-LP range position
```

## Tick System

```
P(i) = 1.0001^i                         price at tick i
sqrt_price(i) = 1.0001^(i/2) * 2^64     Q64.64 fixed-point

MIN_TICK = -443636,  MAX_TICK = 443636
```

Tick spacing per fee tier:

| Fee Rate | BPS | Tick Spacing |
|---|---|---|
| 100 | 0.01% | 1 |
| 500 | 0.05% | 10 |
| 3000 | 0.30% | 60 |
| 10000 | 1.00% | 200 |

Each tick array holds 64 ticks with a u64 bitmap tracking which are initialized. Tick arrays are zero-copy (`#[account(zero_copy(unsafe))]`) to avoid the BPF 4KB stack limit.

## Math (Q64.64 Fixed-Point)

All prices are stored as `sqrt_price` in Q64.64 format (u128): 64 integer bits, 64 fractional bits. `Q64 = 1u128 << 64` represents 1.0.

### Swap Step (within one tick range)

```
a_to_b (price decreasing):
  amount_a_in  = L * (1/sqrt_P_target - 1/sqrt_P_current)    round UP
  amount_b_out = L * (sqrt_P_current - sqrt_P_target)         round DOWN

b_to_a (price increasing):
  amount_b_in  = L * (sqrt_P_target - sqrt_P_current)         round UP
  amount_a_out = L * (1/sqrt_P_current - 1/sqrt_P_target)     round DOWN
```

### Liquidity ↔ Token Amounts

```
Position at [tick_lower, tick_upper], current price P:

In range (P_lower <= P <= P_upper):
  amount_a = L * (1/sqrt_P - 1/sqrt_P_upper)
  amount_b = L * (sqrt_P - sqrt_P_lower)

Below range (P < P_lower):  all token A
  amount_a = L * (1/sqrt_P_lower - 1/sqrt_P_upper)

Above range (P > P_upper):  all token B
  amount_b = L * (sqrt_P_upper - sqrt_P_lower)
```

### Fee Tracking

```
Global:     fee_growth_global_{a,b}           cumulative fees per unit L (Q64.64)
Per tick:   fee_growth_outside_{a,b}          fees on the "other side" of this tick
Per position: fee_growth_inside_last_{a,b}    snapshot at last update

fee_growth_inside = global - below(lower) - above(upper)
fees_owed = L * (fee_growth_inside - fee_growth_inside_last) / Q64
```

Wrapping subtraction is intentional (same as Uniswap V3) to handle global fee counter overflow.

## Key Design Decisions

| Decision | Choice | Rationale |
|---|---|---|
| Fixed-point | Q64.64 in u128 | Uniswap V3 / Orca standard, covers full tick range |
| U256 intermediate | Manual `(u128, u128)` hi/lo pairs | No external bignum dependency |
| Ticks per array | 64 | Power of 2 for clean bitmap (u64), ~4.1KB per array |
| Tick bitmap | u64 per TickArray (inline) | No global bitmap account needed |
| Swap tick arrays | `remaining_accounts` (up to 3) | 192 tick slots per swap, client precomputes |
| Max tick crossings | 20 per swap | ~300-400K CU budget, real swaps rarely cross >5 |
| TickArray layout | zero-copy `repr(C)` | Avoids BPF stack overflow (4KB+ struct) |
| Swap tick access | Raw account data reads | Avoids AccountLoader overhead in hot loop |
| Position model | PDA (not NFT) | Simpler, same user gets multiple via tick range in seeds |
| Fee tiers | {100, 500, 3000, 10000} | Industry standard Uniswap V3 tiers |
| Protocol fee | 10% of swap fees (configurable) | Accrues in pool, admin collectible later |
| Rounding | Always favor protocol | UP for inputs, DOWN for outputs/withdrawals |

## Build & Test

```bash
anchor build
cd tests-litesvm && cargo test
```

20 tests covering pool init (3), tick arrays (2), positions (2), liquidity management (4), swaps in both directions including tick crossings and zero-liquidity gaps (6), fee collection and distribution (2), and a full lifecycle test.

## Project Layout

```
programs/kclmm/src/
  lib.rs                8 instruction dispatchers
  constants.rs          PDA seeds, Q64, tick bounds, fee tiers, tick spacing
  errors.rs             KclmmError enum (~27 variants)
  math.rs               U256 helpers, Q64.64 arithmetic, tick<->price, swap step,
                        liquidity<->amounts, fee growth, bitmap helpers
  state.rs              Pool, TickArray (zero-copy), Tick, Position
  instructions/
    init_pool.rs        Pool + authority + vaults creation
    init_tick_array.rs  Lazy tick array init (zero-copy)
    open_position.rs    Empty position PDA creation
    add_liquidity.rs    Deposit tokens, update ticks, increase L
    remove_liquidity.rs Burn tokens, update ticks, decrease L
    collect_fees.rs     Claim accumulated trading fees
    swap.rs             Tick traversal loop with raw data access
    close_position.rs   Close empty position, reclaim rent
tests-litesvm/src/
  lib.rs                20 LiteSVM integration tests
```
