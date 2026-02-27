# solana-defi

DeFi protocol implementations on Solana. Built with Anchor 0.32.1 and tested with LiteSVM 0.7.

## Protocols

### kpool — Constant Product AMM

Uniswap V2-style automated market maker. Liquidity providers deposit token pairs, traders swap against the pool, and fees accrue to LPs through the constant product invariant (`x * y = k`).

**Instructions**: `initialize_pool`, `add_liquidity`, `swap`, `remove_liquidity`

```bash
cd kpool && anchor build
cd kpool/tests-litesvm && cargo test
```

### klend — Lending/Borrowing

Aave V2/Compound V2-style lending protocol. Users deposit collateral, borrow against it, earn interest from borrowers, and face liquidation if undercollateralized.

- Kinked interest rate model (base + slope1/slope2 around optimal utilization)
- Compound cToken-style exchange rates with virtual shares for inflation attack defense
- Health factor checks on borrow/withdraw, liquidation with 50% close factor and 5% bonus
- Mock oracle for testing (swappable for Pyth/Switchboard in production)

**Instructions**: `init_market`, `init_mock_oracle`, `update_mock_oracle`, `init_reserve`, `refresh_reserve`, `deposit`, `withdraw`, `borrow`, `repay`, `liquidate`

```bash
cd klend && anchor build
cd klend/tests-litesvm && cargo test
```

### kclmm — Concentrated Liquidity AMM

Uniswap V3/Orca Whirlpool-style concentrated liquidity AMM. LPs provide liquidity in custom price ranges for dramatically higher capital efficiency. Swaps traverse tick boundaries, updating active liquidity at each crossing.

- Q64.64 fixed-point arithmetic with manual U256 intermediates
- Tick system: `P(i) = 1.0001^i`, binary exponentiation with precomputed table
- Zero-copy tick arrays (64 ticks each, u64 bitmap for initialized tracking)
- Swap loop with tick traversal via `remaining_accounts` (up to 3 arrays, max 20 crossings)
- Per-position fee tracking with wrapping subtraction (Uniswap V3 pattern)
- Fee tiers: 0.01%, 0.05%, 0.30%, 1.00% with corresponding tick spacings
- Protocol fee: 10% of swap fees

**Instructions**: `init_pool`, `init_tick_array`, `open_position`, `add_liquidity`, `remove_liquidity`, `collect_fees`, `swap`, `close_position`

```bash
cd kclmm && anchor build
cd kclmm/tests-litesvm && cargo test
```

### krouter — DEX Aggregator/Router

Jupiter-style stateless DEX router that composes kpool and kclmm. Routes swaps through the best pool, chains multi-hop swaps (A→B→C), and splits input across pools to minimize price impact. All routing decisions happen off-chain; the on-chain program executes pre-computed routes via CPI and enforces end-to-end slippage protection.

- Direct swaps through kpool or kclmm via CPI
- Two-hop routing through any combination of pool types (4 combinations)
- Split routing across two pools for the same pair
- `remaining_accounts`-based leg encoding with `LegDescriptor` per leg
- Balance reload pattern for measuring intermediate/final output amounts
- Single slippage check at the router level (underlying programs get `min_out = 0`)

**Instructions**: `swap_kpool`, `swap_kclmm`, `route_two_hop`, `route_split`

```bash
cd krouter && anchor build
cd krouter/tests-litesvm && cargo test
```

### klev — Leveraged Yield Vault

Kamino Multiply-style leveraged yield vault that composes klend and kpool. Users deposit SOL and receive share tokens. An admin loops: deposit SOL into klend as collateral → borrow USDC → swap USDC→SOL via kpool → deposit SOL back, amplifying SOL exposure and supply yield by 2-3x.

- CPI into both klend (deposit/borrow/withdraw/repay) and cpamm (swap) in a single instruction
- ERC-4626 share math with virtual offset, dilutive fee share minting
- Leverage ratio and max leverage safety checks
- Cached net equity (collateral − debt in collateral terms) updated at harvest via oracle reads
- Performance fee on yield + time-weighted management fee on total assets

**Instructions**: `init_vault`, `deposit`, `withdraw`, `leverage_up`, `deleverage`, `harvest`, `set_halt`

```bash
cd klend && anchor build          # klend must be built first (CPI dependency)
cd kpool && anchor build          # kpool must be built first (CPI dependency)
cd klev && anchor build
cd klev/tests-litesvm && cargo test
```

### kvault — Yield Vault

ERC-4626/Yearn V3-style yield vault. Users deposit USDC and receive fungible SPL share tokens. An admin allocates idle funds into klend via CPI to earn lending interest. Yield is harvested periodically, and performance + management fees are extracted through dilutive share minting.

- ERC-4626 share math with virtual offset for inflation attack defense
- Dilutive fee share minting (Yearn V3 pattern) — no token transfers needed for fees
- CPI into klend for deposit/withdraw of funds
- Emergency halt toggle (blocks deposits, withdrawals always allowed)
- Cached `total_invested` updated from klend state on harvest

**Instructions**: `init_vault`, `deposit`, `withdraw`, `allocate`, `deallocate`, `harvest`, `set_halt`

```bash
cd klend && anchor build          # klend must be built first (CPI dependency)
cd kvault && anchor build
cd kvault/tests-litesvm && cargo test
```

### kusd — CDP Stablecoin

MakerDAO/Liquity-style CDP (collateralized debt position) stablecoin. Users deposit SOL collateral, mint kUSD stablecoins against it (up to a configurable max LTV), and face liquidation if undercollateralized. kUSD is minted from nothing and burned on repayment — no lending pool involved.

- Self-contained program with its own MockOracle (no CPI dependencies)
- Debt tracked as shares with cumulative fee index for stability fee accrual
- kUSD = $1 with 6 decimals (PRICE_SCALE and token decimals cancel, simplifying all math)
- Health factor, 50% close factor, liquidation bonus
- Configurable per-vault: max LTV, liquidation threshold, bonus, stability fee, debt ceiling
- Admin halt toggle (blocks minting; repay/withdraw/liquidate always allowed)

**Instructions**: `init_mock_oracle`, `update_mock_oracle`, `init_vault`, `open_position`, `deposit_collateral`, `mint_kusd`, `repay_kusd`, `withdraw_collateral`, `liquidate`, `accrue_fees`, `set_halt`

```bash
cd kusd && anchor build
cd kusd/tests-litesvm && cargo test
```

## Research

The `research/` directory contains 9 deep-dive documents (~74,000 words) covering DeFi fundamentals, AMMs, lending, yield sources, stablecoins, the Solana ecosystem, key papers and math, protocol design patterns, and advanced strategies.

## Stack

- **Anchor** 0.32.1
- **LiteSVM** 0.7 (standalone test crates, excluded from workspace)
- **Solana SDK** 2.x / SPL Token 7.x
