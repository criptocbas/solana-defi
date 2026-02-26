# DeFi Protocol Design & Architecture: A Builder's Deep Reference

> A comprehensive engineering guide for experienced Solana developers building DeFi protocols.
> Covers architecture patterns, oracle design, liquidation engines, risk management,
> governance, fee design, tokenomics, and security best practices.

---

## Table of Contents

1. [DeFi Protocol Architecture Patterns](#1-defi-protocol-architecture-patterns)
2. [Oracle Design and Integration](#2-oracle-design-and-integration)
3. [Liquidation Engine Design](#3-liquidation-engine-design)
4. [Risk Management in Protocol Design](#4-risk-management-in-protocol-design)
5. [Governance and Upgradeability](#5-governance-and-upgradeability)
6. [Fee Design and Tokenomics](#6-fee-design-and-tokenomics)
7. [Security Best Practices for DeFi Development](#7-security-best-practices-for-defi-development)
8. [References](#8-references)

---

## 1. DeFi Protocol Architecture Patterns

### 1.1 Pool-Based Architectures

Pool-based architectures are the foundational building block of DeFi. Instead of matching individual buyers and sellers (order book model), protocols aggregate assets into shared pools that users interact with programmatically.

#### Automated Market Maker (AMM) Pools

The canonical example is the **constant product AMM** (Uniswap V2):

```
x * y = k

where:
  x = reserve of token A
  y = reserve of token B
  k = invariant (constant product)
```

When a trader swaps `dx` of token A for token B:
```
(x + dx) * (y - dy) = k
dy = y - k / (x + dx)
dy = y * dx / (x + dx)      // simplified: output amount
```

The price impact increases with trade size relative to pool reserves -- this is the fundamental property that makes AMMs work. The curve `x * y = k` is a hyperbola: you can never fully drain either side of the pool, because prices go to infinity as reserves approach zero.

**Concentrated Liquidity (Uniswap V3)**

Uniswap V3 introduced concentrated liquidity, where LPs allocate capital within custom price ranges (ticks). This dramatically improves capital efficiency:

```
Tick price:  p(i) = 1.0001^i
  - Each tick = 0.01% price change
  - Tick spacing varies by fee tier:
    0.05% fee -> tickSpacing = 10  (~0.10% between initialized ticks)
    0.30% fee -> tickSpacing = 60  (~0.60%)
    1.00% fee -> tickSpacing = 200 (~2.02%)
```

The pool tracks a `current tick` and traverses ticks sequentially during swaps. Only liquidity from positions covering the current price is active. When the price crosses a tick boundary, the contract updates the active liquidity by adding or removing the liquidity of positions that start or end at that tick.

**Alternative AMM Curves:**
- **Constant Mean (Balancer):** Generalizes to multi-asset pools with weighted reserves. `product(x_i^w_i) = k` where `w_i` are weights.
- **StableSwap (Curve):** A hybrid curve between constant-product and constant-sum, optimized for assets that should trade near 1:1 (stablecoins, LSTs). Minimizes slippage around the peg while maintaining the safety properties of constant-product at extremes.

#### Lending Pool Architecture

Lending protocols pool deposited assets into shared liquidity pools. The core components are:

1. **Lending Pool Contract** -- Holds deposited assets, manages deposits/withdrawals/borrows/repayments
2. **Interest Rate Model** -- Computes borrow and supply rates as a function of utilization
3. **Receipt/Share Token** -- Represents depositor's claim on pool assets + accrued interest
4. **Oracle Integration** -- Provides asset prices for collateral valuation and liquidation triggers
5. **Liquidation Module** -- Handles undercollateralized position resolution

```
Utilization Rate = Total Borrows / (Cash + Total Borrows - Reserves)

// Jump Rate Model (Compound-style):
if utilization <= kink:
    borrow_rate = base_rate + utilization * multiplier
else:
    borrow_rate = base_rate + kink * multiplier
                  + (utilization - kink) * jump_multiplier

supply_rate = borrow_rate * utilization * (1 - reserve_factor)
```

The "kink" creates a sharp rate increase at high utilization (typically 80-90%), strongly incentivizing new deposits and discouraging further borrowing when liquidity is scarce. This is crucial for ensuring depositors can withdraw.

### 1.2 Vault Patterns (Deposit -> Receipt Token -> Yield -> Withdraw)

The vault pattern is ubiquitous in DeFi: a contract accepts deposits of an underlying asset, issues "share" tokens representing proportional ownership, deploys the underlying into some yield strategy, and allows redemption of shares for the underlying plus accrued yield.

#### The ERC-4626 Standard (and Solana Equivalents)

ERC-4626 is the canonical interface on Ethereum for tokenized vaults. On Solana, the same economic model is implemented using SPL tokens and PDAs, though no formal standard exists yet:

```
Core Lifecycle:
  deposit(assets, receiver)  -> mint shares to receiver
  mint(shares, receiver)     -> pull assets to mint exact shares
  withdraw(assets, owner)    -> burn shares, return assets
  redeem(shares, owner)      -> burn exact shares, return assets

View Functions:
  totalAssets()              -> total underlying managed by vault
  convertToShares(assets)    -> preview deposit
  convertToAssets(shares)    -> preview redeem
  maxDeposit/maxMint/maxWithdraw/maxRedeem  -> limits
```

#### Solana Vault Architecture with Anchor

On Solana, the vault pattern maps to PDAs and SPL token accounts:

```rust
// Account structure for a Solana vault
#[account]
pub struct VaultState {
    pub authority: Pubkey,           // Vault admin
    pub underlying_mint: Pubkey,     // The asset being deposited
    pub share_mint: Pubkey,          // Receipt token mint (PDA-controlled)
    pub vault_token_account: Pubkey, // PDA-owned token account holding underlying
    pub total_assets: u64,           // Tracked total (may differ from balance)
    pub bump: u8,                    // PDA bump seed
}

// PDA seeds for deterministic derivation:
// Vault state:   ["vault", underlying_mint]
// Share mint:    ["share_mint", vault_state]
// Token account: ["vault_tokens", vault_state]

// Deposit instruction pseudocode:
pub fn deposit(ctx: Context<Deposit>, assets: u64) -> Result<()> {
    let vault = &mut ctx.accounts.vault_state;
    let total_supply = get_share_supply(&ctx.accounts.share_mint);

    // Calculate shares to mint
    let shares = if total_supply == 0 {
        assets  // 1:1 for first deposit
    } else {
        // shares = assets * total_supply / total_assets
        assets.checked_mul(total_supply)?
              .checked_div(vault.total_assets)?
    };

    // Transfer underlying tokens to vault
    transfer_tokens(
        &ctx.accounts.depositor_token_account,
        &ctx.accounts.vault_token_account,
        assets,
    )?;

    // Mint share tokens to depositor
    mint_shares(&ctx.accounts.share_mint, &ctx.accounts.depositor_share_account, shares)?;

    vault.total_assets = vault.total_assets.checked_add(assets)?;
    Ok(())
}
```

### 1.3 Share/Receipt Token Accounting Models

There are two primary approaches to representing a depositor's claim on pool assets:

#### Model A: Exchange-Rate Shares (Compound cTokens)

The share token's *quantity* stays constant after deposit. Value accrues through an increasing **exchange rate**.

```
Exchange Rate = (Total Cash + Total Borrows - Reserves) / Total cToken Supply

Deposit:  cTokens_minted = underlying_amount / exchange_rate
Withdraw: underlying_received = cTokens_redeemed * exchange_rate
```

Example flow:
```
Time 0: Exchange rate = 0.02 (1 cDAI = 0.02 DAI)
  - Alice deposits 100 DAI -> receives 5,000 cDAI

Time 1: Interest accrues, exchange rate = 0.021
  - Alice's 5,000 cDAI now worth 5,000 * 0.021 = 105 DAI
  - Alice redeems 5,000 cDAI -> receives 105 DAI
```

**Advantage:** Simple balance tracking; your cToken balance never changes unless you explicitly transfer.
**Disadvantage:** Requires careful decimal handling. The exchange rate scaling is: `oneCTokenInUnderlying = exchangeRate / (1 * 10^(18 + underlyingDecimals - cTokenDecimals))`.

#### Model B: Rebasing Shares (Aave aTokens)

The share token's *quantity* increases over time to match accumulated interest. 1 aToken always equals approximately 1 underlying token.

```
aToken balance = principal * cumulativeLiquidityIndex / userLiquidityIndex

Where:
  cumulativeLiquidityIndex grows as interest accrues
  userLiquidityIndex is snapshot at deposit time
```

Example flow:
```
Time 0: Alice deposits 100 DAI -> receives 100 aDAI
Time 1: Interest accrues
  - Alice's aDAI balance now shows 105 aDAI
  - She redeems 105 aDAI -> receives 105 DAI
```

**Advantage:** Intuitive -- balance directly shows claimable amount.
**Disadvantage:** Rebasing tokens are harder to integrate (wrapped versions needed for many protocols), and create complexity in fee/tax accounting.

#### Model C: LP Tokens (AMM Pools)

LP tokens represent proportional ownership of both assets in a pool:

```
LP_minted = min(
    amount_a * total_lp / reserve_a,
    amount_b * total_lp / reserve_b
)

// On redemption:
amount_a_out = lp_burned * reserve_a / total_lp
amount_b_out = lp_burned * reserve_b / total_lp
```

### 1.4 The Math of Share-Based Accounting

The fundamental equation for share-based vaults:

```
shares_per_asset = total_shares / total_assets
assets_per_share = total_assets / total_shares  (the "exchange rate")

// Deposit: convert assets to shares
shares_out = assets_in * total_shares / total_assets
  -> Round DOWN (fewer shares to depositor = favor existing shareholders)

// Withdraw: convert shares to assets
assets_out = shares_in * total_assets / total_shares
  -> Round DOWN (fewer assets to withdrawer = favor remaining shareholders)
```

**Critical: Rounding Direction**

Always round *against* the user performing the action and *in favor of* the vault/existing shareholders. This prevents value extraction through rounding.

```
// CORRECT: Round down shares on deposit (user gets fewer shares)
shares = assets * totalSupply / totalAssets;  // integer division rounds down

// CORRECT: Round down assets on withdraw (user gets fewer assets)
assets = shares * totalAssets / totalSupply;  // integer division rounds down

// For mint (user specifies shares, calculates assets needed):
assets = shares * totalAssets / totalSupply;  // Round UP (user pays more)
// In practice: assets = (shares * totalAssets + totalSupply - 1) / totalSupply;
```

**The First-Depositor / Inflation Attack**

This is one of the most critical vulnerabilities in vault design. The attack works as follows:

1. Attacker deposits 1 wei of underlying -> receives 1 share
2. Attacker donates (transfers directly) a large amount (e.g., 1e18 tokens) to the vault
3. Now: `totalAssets = 1e18 + 1`, `totalShares = 1`
4. Victim deposits `X` tokens. Shares = `X * 1 / (1e18 + 1)` -> rounds to 0 if `X < 1e18 + 1`
5. Victim gets 0 shares; attacker redeems their 1 share and gets everything

**Defenses:**

```rust
// Defense 1: Virtual shares and assets (OpenZeppelin approach)
// Add a virtual offset to both shares and assets
const VIRTUAL_SHARES: u64 = 1e6;  // "dead shares"
const VIRTUAL_ASSETS: u64 = 1;

fn convert_to_shares(assets: u64, total_assets: u64, total_shares: u64) -> u64 {
    let effective_shares = total_shares + VIRTUAL_SHARES;
    let effective_assets = total_assets + VIRTUAL_ASSETS;
    assets * effective_shares / effective_assets
}

// Defense 2: Seed initial liquidity (Morpho approach)
// On vault creation, deposit minimum amount and mint shares to dead address
// This establishes a reasonable exchange rate from the start

// Defense 3: Minimum deposit requirement
const MIN_DEPOSIT: u64 = 1_000_000; // 1e6 in smallest units
require!(assets >= MIN_DEPOSIT, "Deposit too small");
```

### 1.5 Modular Vault Architectures (Euler V2 Pattern)

The trend in 2024-2025 is toward modular, composable vault systems. Euler V2 pioneered this with two key components:

- **Euler Vault Kit (EVK):** A framework for deploying customized, ERC-4626-based lending vaults. Each vault handles a single asset.
- **Ethereum Vault Connector (EVC):** An immutable primitive that allows vaults to use each other as collateral, creating flexible lending markets.

**Vault Classes in Euler V2:**
1. **Core Vaults** -- Governed lending products with curated risk parameters
2. **Edge Vaults** -- Permissionless, ungoverned vaults for long-tail asset markets
3. **Escrow Vaults** -- Enable any ERC-20 as collateral without requiring yield

The key insight: instead of one monolithic lending pool (Aave/Compound style), you create many small, specialized vaults that can reference each other. This solves the tension between:
- **Shared pools** (capital efficient but risk-coupled) vs.
- **Isolated pools** (safe but capital-fragmented)

### 1.6 Solana-Specific Architecture Patterns

Solana's account model requires different architectural thinking than EVM:

**Program-Derived Addresses (PDAs) as Vault Authorities:**
```rust
// PDA seeds for a lending pool
seeds = ["pool", token_mint.key().as_ref()]
bump = pool_state.bump

// PDA seeds for user position
seeds = ["position", pool.key().as_ref(), user.key().as_ref()]

// PDA signs for token transfers via invoke_signed
// No private key exists -- only the program can authorize
```

**Account Structure for a Lending Protocol:**
```
Program
  |-- PoolState (PDA: ["pool", mint])
  |     |-- underlying_mint: Pubkey
  |     |-- share_mint: Pubkey (PDA-controlled)
  |     |-- vault_token_account: Pubkey (PDA-owned)
  |     |-- total_deposits: u64
  |     |-- total_borrows: u64
  |     |-- interest_rate_model: InterestRateConfig
  |     |-- oracle: Pubkey
  |     |-- last_update_slot: u64
  |
  |-- UserPosition (PDA: ["position", pool, user])
  |     |-- owner: Pubkey
  |     |-- deposited_shares: u64
  |     |-- borrowed_amount: u64
  |     |-- borrow_index_snapshot: u128
  |
  |-- GlobalConfig (PDA: ["config"])
        |-- admin: Pubkey
        |-- fee_receiver: Pubkey
        |-- emergency_authority: Pubkey
```

**Cross-Program Invocations (CPIs):** Every DeFi protocol on Solana uses CPIs extensively -- for token transfers (SPL Token program), oracle reads (Pyth/Switchboard), and composability with other protocols.

---

## 2. Oracle Design and Integration

### 2.1 Why DeFi Needs Price Oracles

Price oracles are the bridge between on-chain protocols and real-world asset prices. They are required for:

- **Collateral valuation** in lending protocols (is this position safe?)
- **Liquidation triggers** (has the health factor dropped below 1?)
- **Synthetic asset pricing** (what should this synthetic track?)
- **DEX reference prices** (TWAP oracles for manipulation resistance)
- **Insurance/options pricing** (settlement prices for derivatives)

Without accurate price data, a lending protocol cannot determine if a loan is undercollateralized, cannot trigger liquidations, and will inevitably accumulate bad debt. **Oracles are the single most critical external dependency in any DeFi protocol.**

### 2.2 Oracle Types

#### On-Chain Oracles: TWAP (Time-Weighted Average Price)

TWAP oracles derive prices from on-chain DEX trading activity:

```
TWAP = sum(price_i * time_i) / sum(time_i)

// Uniswap V2/V3 TWAP implementation:
// Each pool accumulates price*time in a cumulative variable
// To compute TWAP over interval [t1, t2]:
twap = (priceCumulative_t2 - priceCumulative_t1) / (t2 - t1)
```

**Properties:**
- Fully on-chain, no external trust assumption
- Manipulation-resistant over longer windows (30 min+)
- **Lag**: TWAP inherently lags spot price -- a 30-minute TWAP reflects the average of the last 30 minutes, not the current price
- **Manipulation cost**: Proportional to `pool_liquidity * time_window` -- manipulating a deep pool's 30-min TWAP requires sustaining a skewed price across many blocks, which is prohibitively expensive for liquid pairs
- **Weakness**: For illiquid pairs, TWAP can be manipulated with moderate capital. Multi-block MEV (proposer controlling consecutive blocks) can make manipulation easier

```
Cost to manipulate TWAP ~ liquidity_in_pool * manipulation_window * fee_rate
```

#### Off-Chain Oracles: Push Model (Chainlink)

Chainlink aggregates prices from multiple off-chain data providers:

1. Multiple independent node operators fetch prices from exchanges
2. Nodes submit on-chain, and the aggregator computes the median
3. On-chain price updates when deviation threshold or heartbeat is reached
4. Protocols read the latest aggregated price

**Deviation triggers:** Typically 0.5-1% for major assets. Price updates only when the price changes by more than the threshold OR a heartbeat timer expires (e.g., every hour).

**Trust model:** You trust the set of Chainlink node operators and the aggregation mechanism.

#### Off-Chain Oracles: Pull Model (Pyth Network)

Pyth is the dominant oracle on Solana, using a "pull" architecture:

```
Architecture:
  1. 95+ publishers (exchanges, market makers, trading firms) submit prices
  2. Prices aggregated on Pythnet (Solana-fork appchain) every 400ms
  3. Aggregated prices posted to Wormhole for cross-chain availability
  4. DeFi protocols "pull" prices on-chain only when needed via CPI

Key Innovation: Pull vs Push
  PUSH (old model): Oracle pays gas to update on-chain every N seconds
    -> Expensive, limits scalability, stale between updates
  PULL (Pyth model): User/protocol fetches latest price in their transaction
    -> No oracle gas costs, always fresh data, supports 500+ feeds

Pyth Price Feed Structure (Solana accounts):
  - Product Account: Asset metadata (symbol, asset type, etc.)
  - Price Account: Current price, confidence interval, timestamp
  - Mapping Account: Links products to prices

// Reading Pyth in Anchor:
use pyth_solana_receiver_sdk::price_update::PriceUpdateV2;

pub fn use_price(ctx: Context<UsePrice>) -> Result<()> {
    let price_update = &ctx.accounts.price_update;
    let price = price_update.get_price_no_older_than(
        &Clock::get()?,
        60,  // max staleness in seconds
        &price_feed_id,
    )?;

    // price.price: i64 (the price)
    // price.conf: u64 (confidence interval)
    // price.expo: i32 (exponent, e.g., -8 means divide by 1e8)
    // Actual price = price.price * 10^price.expo

    let price_in_usd = price.price as u128 * PRECISION
                        / 10u128.pow((-price.expo) as u32);
    Ok(())
}
```

**Confidence Intervals:** Pyth uniquely provides a confidence interval with each price, reflecting publisher disagreement. A wide confidence interval signals uncertainty -- protocols should handle this (e.g., use the pessimistic bound for collateral valuation).

#### Switchboard Oracle

Switchboard is the other major Solana oracle, with key differentiators:

- **Permissionless feeds:** Anyone can create a data feed without DAO approval
- **TEE architecture:** Uses Trusted Execution Environments for data verification
- **Switchboard Surge:** Ultra-low latency (2-5ms) oracle updates
- **Data diversity:** Supports custom data sources beyond just price feeds
- **Standard oracles:** ~400ms latency

**Comparison for Solana builders:**
| Feature | Pyth | Switchboard |
|---------|------|-------------|
| Latency | 400ms-1.4s (via Wormhole) | 2-5ms (Surge), 400ms (standard) |
| Feed creation | Permissioned (DAO approval) | Permissionless |
| Architecture | Pythnet + Wormhole | TEE-based |
| TVS (Total Value Secured) | ~$2.3B | ~$1.2B |
| Confidence intervals | Yes | Via custom logic |
| Best for | Major assets, cross-chain | Custom feeds, low-latency |

### 2.3 Oracle Manipulation Attacks and Defenses

Oracle manipulation was responsible for over 49% of DeFi price manipulation losses in 2023. Attack vectors:

#### Attack: Spot Price Manipulation
```
Attacker's strategy:
1. Flash loan a large amount of token A
2. Swap into token B on the DEX -> spikes token B price on that DEX
3. Use inflated price to borrow against token B on lending protocol
   (if protocol uses that DEX's spot price as oracle)
4. Default on loan, keep borrowed assets
5. Repay flash loan
```

#### Attack: TWAP Manipulation
```
Longer-term attack (multi-block):
1. Over N blocks, maintain skewed price on DEX
2. Cost ~ pool_depth * N_blocks * block_time
3. If protocol uses short TWAP window, cost may be affordable
4. Multi-block MEV: validator controlling consecutive blocks
   can manipulate TWAP more cheaply
```

#### Defense Strategies

**1. Multiple Oracle Sources:**
```rust
// Use multiple independent price sources and take the median
fn get_price(oracles: &[OraclePrice]) -> Result<Price> {
    let mut prices: Vec<i64> = oracles.iter()
        .filter(|o| !o.is_stale(MAX_STALENESS))
        .map(|o| o.price)
        .collect();

    require!(prices.len() >= MIN_ORACLE_COUNT, "Insufficient oracles");
    prices.sort();
    Ok(prices[prices.len() / 2])  // median
}
```

**2. Staleness Checks:**
```rust
// Never use a price older than N seconds
let price = oracle.get_price()?;
let age = clock.unix_timestamp - price.publish_time;
require!(age <= MAX_PRICE_AGE_SECONDS, OracleError::StalePrice);
```

**3. Confidence Interval Checks (Pyth-specific):**
```rust
// Reject prices with high uncertainty
let price = pyth_price.get_price()?;
let conf_ratio = price.conf as f64 / price.price.abs() as f64;
require!(conf_ratio < MAX_CONFIDENCE_RATIO, OracleError::PriceTooUncertain);

// Use pessimistic pricing for collateral:
// Collateral value: price - confidence (lower bound)
// Debt value: price + confidence (upper bound)
```

**4. Price Band / Circuit Breaker:**
```rust
// Reject prices that deviate too far from TWAP or last known good price
let deviation = (new_price - last_price).abs() * 10000 / last_price;
if deviation > MAX_DEVIATION_BPS {
    emit!(PriceCircuitBreaker { asset, deviation });
    return Err(OracleError::PriceDeviationTooHigh);
}
```

**5. TWAP with Sufficient Window:**
```rust
// Use 30+ minute TWAP for non-time-sensitive operations
// Use spot price with sanity checks for time-sensitive operations (liquidations)
let twap_30m = compute_twap(price_history, 30 * 60);
let spot = get_spot_price()?;

// Cross-check: spot shouldn't deviate too far from TWAP
let deviation = (spot - twap_30m).abs() * 10000 / twap_30m;
require!(deviation < MAX_SPOT_TWAP_DEVIATION, OracleError::Anomaly);
```

**6. Fallback Oracle Logic:**
```rust
fn get_safe_price(primary: &Oracle, fallback: &Oracle) -> Result<Price> {
    match primary.get_price() {
        Ok(price) if !price.is_stale() && price.conf_ok() => Ok(price),
        _ => {
            // Primary failed or stale -- use fallback
            let fb_price = fallback.get_price()?;
            require!(!fb_price.is_stale(), "Both oracles stale");
            Ok(fb_price)
        }
    }
}
```

---

## 3. Liquidation Engine Design

### 3.1 How Liquidation Systems Work

Liquidation is the process of selling a borrower's collateral to repay their debt when the position becomes undercollateralized. It is the primary mechanism that keeps lending protocols solvent.

**Health Factor:**
```
Health Factor = (Collateral Value * Liquidation Threshold) / Total Debt Value

if Health Factor < 1.0 -> position is liquidatable

Example:
  Collateral: 10 ETH at $2,000 = $20,000
  Liquidation Threshold: 82.5%
  Borrow: $15,000 USDC

  Health Factor = ($20,000 * 0.825) / $15,000 = 1.10 (safe)

  If ETH drops to $1,800:
  Health Factor = ($18,000 * 0.825) / $15,000 = 0.99 (liquidatable!)
```

### 3.2 Liquidation Mechanism Types

#### Fixed-Price Liquidation (Aave/Compound Style)

The simplest approach: a liquidator repays some of the borrower's debt and receives collateral at a discount (the "liquidation bonus").

```
Liquidation Parameters:
  - Close Factor: 50% (max debt repayable in one liquidation)
  - Liquidation Bonus: 5-10% (discount on collateral for liquidator)

Liquidator's Action:
  1. Identify undercollateralized position (health factor < 1.0)
  2. Repay up to close_factor * total_debt of borrower's debt
  3. Receive collateral worth (debt_repaid * (1 + liquidation_bonus))

Liquidator's Profit:
  profit = collateral_received - debt_repaid
         = debt_repaid * liquidation_bonus
         - gas_costs
         - slippage (if selling collateral)
```

**Partial Liquidation:** The close factor (typically 50%) means only a portion of the position is liquidated at once. This gives the borrower a chance to add collateral, and prevents unnecessary full liquidations that would cause excessive market impact.

```rust
// Pseudocode for fixed-price liquidation
pub fn liquidate(
    ctx: Context<Liquidate>,
    debt_to_repay: u64,
) -> Result<()> {
    let position = &ctx.accounts.borrower_position;
    let health_factor = calculate_health_factor(position, &ctx.accounts.oracle)?;

    require!(health_factor < HEALTH_FACTOR_ONE, "Position is healthy");

    // Enforce close factor
    let max_repayable = position.total_debt * CLOSE_FACTOR / PRECISION;
    let repay_amount = debt_to_repay.min(max_repayable);

    // Calculate collateral to seize (with bonus)
    let collateral_price = get_oracle_price(&ctx.accounts.collateral_oracle)?;
    let debt_price = get_oracle_price(&ctx.accounts.debt_oracle)?;

    let collateral_to_seize = repay_amount
        * debt_price
        * (PRECISION + LIQUIDATION_BONUS)
        / (collateral_price * PRECISION);

    // Transfer debt tokens from liquidator to pool
    transfer_to_pool(repay_amount)?;

    // Transfer collateral from borrower to liquidator
    transfer_collateral(collateral_to_seize)?;

    // Update borrower's position
    position.total_debt -= repay_amount;
    position.collateral -= collateral_to_seize;

    Ok(())
}
```

#### Dutch Auction Liquidation (MakerDAO Liquidation 2.0)

Instead of a fixed discount, collateral is auctioned using a descending price:

```
Dutch Auction Mechanics:
  1. Liquidation triggered -> auction starts at a high price
  2. Price decreases over time according to a price curve
  3. Any participant can buy collateral at the current price
  4. Multiple partial fills allowed -- anyone who sends DAI receives
     proportional collateral at the current price
  5. Auction ends when all collateral is sold OR time expires

Price Function:
  price(t) = initial_price * decay_function(t)

  // Exponential decay example:
  price(t) = initial_price * (decay_factor ^ (t / step_duration))
```

**Advantages over fixed-price:**
- Price discovery: market determines the fair discount
- Lower liquidation penalties on average (efficient markets bid early)
- No fixed bonus to extract -- reduces unnecessary value extraction from borrowers
- Multiple participants can fill -- more decentralized

**Disadvantages:**
- More complex to implement
- Auction duration introduces delay (risk of further price decline)
- Requires sufficient liquidator participation

#### Soft Liquidation / LLAMMA (Curve crvUSD)

Curve's LLAMMA (Lending Liquidating AMM Algorithm) represents a paradigm shift:

```
LLAMMA Design:
  - Collateral is deposited into a Uniswap V3-style AMM with price "bands"
  - Users choose 4-50 bands when creating a loan
  - As collateral price falls, the AMM automatically converts
    collateral -> borrowed asset (soft liquidation)
  - As price rises, it converts back (de-liquidation)
  - Arbitrageurs drive the rebalancing by trading against the AMM

Band System:
  - Each band has an upper and lower price range
  - Collateral is distributed across selected bands
  - Liquidation happens per-band, not per-user
  - Multiple users can have collateral in the same band

Key Benefit: Gradual, continuous liquidation instead of discrete events
  -> Supports higher LTV ratios (better capital efficiency)
  -> Borrowers can recover if price bounces back (de-liquidation)
  -> No cliff-edge liquidation penalties
```

### 3.3 Liquidation Bot Architecture

Liquidation bots ("keepers") are essential infrastructure for protocol health. A well-designed protocol makes it easy and profitable to build bots:

```
Liquidation Bot Components:

1. POSITION INDEXER
   - Subscribe to on-chain events (deposits, borrows, repayments)
   - Maintain sorted data structure of positions by health factor
   - Pre-compute liquidation prices for each position

2. PRICE MONITOR
   - Multi-source price feeds (Pyth, Switchboard, DEX prices)
   - Sub-second update frequency
   - Cross-validate sources for anomaly detection

3. OPPORTUNITY DETECTOR
   - On each price update, check positions near liquidation threshold
   - Binary search in sorted position list for efficiency
   - Calculate expected profit = liquidation_bonus - gas - slippage

4. EXECUTION ENGINE
   - Build and simulate liquidation transaction
   - Calculate optimal repay amount (maximizing profit)
   - Route collateral sale through DEX aggregator
   - Submit transaction with appropriate priority fee

5. RISK MANAGEMENT
   - Capital allocation limits
   - Maximum exposure per asset
   - Slippage tolerance checks
   - Gas price limits
```

```
// Simplified Solana liquidation bot loop (pseudocode)
loop {
    // 1. Get latest prices
    let prices = fetch_prices_from_pyth().await?;

    // 2. Check all positions
    for position in positions.iter_by_health_factor() {
        let hf = compute_health_factor(position, &prices);

        if hf >= 1.0 { break; }  // sorted: rest are healthy

        // 3. Calculate optimal liquidation
        let (repay_amount, expected_profit) =
            calculate_optimal_liquidation(position, &prices)?;

        if expected_profit < MIN_PROFIT_THRESHOLD { continue; }

        // 4. Build and simulate transaction
        let tx = build_liquidation_tx(position, repay_amount)?;
        let sim = simulate_transaction(&tx).await?;

        if sim.success && sim.profit >= expected_profit * 0.9 {
            // 5. Submit with priority fee
            submit_transaction(tx, priority_fee).await?;
        }
    }

    sleep(Duration::from_millis(100)).await;
}
```

### 3.4 MEV-Resistant Liquidation Design

Liquidations are a prime target for MEV extraction (frontrunning, sandwiching). Strategies to mitigate:

**1. Chainlink Smart Value Recapture (SVR):**
Oracle updates are auctioned through Flashbots MEV-Share, and the MEV extracted from liquidations is returned to the protocol instead of going to validators.

**2. Commit-Reveal Liquidation:**
```
Phase 1 (Commit): Liquidator commits hash of liquidation parameters
Phase 2 (Reveal): After N blocks, liquidator reveals and executes
-> Prevents frontrunning because parameters are hidden during commit
-> Tradeoff: adds latency, position may worsen during delay
```

**3. Batch Liquidation Auctions:**
Instead of first-come-first-served, collect liquidation requests over a short window and process as a batch, allocating pro-rata or by sealed bid.

**4. Protocol-Owned Liquidation:**
The protocol itself acts as liquidator using reserves/insurance fund, capturing the liquidation bonus for the treasury instead of external MEV searchers. This can be combined with gradual Dutch auctions.

**5. MEV-Share / Programmable Order Flow:**
The MEV profit from liquidation is shared back to the protocol or the borrower, rather than being fully captured by validators/searchers.

---

## 4. Risk Management in Protocol Design

### 4.1 Collateral Factors and LTV Ratios

The Loan-to-Value (LTV) ratio and Liquidation Threshold are the two most critical risk parameters:

```
LTV (Loan-to-Value): Maximum borrowing power
  = Max Borrow / Collateral Value
  Example: 75% LTV -> $100 collateral allows $75 borrow

Liquidation Threshold: Point at which position becomes liquidatable
  = Debt Value / Collateral Value at which liquidation triggers
  Example: 82.5% threshold -> liquidation when debt > 82.5% of collateral

Buffer = Liquidation Threshold - LTV
  Example: 82.5% - 75% = 7.5% buffer
  This buffer absorbs price movements between user's max borrow and liquidation

Liquidation Penalty: Discount given to liquidators
  Example: 5% -> liquidator gets collateral at 5% below market price
```

**How to set these parameters:**

| Asset Property | Effect on LTV | Reasoning |
|---|---|---|
| High liquidity | Higher LTV | Can be liquidated with low slippage |
| Low volatility | Higher LTV | Less likely to gap past liquidation threshold |
| Large market cap | Higher LTV | More resistant to manipulation |
| Decentralized | Higher LTV | Lower tail risks |
| Strong oracle coverage | Higher LTV | Reliable price data for liquidations |

**Typical ranges by asset class:**
- Blue-chip (ETH, BTC): LTV 75-80%, Liquidation 82-85%
- Large-cap stablecoins: LTV 75-80%, Liquidation 85-90%
- Mid-cap tokens: LTV 50-65%, Liquidation 65-75%
- Long-tail / new assets: LTV 0-40%, Liquidation 50-65%

**Aave V3 E-Mode (Efficiency Mode):**

For correlated assets (e.g., stablecoins, ETH/stETH), dramatically higher parameters:
- Stablecoin E-Mode: LTV up to 97%, Liquidation Threshold 97.5%
- ETH-correlated E-Mode: LTV up to 93%, Liquidation Threshold 95%

This works because correlated assets are unlikely to diverge significantly, so the liquidation buffer can be much smaller. Users can only enter E-Mode if all their collateral and borrows are within the same category.

### 4.2 Risk Parameter Frameworks (Gauntlet / Chaos Labs)

Professional risk management in DeFi is performed by specialized firms:

#### Gauntlet's Approach
- **Agent-based simulation:** Thousands of simulated agents (borrowers, lenders, liquidators, arbitrageurs) interact under various market scenarios
- **Monte Carlo stress tests:** Random market scenarios generated from historical volatility distributions
- **Key metric: Value at Risk (VaR):** 95th percentile insolvency value across simulations
- **Parameter optimization:** Given a risk budget, find optimal LTV/threshold/cap parameters that maximize capital efficiency

#### Chaos Labs' Approach
- **Risk Oracles:** Automated, real-time risk parameter adjustment -- turns risk management from a governance process into automated infrastructure
- **Dynamic parameters:** Instead of fixed LTV ratios set by governance vote every few weeks, parameters adjust continuously based on market conditions
- **Stress-testing-as-a-service:** Continuous monitoring of protocol health under various scenarios

#### Building Your Own Risk Framework

For a new protocol, start with conservative parameters and tighten over time:

```
Step 1: Asset Classification
  - Tier 1 (blue chip): BTC, ETH, major stablecoins
  - Tier 2 (established): Top 20 market cap, proven liquidity
  - Tier 3 (emerging): Smaller cap, less liquid
  - Tier 4 (long tail): New, illiquid, concentrated ownership

Step 2: Volatility Analysis
  - Compute historical daily/hourly volatility
  - Compute max drawdown over 1h/4h/24h windows
  - Compute tail risk (99th percentile hourly moves)

Step 3: Liquidity Analysis
  - How much can be liquidated in one transaction with <2% slippage?
  - Across how many venues?
  - What's the 24h volume?

Step 4: Set Parameters
  - LTV < (1 - max_hourly_drawdown_99th - liquidation_penalty)
  - Liquidation Threshold = LTV + safety_buffer
  - Supply cap = f(market_cap, circulating_supply, on_chain_liquidity)
  - Borrow cap = supply_cap * target_max_utilization

Step 5: Continuous Monitoring
  - Track utilization, health factor distributions
  - Simulate cascade liquidation scenarios
  - Adjust parameters via governance as data accumulates
```

### 4.3 Supply Caps, Borrow Caps, and Why They Matter

**Supply Caps:** Maximum total deposit of an asset in the protocol.
```
Purpose:
  - Limit protocol exposure to any single asset
  - Prevent governance attacks (deposit massive amount, vote to change params)
  - Ensure sufficient liquidity diversity
  - Limit potential bad debt from a single asset collapse

Setting supply caps:
  supply_cap = min(
      market_cap * max_protocol_concentration,  // e.g., 1-5% of market cap
      on_chain_liquidity * liquidation_factor,   // enough liquidity to liquidate
      circulating_supply * max_supply_ratio       // e.g., 25% of circulating
  )
```

**Borrow Caps:** Maximum total borrows of an asset.
```
Purpose:
  - Prevent short squeezes (borrowing all supply to manipulate price)
  - Ensure depositors can withdraw (cap < supply * max_utilization)
  - Limit exposure to oracle manipulation of borrowed assets
  - Dampen procyclical behavior during booms

Setting borrow caps:
  borrow_cap = supply_cap * target_max_utilization  // e.g., 80%
```

**Aave V3 Isolation Mode:**

For newly listed, riskier assets:
```
Isolation Mode Constraints:
  - Asset can only be used as sole collateral (no mixing with other assets)
  - Can only borrow specific assets (usually stablecoins only)
  - Has a debt ceiling (max total USD value borrowable against this collateral)

Debt Ceiling:
  - Represents maximum protocol exposure to catastrophic devaluation
  - Set based on on-chain liquidity available for liquidation
  - Much lower than supply cap (e.g., $10M debt ceiling for $50M supply cap)

Graduation Path:
  Isolation Mode (restricted) -> Standard Mode (full features)
  As asset proves itself over time, governance can graduate it
```

### 4.4 Circuit Breakers and Emergency Shutdown

#### Circuit Breakers

Automated safety mechanisms that pause or restrict protocol operations when anomalous conditions are detected:

```rust
// Types of circuit breakers:

// 1. Price Circuit Breaker
// Halt operations if price moves too fast
pub fn check_price_circuit_breaker(
    new_price: u64,
    last_price: u64,
    time_elapsed: i64,
) -> Result<()> {
    let pct_change = (new_price as i128 - last_price as i128).abs()
                     * 10000 / last_price as i128;

    let max_change = match time_elapsed {
        0..=60 => 500,      // 5% per minute
        61..=3600 => 2000,  // 20% per hour
        _ => 5000,          // 50% per day
    };

    require!(pct_change <= max_change, "Price circuit breaker triggered");
    Ok(())
}

// 2. Volume Circuit Breaker
// Halt if withdrawal volume exceeds threshold
// ERC-7265 pattern: lock funds above threshold in separate vault
pub fn check_withdrawal_circuit_breaker(
    withdrawal_amount: u64,
    period_withdrawals: u64,
    max_period_withdrawals: u64,
) -> Result<()> {
    let new_total = period_withdrawals + withdrawal_amount;
    require!(
        new_total <= max_period_withdrawals,
        "Withdrawal circuit breaker triggered"
    );
    Ok(())
}

// 3. Oracle Circuit Breaker
// Halt if oracle is stale or shows anomalous data
// Combined with Chainlink Proof of Reserve for bridged assets
```

#### Emergency Shutdown (MakerDAO-style)

A protocol's nuclear option -- complete halt of all operations:

```
Emergency Shutdown Sequence:
  1. Triggered by governance multisig or oracle-based automatic trigger
  2. Immediately freeze all new borrows and deposits
  3. Lock oracle prices at current values (prevent manipulation during shutdown)
  4. Allow orderly position unwinding:
     a. Borrowers can repay and reclaim collateral
     b. Depositors can withdraw proportional share of remaining assets
  5. If bad debt exists, socialize losses across depositors
  6. Time-lock on restart (e.g., 48 hours minimum)
```

### 4.5 Insurance Funds and Bad Debt Socialization

When a position is liquidated but the collateral is insufficient to cover the debt (e.g., during a flash crash), the protocol incurs **bad debt**. Handling this is critical:

**Insurance Fund / Safety Module:**
```
Revenue Flow:
  Protocol fees -> X% to Insurance Fund, Y% to Treasury, Z% to Stakers

Insurance Fund Usage:
  1. First line of defense: Cover bad debt from insurance fund
  2. If insurance fund is depleted: Socialize across all depositors
  3. Final backstop: Governance intervention / token dilution

Aave Safety Module:
  - Users stake AAVE tokens in Safety Module
  - Earn staking rewards (from protocol fees)
  - In return, up to 30% of staked AAVE can be slashed to cover bad debt
  - Creates aligned incentives: stakers earn yield but accept downside risk
```

**Bad Debt Socialization:**
```
When insurance is insufficient:
  1. Calculate total bad debt
  2. Distribute proportionally across all depositors of the affected asset
  3. Each depositor's claimable amount reduced by:
     loss_per_depositor = total_bad_debt * user_deposits / total_deposits

  // This is why supply caps matter -- they limit maximum possible bad debt
```

---

## 5. Governance and Upgradeability

### 5.1 Governance Token Models

Governance tokens give holders voting power over protocol parameters and upgrades:

```
Standard Governance Flow:
  1. Proposal Creation (requires minimum token threshold)
  2. Voting Period (typically 3-7 days)
  3. Timelock Delay (typically 24-48 hours)
  4. Execution (automatic or manual)

Common Governance Decisions:
  - Risk parameter changes (LTV, caps, interest rates)
  - New asset listings
  - Fee changes
  - Protocol upgrades
  - Treasury allocation
  - Emergency actions
```

**Token-Weighted Voting:**
Simple 1-token-1-vote. Problem: plutocratic, flash loan governance attacks possible.

**Delegate Voting:**
Token holders delegate to "professional voters" who specialize in governance. Reduces voter apathy.

**Quadratic Voting:**
Voting power = sqrt(tokens). Reduces concentration of power but is Sybil-vulnerable.

### 5.2 Timelock Contracts

Timelocks enforce a delay between when a governance action is approved and when it can be executed:

```
Purpose:
  - Give users time to evaluate approved changes
  - Allow users to exit positions before unfavorable changes take effect
  - Prevent governance attacks from having immediate effect

Typical Parameters:
  - Minor parameter changes: 24-hour timelock
  - Major changes (new assets, fee changes): 48-72 hour timelock
  - Critical upgrades: 7-14 day timelock
  - Emergency actions: Can bypass timelock via multisig (but limited scope)

Implementation Pattern:
  1. Governance votes to approve proposal
  2. Proposal queued in Timelock contract with execution timestamp
  3. Anyone can monitor the Timelock for pending changes
  4. After delay, anyone can trigger execution
  5. If not executed within grace period, proposal expires
```

### 5.3 Multi-Sig Administration

Multi-signature wallets require N of M authorized signers to approve transactions:

```
Common Configurations:
  - 3/5 multisig: 3 of 5 signers needed (early stage)
  - 5/9 multisig: Standard for established protocols (Yearn uses 6/9)
  - 7/13 multisig: Maximum security, slower execution

Role-Based Multisig Strategy:
  - Emergency Multisig (3/5): Can pause protocol, limited to safety actions
  - Operations Multisig (4/7): Can adjust parameters within governance-set bounds
  - Upgrade Multisig (6/9): Can upgrade contracts (with timelock)
  - Treasury Multisig (5/7): Controls treasury spending

Best Practices:
  - Geographically distributed signers
  - Mix of team members and external/community signers
  - Hardware wallet requirements for all signers
  - Regular key rotation
  - Document and publish signer identities (where appropriate)
```

### 5.4 Progressive Decentralization

The journey from centralized control to full decentralization:

```
Phase 1: Team-Controlled (Launch)
  - Team multisig has full control
  - Rapid iteration and bug fixes
  - All parameters set by team
  - Rationale: Speed and safety during early, unproven period

Phase 2: Guided Governance (Months 3-12)
  - Governance token distributed
  - Community can propose changes
  - Team retains emergency powers (multisig)
  - Timelock on all governance actions
  - Team guides parameter decisions, community ratifies

Phase 3: Community Governance (Year 1-2)
  - Full governance control transferred to token holders
  - Team multisig reduced to emergency pause only
  - Professional risk managers (Gauntlet/Chaos) advise governance
  - Delegate ecosystem matures

Phase 4: Ossification (Year 2+)
  - Core contracts become immutable
  - Only parameter changes via governance
  - Emergency powers further restricted or removed
  - Protocol "runs itself"
```

### 5.5 Upgradeability vs. Trustlessness Tradeoff

The fundamental tension in DeFi protocol design:

```
UPGRADEABLE                          IMMUTABLE
  - Can fix bugs                       - Users trust the code, not the team
  - Can add features                   - No rug-pull risk from upgrades
  - Can respond to attacks             - Composability is reliable
  - Risk: malicious upgrades           - Risk: bugs are permanent
  - Trust assumption: admin/governance - Zero trust assumptions

Hybrid Strategy (Recommended):

  IMMUTABLE:
    - Core lending/AMM logic
    - Token contracts
    - User fund custody

  UPGRADEABLE (with timelock + governance):
    - Interest rate models
    - Oracle adapters
    - Liquidation parameters
    - Fee configurations
    - New asset additions

Solana-Specific:
  - Solana programs can be marked as immutable by revoking upgrade authority
  - Before immutability: use upgrade authority with multisig
  - Pattern: deploy v2 as new program, migrate via "social migration"
    rather than proxy-based upgrades
  - Anchor's `declare_id!` ensures program ID consistency
```

---

## 6. Fee Design and Tokenomics

### 6.1 How Protocols Capture Value

DeFi protocols capture value through various fee mechanisms:

```
Lending Protocols:
  - Interest rate spread: supply_rate < borrow_rate
  - Reserve factor: % of interest that goes to protocol (typically 10-20%)
  - Origination fees: One-time fee on new borrows (rare, usually 0.01-0.05%)
  - Liquidation fees: Protocol takes a cut of liquidation bonus

DEXs:
  - Swap fees: 0.01% to 1% per trade (varies by pool)
  - Protocol fee: Portion of swap fees diverted to protocol treasury
  - LP withdrawal fees: Rare, discourages mercenary liquidity

Yield Aggregators:
  - Performance fee: % of yield generated (typically 10-20%)
  - Management fee: Annual fee on AUM (typically 0-2%)
  - Withdrawal fee: Small fee to discourage rapid in/out (0.01-0.1%)

Derivatives:
  - Trading fees: Per-trade fee (typically 0.05-0.1%)
  - Funding rates: Periodic payments between longs/shorts
  - Liquidation fees: Protocol's share of liquidation proceeds
```

### 6.2 The Fee Switch (Uniswap Model)

Uniswap's fee switch is a landmark case study in DeFi fee design:

```
Background:
  - Uniswap initially directed ALL swap fees to LPs (no protocol revenue)
  - UNI token had zero value accrual -- purely governance
  - The "fee switch" is a governance-controlled parameter that diverts
    a portion of LP fees to the protocol

Fee Switch Parameters:
  - V2: Redirects 0.05% of 0.30% swap fee (1/6) to protocol
  - V3: Configurable per pool (1/4 to 1/10 of pool fee)
  - LP fees decrease when fee switch is on (from 0.30% to 0.25% in V2)

UNIfication Proposal (Passed 2025):
  - Activated fee switch
  - Fees flow to a "Token Jar" contract
  - Anyone who burns UNI can withdraw proportional share from jar
  - Protocol Fee Discount Auction (PFDA): Auction right to skip protocol fee
    -> winning bid burns UNI, MEV recaptured
  - 100M UNI burned immediately from treasury
```

**Key Insight for Builders:** The fee switch debate illustrates the tension between LP compensation and token holder value. Starting without a fee switch (all fees to LPs) maximizes LP attraction. The fee switch can be activated later once sufficient liquidity and market share are established.

### 6.3 Token Buyback and Burn Mechanisms

```
Buyback-and-Burn Flow:
  1. Protocol earns fees in various tokens
  2. Fees accumulated in treasury/fee contract
  3. Periodically (or continuously), treasury buys protocol token on open market
  4. Purchased tokens are burned (sent to dead address)
  5. Reduces circulating supply -> deflationary pressure

Design Considerations:
  - Automation: Use smart contracts to execute buybacks transparently
    (no discretionary timing by team)
  - Frequency: Continuous small buybacks vs. periodic large ones
    (continuous is more manipulation-resistant)
  - Transparency: On-chain verifiable burns with public reporting
  - Slippage: Use TWAP execution to minimize price impact

Real Example - Raydium (Solana):
  - 12% of all trading fees go to RAY buybacks
  - $196M+ spent buying back ~71M RAY (26.4% of circulating supply)
  - Fully automated, on-chain verifiable

Alternative: Revenue Distribution (not burn)
  - Instead of burning, distribute fees directly to stakers/holders
  - Maker's Smart Burn Engine: uses protocol surplus to buy MKR in market
  - Some protocols prefer distribution (income) vs. burn (capital appreciation)
```

### 6.4 Vote-Escrowed Tokens (ve-Tokens)

The ve-token model, pioneered by Curve Finance, is the most influential tokenomics design in DeFi:

```
Core Mechanism:
  1. Lock governance tokens for a chosen duration (up to 4 years)
  2. Receive ve-tokens proportional to lock duration
  3. ve-tokens are NON-TRANSFERABLE
  4. Voting power decays linearly toward unlock date
  5. ve-holders earn protocol revenue + boosted rewards

veCRV Example:
  Lock 1,000 CRV for 4 years -> receive 1,000 veCRV
  Lock 1,000 CRV for 2 years -> receive 500 veCRV
  Lock 1,000 CRV for 1 year  -> receive 250 veCRV

  After 2 years (of 4-year lock):
    Your 1,000 veCRV has decayed to 500 veCRV
    Must re-lock to maintain full voting power

Benefits of ve-Tokens:
  Revenue Share:
    - Curve charges 0.04% swap fee
    - 50% to LPs, 50% to veCRV holders
    - veCRV holders earn real yield from protocol usage

  Gauge Voting:
    - veCRV holders vote on which pools receive CRV emissions
    - This determines where liquidity is incentivized
    - Creates enormous strategic value in controlling votes

  Boost Mechanism:
    - LPs who also hold veCRV get up to 2.5x boosted CRV rewards
    - Aligns LP and governance incentives
    - Encourages LPs to also become long-term token holders
```

**Implementing ve-Tokens:**
```rust
// Simplified ve-token state on Solana
#[account]
pub struct VePosition {
    pub owner: Pubkey,
    pub locked_amount: u64,
    pub lock_end: i64,        // Unix timestamp
    pub lock_start: i64,
    pub ve_balance_at_lock: u64,
}

impl VePosition {
    pub fn current_ve_balance(&self, now: i64) -> u64 {
        if now >= self.lock_end {
            return 0;
        }
        let total_duration = self.lock_end - self.lock_start;
        let remaining = self.lock_end - now;

        // Linear decay
        self.ve_balance_at_lock * remaining as u64 / total_duration as u64
    }
}

pub fn lock_tokens(
    ctx: Context<Lock>,
    amount: u64,
    lock_duration: i64,
) -> Result<()> {
    require!(lock_duration <= MAX_LOCK_DURATION, "Lock too long");
    require!(lock_duration >= MIN_LOCK_DURATION, "Lock too short");

    let now = Clock::get()?.unix_timestamp;
    let lock_end = now + lock_duration;

    // ve balance = amount * lock_duration / max_lock_duration
    let ve_balance = amount * lock_duration as u64 / MAX_LOCK_DURATION as u64;

    let position = &mut ctx.accounts.ve_position;
    position.owner = ctx.accounts.user.key();
    position.locked_amount = amount;
    position.lock_end = lock_end;
    position.lock_start = now;
    position.ve_balance_at_lock = ve_balance;

    // Transfer tokens to lock vault
    transfer_to_lock_vault(amount)?;

    Ok(())
}
```

### 6.5 Bribes and Vote Markets (Convex/Votium Model)

The ve-token model spawned an entire meta-game of "voting markets":

```
The Curve Wars Ecosystem:

Curve Finance:        veCRV holders vote on gauge weights (emission allocation)
    |
    v
Convex Finance:       Aggregates CRV -> locks as veCRV permanently
    |                 Issues cvxCRV (liquid wrapper) to depositors
    |                 vlCVX holders control Convex's veCRV votes
    |
    v
Votium:               Marketplace where protocols bribe vlCVX holders
                      to vote for specific gauge weights

Bribe Economics:
  Protocol X wants liquidity in its Curve pool
  -> Posts bribe: "Vote for pool X's gauge, earn $Y per vlCVX voted"
  -> vlCVX holders compare bribe value vs. opportunity cost
  -> If bribe ROI > alternative, they vote for pool X
  -> Pool X receives more CRV emissions
  -> More CRV emissions attract more LPs
  -> More LPs = deeper liquidity for Protocol X

Bribe Efficiency:
  Cost of $1 of CRV emissions via bribes ≈ $0.80-1.20
  (When < $1, bribing is cheaper than buying CRV directly)

  Protocols compare:
    Cost of bribing for emissions vs.
    Cost of direct LP incentives
  Usually bribing is more capital-efficient

Ve(3,3) Evolution (Solidly/Velodrome):
  Combines ve-tokens with (3,3) game theory:
  - Emissions proportional to voting, but also to fees generated
  - Pools that generate more fees get more emissions
  - Creates positive feedback loop: fees -> votes -> emissions -> TVL -> fees
  - Solana equivalents exist (e.g., various ve-model DEXs)
```

---

## 7. Security Best Practices for DeFi Development

### 7.1 Common Vulnerability Patterns

#### Reentrancy

A contract calls an external contract, which calls back into the original contract before the first call completes. The original contract's state is inconsistent.

```
Classic Pattern:
  1. User calls withdraw()
  2. Contract sends ETH/SOL to user
  3. User's receive callback re-enters withdraw()
  4. Contract hasn't updated balance yet -> sends again
  5. Repeat until drained

// VULNERABLE (EVM):
function withdraw() {
    uint amount = balances[msg.sender];
    msg.sender.call{value: amount}("");  // External call before state update
    balances[msg.sender] = 0;           // Too late!
}

// SAFE: Checks-Effects-Interactions pattern
function withdraw() {
    uint amount = balances[msg.sender];
    balances[msg.sender] = 0;           // Update state FIRST
    msg.sender.call{value: amount}("");  // External call LAST
}
```

On Solana, traditional reentrancy is prevented by the runtime (programs cannot call themselves via CPI). However, **cross-program reentrancy** is still possible if Program A calls Program B which calls back into Program A. Use reentrancy guards:

```rust
// Solana reentrancy guard pattern
#[account]
pub struct ProtocolState {
    pub reentrancy_guard: bool,
    // ...
}

pub fn sensitive_operation(ctx: Context<SensitiveOp>) -> Result<()> {
    let state = &mut ctx.accounts.protocol_state;
    require!(!state.reentrancy_guard, "Reentrancy detected");
    state.reentrancy_guard = true;

    // ... perform operations including CPIs ...

    state.reentrancy_guard = false;
    Ok(())
}
```

#### Flash Loan Attacks

Flash loans allow borrowing unlimited capital with zero collateral, provided it is returned within the same transaction. Attackers use this to:

```
Attack Patterns:

1. Price Manipulation:
   - Flash loan huge amount of token A
   - Swap on DEX to spike price of token B
   - Use inflated price to borrow against token B
   - Repay flash loan, keep profits

2. Governance Manipulation:
   - Flash loan governance tokens
   - Vote on malicious proposal (if no snapshot/lock requirement)
   - Repay flash loan

3. Oracle Manipulation:
   - Flash loan + massive swap to move spot price
   - If protocol uses spot price as oracle, exploit the mispricing
   - Repay flash loan

Defenses:
   - Use TWAP or off-chain oracles (not manipulable in single tx)
   - Snapshot token balances for governance voting
   - Require tokens to be locked for N blocks before they have voting power
   - Multi-block delay for large position changes
```

#### Price / Oracle Manipulation

```
Real Example - Radiant Capital ($4.5M loss):
  Exploited a known rounding issue in Compound/Aave codebase
  Combined with flash loan to amplify the rounding error

Real Example - Cheese Bank ($3.3M loss):
  Flash loan used to manipulate Uniswap collateral price
  Protocol relied on Uniswap spot price as oracle

Real Example - Euler Finance ($197M loss):
  Flash loan exploited flaw in lending logic
  Manipulated debt positions to drain funds in single transaction
```

#### Rounding Errors

Integer arithmetic in smart contracts always rounds (typically toward zero). Attackers exploit this for value extraction:

```
// VULNERABLE: Rounding can be exploited
fn calculate_interest(principal: u64, rate_bps: u64, time: u64) -> u64 {
    principal * rate_bps * time / 10000 / SECONDS_PER_YEAR
    // If principal * rate_bps * time < 10000 * SECONDS_PER_YEAR,
    // result rounds to 0 -- borrower pays no interest!
}

// SAFER: Use higher precision intermediate calculation
fn calculate_interest(principal: u64, rate_bps: u64, time: u64) -> u64 {
    let numerator = (principal as u128) * (rate_bps as u128) * (time as u128);
    let denominator = 10000u128 * SECONDS_PER_YEAR as u128;
    // Round UP for interest owed (favor protocol)
    ((numerator + denominator - 1) / denominator) as u64
}

// General principle: ALWAYS round in favor of the protocol / existing users
// Deposits: round shares DOWN (depositor gets fewer shares)
// Withdrawals: round assets DOWN (withdrawer gets fewer assets)
// Interest: round UP (borrower pays at least minimum)
// Fees: round UP (protocol receives at least minimum)
```

#### Other Critical Vulnerabilities

```
Access Control Failures:
  - Missing signer checks on admin functions
  - PDA authority not properly validated
  - Anyone can call privileged functions

  // Solana Anchor: Always use proper account constraints
  #[account(
      mut,
      constraint = pool.admin == admin.key() @ ErrorCode::Unauthorized,
  )]
  pub pool: Account<'info, PoolState>,
  pub admin: Signer<'info>,

Account Validation (Solana-specific):
  - Not checking account ownership (any program could own the account)
  - Not checking account discriminator (wrong account type passed)
  - PDA seed injection (attacker crafts seeds to access wrong PDA)

  // Anchor handles most of this automatically with Account<'info, T>
  // But always validate cross-program accounts manually

Integer Overflow/Underflow:
  - Rust panics on overflow in debug mode but WRAPS in release mode
  - Always use checked arithmetic: checked_add, checked_mul, checked_div
  - Or use Anchor's #[overflow_checks] attribute

Denial of Service:
  - Unbounded loops over user-controlled arrays
  - Blocking withdrawals (griefing attack on shared pools)
  - Account size limits on Solana (10MB per account)

Timestamp Dependence:
  - Block timestamps can be slightly manipulated by validators
  - Don't use for time-sensitive operations with second-level precision
  - Use slot numbers for relative timing on Solana
```

### 7.2 The Audit Process

```
Pre-Audit Preparation:
  1. Freeze code (no changes during audit)
  2. Write comprehensive documentation:
     - Architecture overview
     - Threat model
     - Invariants that must hold
     - Known issues / accepted risks
  3. Full test suite with >90% coverage
  4. Internal review and static analysis first (Slither, Clippy)
  5. Define clear audit scope

Audit Phases:
  Phase 1: Automated Analysis
    - Static analysis tools (Slither for Solidity, custom tools for Rust)
    - Symbolic execution (Mythril, Manticore)
    - Fuzzing (Echidna, Medusa, Foundry fuzz)

  Phase 2: Manual Review
    - Line-by-line code review by experienced auditors
    - Focus on business logic, economic attacks, edge cases
    - Cross-reference with documentation and invariants

  Phase 3: Findings Report
    - Severity classification: Critical / High / Medium / Low / Informational
    - Proof of concept for each finding
    - Remediation recommendations

  Phase 4: Fix Review
    - Audit team reviews fixes for each finding
    - Verify fixes don't introduce new issues
    - Final report published

Cost and Timeline:
  - Simple protocol: $50K-100K, 2-4 weeks
  - Complex protocol: $200K-500K+, 4-12 weeks
  - Multiple audits recommended (different firms find different things)

Top Audit Firms (2024-2025):
  - Trail of Bits, OpenZeppelin, Spearbit, Cantina
  - Cyfrin, Sherlock (competitive audit platform)
  - OtterSec, Neodyme (Solana specialists)
  - Halborn, CertiK, MixBytes
```

### 7.3 Formal Verification

Mathematical proof that code behaves according to its specification:

```
What Formal Verification Does:
  - Proves properties hold for ALL possible inputs and states
  - Not just testing (which covers specific cases)
  - Can find bugs that fuzzing and auditing miss

Example Properties to Verify:
  - "Total shares * exchange_rate == total_assets (within rounding)"
  - "No sequence of operations can make total_debt > total_collateral"
  - "Only the owner can withdraw funds"
  - "Interest always accrues monotonically"

Tools:
  - Certora Prover: Writes specs in CVL (Certora Verification Language)
  - K Framework: Formal semantics of EVM
  - Solana: Limited tooling, but improving

Limitations:
  - Expensive and time-consuming
  - Requires formal specification (bugs in specs = false confidence)
  - Verifies code matches spec, not that spec is correct
  - Cannot verify economic/incentive design

Best used for: Core accounting logic, access control, token math
```

### 7.4 Bug Bounty Programs

Post-deployment continuous security through incentivized vulnerability discovery:

```
Program Structure:

Severity Tiers and Typical Payouts:
  Critical (direct fund loss):     $100K - $1M+
  High (potential fund loss):      $25K - $100K
  Medium (limited impact):         $5K - $25K
  Low (informational):             $1K - $5K

Best Practices:
  - Scope clearly defines in-scope contracts and out-of-scope issues
  - Payout sized as % of economic damage prevented (5-10% of TVL at risk)
  - Rapid response SLA (24h acknowledgment, 72h initial assessment)
  - Safe harbor for researchers acting in good faith
  - Transparent disclosure policy (90-day disclosure window)

Platforms:
  - Immunefi: Largest Web3 bug bounty platform ($100M+ paid out)
  - Sherlock: Audit contests + bug bounties combined
  - HackerOne / Bugcrowd: Traditional platforms with Web3 programs

Pricing Rule of Thumb:
  Critical bounty >= 10% of maximum damage the bug could cause
  This ensures even blackhats are incentivized to report rather than exploit
```

### 7.5 Invariant Testing

Invariant testing (property-based testing) is the most effective automated testing approach for DeFi:

```
Key Invariants for Common DeFi Protocols:

Lending Protocol:
  1. sum(all_deposits) >= sum(all_borrows) + reserves
  2. total_shares * exchange_rate == total_underlying (within rounding)
  3. health_factor >= 1.0 for all non-liquidatable positions
  4. interest accrual is monotonically increasing
  5. no user can withdraw more than they deposited + interest
  6. liquidation cannot make protocol's position worse

AMM/DEX:
  1. x * y >= k (constant product never decreases, only increases via fees)
  2. sum(LP_tokens) == total_LP_supply
  3. no swap can extract more value than the fee allows
  4. removing liquidity returns proportional share of both assets

Vault:
  1. total_shares * price_per_share == total_assets (within rounding)
  2. deposit followed by immediate withdrawal returns >= (deposit - max_rounding_loss)
  3. sum(user_shares) == total_share_supply
  4. no user can withdraw other users' funds
  5. share price only increases (for yield-bearing vaults)
```

```rust
// Invariant test example (Rust/Anchor test pseudocode)
#[test]
fn invariant_total_assets_matches_shares() {
    let mut rng = thread_rng();

    for _ in 0..10_000 {
        // Random action: deposit, withdraw, accrue interest, or liquidate
        match rng.gen_range(0..4) {
            0 => {
                let amount = rng.gen_range(1..1_000_000_000);
                deposit(&mut state, amount);
            }
            1 => {
                let shares = rng.gen_range(1..state.total_shares.max(1));
                withdraw(&mut state, shares);
            }
            2 => {
                accrue_interest(&mut state);
            }
            3 => {
                attempt_liquidation(&mut state);
            }
            _ => unreachable!(),
        }

        // CHECK INVARIANT after every action
        let expected_assets = state.total_shares as u128
            * state.exchange_rate as u128 / PRECISION;
        let actual_assets = state.total_assets;

        // Allow 1 unit of rounding error per operation
        assert!(
            (expected_assets as i128 - actual_assets as i128).abs() <= MAX_ROUNDING_ERROR,
            "Invariant violated: shares*rate != assets"
        );
    }
}
```

**Fuzzing Tools:**
- **Echidna:** Generates random transaction sequences, checks user-defined properties
- **Medusa:** Similar to Echidna, with additional features
- **Foundry Fuzz:** Built into Foundry, Solidity-native
- **Trident:** Solana-specific fuzzing framework for Anchor programs
- **Recon:** Unified platform integrating multiple fuzzers

### 7.6 Economic Security vs. Smart Contract Security

```
Smart Contract Security:
  "Does the code do what it's supposed to do?"
  - Reentrancy, overflow, access control, etc.
  - Can be found by auditors, fuzzers, formal verification
  - Binary: the code is either vulnerable or it's not

Economic Security:
  "Even if the code is perfect, can the SYSTEM be exploited?"
  - Oracle manipulation (code works correctly with wrong price data)
  - Governance attacks (voting to drain treasury)
  - Market manipulation (creating conditions that trigger unintended behavior)
  - Incentive misalignment (rational actors behaving in protocol-harmful ways)
  - Cascading liquidations (one liquidation causes prices to drop, triggering more)
  - Cannot be found by code auditing alone

Economic Security Analysis:
  1. Agent-based modeling: Simulate rational and adversarial actors
  2. Game theory analysis: What's the optimal strategy for each participant?
  3. Mechanism design review: Are incentives aligned?
  4. Stress testing: What happens under extreme conditions?
  5. Attack cost analysis: How much capital needed to profitably attack?

Key Principle:
  The cost to attack the protocol must ALWAYS exceed the profit from attacking.

  attack_cost > attack_profit

  This means:
  - Oracle manipulation cost > maximum extractable value from bad prices
  - Governance attack cost > treasury value
  - Liquidity manipulation cost > liquidation profit
```

### 7.7 Comprehensive Security Checklist for DeFi Builders

```
Pre-Launch:
  [ ] Multiple independent audits (at least 2 firms)
  [ ] Invariant test suite covering all critical properties
  [ ] Fuzz testing with 10K+ random sequences
  [ ] Formal verification of core accounting logic
  [ ] Economic security review / attack simulation
  [ ] Testnet deployment with public testing period
  [ ] Emergency pause mechanism implemented and tested
  [ ] Admin key management (multisig, timelock)
  [ ] Oracle integration tested with stale/manipulated prices
  [ ] Checked arithmetic everywhere (no unchecked blocks for math)
  [ ] Rounding direction correct for all conversions
  [ ] Access control on all admin/privileged functions
  [ ] Account validation (Solana: ownership, discriminator, seeds)
  [ ] First-depositor attack mitigated (virtual shares or seed deposit)

Post-Launch:
  [ ] Bug bounty program live (Immunefi or similar)
  [ ] Real-time monitoring (Forta, Tenderly, custom alerts)
  [ ] Incident response plan documented
  [ ] Insurance coverage (Nexus Mutual, InsurAce, or self-insured)
  [ ] Regular risk parameter reviews
  [ ] Ongoing audit for new features/upgrades
  [ ] Transparent security disclosure process
```

---

## 8. References

### Architecture and Design
- [DeFi Protocol Architecture: A Builder's Guide](https://medium.com/@athguy.dev/defi-protocol-architecture-a-builders-guide-to-what-actually-happens-under-the-hood-38713f93b4e5)
- [Modern DeFi Lending Protocols: The Compilation](https://mixbytes.io/blog/modern-lending-protocols-how-its-made-the-compilation)
- [How to Design a Lending Protocol on Ethereum](https://alcueca.medium.com/how-to-design-a-lending-protocol-on-ethereum-18ba5849aaf0)
- [How DeFi Lending Protocols Work: A Developer's Guide](https://metalamp.io/magazine/article/how-defi-lending-protocols-work-a-developers-guide)
- [Intent-Based Architecture in DeFi](https://www.orbs.com/Intent-Based-Architecture-in-DeFi/)
- [Euler V2: The New Modular Age of DeFi](https://www.euler.finance/blog/euler-v2-the-new-modular-age-of-defi)
- [Modern DeFi Lending Protocols: Euler V2](https://mixbytes.io/blog/modern-defi-lending-protocols-how-its-made-euler-v2)
- [Euler Vault Kit Introduction](https://docs.euler.finance/creator-tools/vaults/evk/introduction/)

### Vault and Share Token Design
- [ERC-4626 Tokenized Vault Standard](https://ethereum.org/developers/docs/standards/tokens/erc-4626/)
- [ERC-4626 Vaults: Secure Design, Risks & Best Practices](https://speedrunethereum.com/guides/erc-4626-vaults)
- [ERC-4626 | OpenZeppelin Docs](https://docs.openzeppelin.com/contracts/5.x/erc4626)
- [ERC4626 Interface Explained | RareSkills](https://rareskills.io/post/erc4626)
- [A Novel Defense Against ERC4626 Inflation Attacks | OpenZeppelin](https://www.openzeppelin.com/news/a-novel-defense-against-erc4626-inflation-attacks)
- [Overview of the Inflation Attack | MixBytes](https://mixbytes.io/blog/overview-of-the-inflation-attack)
- [Solana ERC-4626 Equivalent](https://solana.com/developers/evm-to-svm/erc4626)
- [Compound V2 cTokens Documentation](https://docs.compound.finance/v2/ctokens/)
- [Back to the Basics: Compound, Aave](https://medium.com/@kinaumov/back-to-the-basics-compound-aave-436a1887ad94)

### AMM and DEX Design
- [Automated Market Makers: Math, Risks & Solidity Code](https://speedrunethereum.com/guides/automated-market-makers-math)
- [Concentrated Liquidity | Uniswap V3 Docs](https://docs.uniswap.org/concepts/protocol/concentrated-liquidity)
- [Liquidity Math in Uniswap V3 (Technical Note)](https://atiselsts.github.io/pdfs/uniswap-v3-liquidity-math.pdf)
- [Uniswap V3 Ticks: Dive Into Concentrated Liquidity | MixBytes](https://mixbytes.io/blog/uniswap-v3-ticks-dive-into-concentrated-liquidity)
- [Introducing Ticks in Uniswap V3 | RareSkills](https://rareskills.io/post/uniswap-v3-ticks)

### Interest Rate Models
- [Interest Rate Model of Aave V3 and Compound V2 | RareSkills](https://rareskills.io/post/aave-interest-rate-model)
- [Understanding Compound Protocol's Interest Rates](https://ianm.com/posts/2020-12-20-understanding-compound-protocols-interest-rates)
- [Compound III Interest Rates Documentation](https://docs.compound.finance/interest-rates/)

### Oracle Design
- [Pull, Don't Push: A New Price Oracle Architecture (Pyth)](https://www.pyth.network/blog/pyth-a-new-model-to-the-price-oracle)
- [How Pyth Works | Pyth Developer Hub](https://docs.pyth.network/price-feeds/core/how-pyth-works)
- [How to Use Pyth for Price Feeds on Solana](https://www.quicknode.com/guides/solana-development/3rd-party-integrations/pyth-price-feeds)
- [Switchboard vs. The Competition](https://switchboardxyz.medium.com/switchboard-vs-the-competition-why-we-are-the-everything-oracle-bbc27b967215)
- [Switchboard Surge: The Fastest Oracle on Solana](https://switchboardxyz.medium.com/introducing-switchboard-surge-the-fastest-oracle-on-solana-is-here-36ff615bfdf9)
- [Oracle Wars: Rise of Price Manipulation Attacks | CertiK](https://www.certik.com/resources/blog/oracle-wars-the-rise-of-price-manipulation-attacks)
- [The Full Guide to Price Oracle Manipulation Attacks | Cyfrin](https://www.cyfrin.io/blog/price-oracle-manipulation-attacks-with-examples)
- [TWAP Oracle Attacks: Easier Done than Said?](https://eprint.iacr.org/2022/445.pdf)
- [Oracle Manipulation | Smart Contract Security Field Guide](https://scsfg.io/hackers/oracle-manipulation/)

### Liquidation Design
- [How Liquidations Work in DeFi: A Deep Dive | MixBytes](https://mixbytes.io/blog/how-liquidations-work-in-defi-a-deep-dive)
- [DeFi Risk Management: Liquidation Engine Design for 2025](https://johal.in/defi-risk-management-liquidation-engine-design-for-2025-lending-protocols-2/)
- [MakerDAO Liquidation 2.0 Module Documentation](https://docs.makerdao.com/smart-contract-modules/dog-and-clipper-detailed-documentation)
- [LLAMMA Explainer | Curve Technical Docs](https://docs.curve.finance/crvUSD/llamma-explainer/)
- [Curve crvUSD Loan Concepts](https://resources.curve.finance/crvusd/loan-concepts/)
- [DeFi Liquidations, Incentives, Risks | Academic Paper](https://arxiv.org/pdf/2106.06389)
- [Liquidators: The Secret Whales Helping DeFi Function](https://medium.com/dragonfly-research/liquidators-the-secret-whales-helping-defi-function-acf132fbea5e)
- [Tutorial: Liquidation Bot | Drift Protocol](https://docs.drift.trade/tutorial-bots/keeper-bots/tutorial-liquidation-bot)

### Risk Management
- [DeFi Lending & Borrowing Risk Framework | ChainRisk](https://www.chainrisk.xyz/blog-posts/defi-lending-borrowing-risk-framework)
- [DeFi Liquidation Risks & Vulnerabilities | Cyfrin](https://www.cyfrin.io/blog/defi-liquidation-vulnerabilities-and-mitigation-strategies)
- [DeFi Liquidations and Collateral | RareSkills](https://rareskills.io/post/defi-liquidations-collateral)
- [Aave V3 Risk Parameters | GitHub](https://github.com/aave/risk-v3/blob/main/asset-risk/risk-parameters.md)
- [Aave V3 Overview](https://aave.com/docs/aave-v3/overview)
- [Chaos Labs Risk Oracles](https://chaoslabs.xyz/posts/risk-oracles-real-time-risk-management-for-defi)
- [Gauntlet: Simulation-Based Risk Modeling](https://medium.com/@gwrx2005/gauntlet-simulation-based-risk-modeling-and-quantitative-research-for-defi-fb736e4392d2)
- [Circuit Breakers in Web3 | Olympix](https://olympixai.medium.com/circuit-breakers-in-web3-a-comprehensive-analysis-of-defis-emergency-brake-d76f838226f2)
- [DeFi Circuit Breakers With Chainlink](https://blog.chain.link/defi-circuit-breakers/)
- [ERC 7265: Circuit Breaker Standard](https://medium.com/@bunzzdev/what-is-erc726555-enhancing-defi-security-with-a-circuit-breaker-mechanism-b82becf68552)

### Governance and Upgradeability
- [Best Practices For Secure DeFi Governance | Halborn](https://www.halborn.com/blog/post/best-practices-for-secure-defi-governance)
- [The Guide to DeFi Governance | Blockchain at Berkeley](https://calblockchain.mirror.xyz/Imjv6Y13fA23MpvEoHdYnRDnIvrcp_pyBnwn9pTZ0Bo)
- [Upgradeable Smart Contracts: Proxy & UUPS Explained | Three Sigma](https://threesigma.xyz/blog/web3-security/upgradeable-smart-contracts-proxy-patterns-ethereum)
- [Behind the Decentralization Theater | TokenBrice](https://tokenbrice.xyz/unstoppable-defi/)
- [How to Build Timelock Smart Contracts](https://101blockchains.com/timelock-smart-contracts/)

### Tokenomics and Fee Design
- [Uniswap Tokenomics: From Governance to Value Accrual](https://tokenomics.com/articles/uniswap-tokenomics)
- [UNIfication (Uniswap Fee Switch)](https://blog.uniswap.org/unification)
- [Fee Switch Design Space | Uniswap Governance](https://gov.uniswap.org/t/fee-switch-design-space-next-steps/17132)
- [Curve Finance and veCRV Tokenomics](https://medium.com/coinmonks/curve-finance-and-vecrv-8490d51537c5)
- [Beyond Burn: Why veCRV Unlocks Sustainable Tokenomics](https://news.curve.finance/beyond-burn-why-vecrv-unlocks-sustainable-tokenomics-for-curve/)
- [veTokenomics & Bribe Markets | Mitosis University](https://university.mitosis.org/vetokenomics-bribe-markets-gauge-voting-incentives-and-curve-wars-mechanics/)
- [Ve-Token Model Implementation Strategy Guide](https://tokenomics.net/blog/ve-token-model-implementation-strategy-guide)
- [Token Buybacks in Web3 | DWF Labs](https://www.dwf-labs.com/research/547-token-buybacks-in-web3)
- [Ve(3,3) Comprehensive Guide | MEXC](https://blog.mexc.com/exploring-ve33-a-comprehensive-guide-to-understanding-the-tokenomics-model-creator-kingsley/)

### Security
- [Flash Loan Attacks: Understanding DeFi Security Risks](https://www.startupdefense.io/cyberattacks/flash-loan-attack)
- [Flash Loan Attacks: Risks & Prevention | Hacken](https://hacken.io/discover/flash-loan-attacks/)
- [Price Oracle Manipulation Attacks in DeFi | Halborn](https://www.halborn.com/blog/post/what-are-price-oracle-manipulation-attacks-in-defi)
- [Web3 Security Best Practices: Complete Guide 2025 | Olympix](https://olympix.security/blog/web3-security-best-practices-complete-guide-for-smart-contract-protection-in-2025)
- [Bug Bounty Programs Explained | Sherlock](https://sherlock.xyz/post/web3-bug-bounty-programs-explained-continuous-protection-for-defi-protocols)
- [A DeFi Security Standard: The Scaling Bug Bounty | Immunefi](https://immunefi.com/blog/industry-trends/a-defi-security-standard-the-scaling-bug-bounty/)
- [MEV Protection Techniques | Olympix](https://olympixai.medium.com/mev-miner-extractable-value-protection-techniques-and-emerging-solutions-2b1d66886cd6)
- [Chainlink SVR Analysis: How DeFi Protocols Can Recapture MEV](https://blog.chain.link/chainlink-svr-analysis/)
- [Solana Security Patterns: A Deep Dive](https://angrypacifist.substack.com/p/solana-security-patterns)

### Solana-Specific
- [Solana DeFi Developer Resources](https://solana.com/developers/defi)
- [Anchor Framework Documentation](https://www.anchor-lang.com/docs)
- [Program Derived Addresses (PDAs) | Solana Docs](https://solana.com/docs/core/pda)
- [What are Solana PDAs? | Helius](https://www.helius.dev/blog/solana-pda)
- [PDA Sharing Security | Solana Courses](https://solana.com/developers/courses/program-security/pda-sharing)
- [Mastering Cross-Program Invocations in Anchor](https://medium.com/@ancilartech/mastering-cross-program-invocations-in-anchor-a-developers-guide-to-solana-s-cpi-patterns-0f29a5734a3e)

### Risk Simulation and Economic Security
- [ChainRisk: DeFi Risk Management Solutions](https://chainrisk.xyz/)
- [Stress Testing and Scenario Analysis | Amplified Protocol](https://docs.amplified.fi/security-and-risk-assessment-report/stress-testing-and-scenario-analysis)
- [Stress Testing in Tokenomics | Nomiks](https://medium.com/@Nomiks/stress-testing-in-tokenomics-the-state-of-the-art-in-web3-simulation-9f5ca4054759)
