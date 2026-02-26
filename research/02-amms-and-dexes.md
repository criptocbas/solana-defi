# Automated Market Makers (AMMs) and Decentralized Exchanges: A Deep Technical Reference

> Written for experienced Solana developers entering DeFi. This document covers the mathematical foundations, design tradeoffs, and practical mechanics of AMMs and DEXes from first principles through advanced designs.

---

## Table of Contents

1. [The Problem AMMs Solve](#1-the-problem-amms-solve)
2. [Constant Product Market Maker (x * y = k)](#2-constant-product-market-maker-x--y--k)
3. [Liquidity Provision](#3-liquidity-provision)
4. [Impermanent Loss](#4-impermanent-loss)
5. [Uniswap v3 Concentrated Liquidity](#5-uniswap-v3-concentrated-liquidity)
6. [Curve Finance and the StableSwap Invariant](#6-curve-finance-and-the-stableswap-invariant)
7. [Order Book DEXes](#7-order-book-dexes)
8. [Advanced AMM Designs](#8-advanced-amm-designs)
9. [MEV and Sandwich Attacks](#9-mev-and-sandwich-attacks)
10. [The Fee Structure](#10-the-fee-structure)
11. [Solana DEX Ecosystem Overview](#11-solana-dex-ecosystem-overview)
12. [References](#12-references)

---

## 1. The Problem AMMs Solve

### Why Order Books Do Not Work Well On-Chain

Traditional finance relies on Central Limit Order Books (CLOBs): a sorted list of buy and sell orders matched by a centralized engine. This model requires:

- **High throughput**: Market makers submit and cancel hundreds or thousands of orders per minute. On Ethereum (pre-merge, ~15 TPS), each order placement or cancellation is a transaction that costs gas and competes for block space. Even on Solana (~4,000 TPS effective, 400ms slots), the overhead of on-chain order management is non-trivial.
- **Low latency**: Professional market makers require sub-millisecond response times to adjust quotes. Blockchain finality is measured in seconds at best.
- **Cheap cancellations**: In traditional order books, cancelling an order is free. On-chain, every cancellation costs gas/compute units. A market maker adjusting quotes 100 times per second across 50 pairs would be economically crushed by transaction fees.
- **MEV vulnerability**: On-chain order books expose every order to mempool observers. Market makers face frontrunning and sandwich attacks on every quote update, making on-chain market making significantly more expensive and risky.

### The Bootstrapping Problem

Order books face a classic chicken-and-egg problem:

1. **Traders need liquidity** -- without tight spreads and deep order books, traders suffer from slippage and poor execution.
2. **Liquidity needs traders** -- market makers only profit when there is trading volume; without it, they have capital sitting idle and exposed to adverse selection.
3. **New token pairs** are essentially impossible to bootstrap -- who will make a market for a brand-new token with no volume history and high volatility?

### How AMMs Solve This

AMMs replace the order book with a **deterministic pricing function** encoded in a smart contract. Key properties:

- **Permissionless liquidity**: Anyone can deposit tokens and earn fees. No market making expertise required.
- **Always-on liquidity**: The pool always quotes a price -- there is no "empty order book" state.
- **Algorithmic pricing**: Prices are determined mathematically from the reserve ratios, not by individual order placement.
- **Composability**: AMM pools are smart contracts that other protocols can build on top of (flash loans, yield aggregators, liquidation bots).

The tradeoff: AMMs generally provide worse execution than well-run order books for large trades, and expose liquidity providers to impermanent loss (covered in Section 4).

---

## 2. Constant Product Market Maker (x * y = k)

### The Formula

The Constant Product Market Maker (CPMM), popularized by Uniswap v1 (2018) and refined in Uniswap v2 (2020), uses the invariant:

```
x * y = k
```

Where:
- `x` = reserve of token X in the pool
- `y` = reserve of token Y in the pool
- `k` = a constant that must be preserved after every trade (excluding fees)

This is a **hyperbola** in (x, y) space. The pool can never reach x=0 or y=0 -- the curve asymptotically approaches both axes but never touches them. This means the pool always has some of both tokens and can always quote a price.

### How Prices Are Determined

The **marginal (spot) price** of token X in terms of token Y is derived from the ratio of reserves:

```
Price_X = y / x
Price_Y = x / y
```

This follows from taking the derivative of the invariant. If `x * y = k`, then:

```
d(x * y) = 0
y * dx + x * dy = 0
dy/dx = -y/x
```

The magnitude `|dy/dx| = y/x` is the instantaneous exchange rate: how many units of Y you receive per marginal unit of X.

### How Trades Execute Against the Curve

When a trader wants to swap `dx` units of token X for some amount `dy` of token Y:

1. The trader sends `dx` tokens of X to the pool contract.
2. The contract calculates the new reserve of X: `x' = x + dx`
3. To maintain the invariant: `x' * y' = k`, so `y' = k / x'`
4. The output amount is: `dy = y - y' = y - k/(x + dx)`

Rearranging:

```
dy = (y * dx) / (x + dx)
```

This is the **output amount formula** without fees. Note the key insight: the effective price gets worse as `dx` increases relative to `x`. This is **price impact** (commonly called slippage).

### Worked Example with Numbers

**Setup**: A pool contains 10 ETH and 20,000 USDC.

```
x = 10 ETH
y = 20,000 USDC
k = 10 * 20,000 = 200,000
Spot price: 20,000 / 10 = 2,000 USDC/ETH
```

**Trade 1: Buy 1 ETH (small trade)**

The trader sends USDC to receive 1 ETH. After the trade, x' = 9 ETH.

```
y' = k / x' = 200,000 / 9 = 22,222.22 USDC
Cost = y' - y = 22,222.22 - 20,000 = 2,222.22 USDC
Effective price = 2,222.22 USDC/ETH
Price impact = (2,222.22 - 2,000) / 2,000 = 11.1%
```

That is high because the trade size is 10% of the pool. Let us try a smaller trade.

**Trade 2: Buy 0.1 ETH (smaller trade)**

```
x' = 10 - 0.1 = 9.9 ETH
y' = 200,000 / 9.9 = 20,202.02 USDC
Cost = 20,202.02 - 20,000 = 202.02 USDC
Effective price = 202.02 / 0.1 = 2,020.20 USDC/ETH
Price impact = (2,020.20 - 2,000) / 2,000 = 1.01%
```

**Trade 3: Buy 5 ETH (large trade -- 50% of reserves)**

```
x' = 10 - 5 = 5 ETH
y' = 200,000 / 5 = 40,000 USDC
Cost = 40,000 - 20,000 = 20,000 USDC
Effective price = 20,000 / 5 = 4,000 USDC/ETH
Price impact = (4,000 - 2,000) / 2,000 = 100%
```

This demonstrates the key property of CPMMs: **price impact scales super-linearly with trade size relative to pool depth**. This naturally protects pools from being drained and incentivizes deep liquidity.

### With Fees (The Real Formula)

In practice, Uniswap v2 charges a 0.30% fee on the input amount. The fee is applied **before** the constant product calculation:

```
dx_effective = dx * (1 - fee)
             = dx * 0.997       (for 0.30% fee)

dy = (y * dx_effective) / (x + dx_effective)
```

Or equivalently in integer math (as implemented in Solidity/Rust):

```
amountOut = (reserveOut * amountIn * 997) / (reserveIn * 1000 + amountIn * 997)
```

The fee portion stays in the pool, increasing `k` over time. This is how LP fees accrue -- the invariant `k` grows with each trade.

**Revised Trade 2 with fees:**

```
dx = 0.1 ETH sends, but only 0.0997 ETH is effective
x' = 10 + 0.0997 = 10.0997
y' = 200,000 / 10.0997 = 19,802.58 (approximately)
dy = 20,000 - 19,802.58 = 197.42 USDC (vs. 202.02 without fees)
Effective price = 197.42 / 0.1 = 1,974.20 USDC/ETH
Fee paid = 202.02 - 197.42 ~ 4.60 USDC (remains in pool for LPs)
```

---

## 3. Liquidity Provision

### How LPs Deposit Tokens

In a Uniswap v2-style AMM, liquidity providers must deposit both tokens in the pool at the **current reserve ratio**. If the pool has 10 ETH and 20,000 USDC (ratio 1:2000), you must deposit at that same ratio.

For example, to add ~10% more liquidity:
- Deposit 1 ETH + 2,000 USDC
- This increases reserves to 11 ETH + 22,000 USDC
- k increases from 200,000 to 242,000

### LP Tokens: Minting and Burning

LP tokens represent proportional ownership of the pool. They are ERC-20 (or SPL on Solana) tokens themselves.

**First depositor (pool creation):**

```
LP_tokens_minted = sqrt(x * y) - MINIMUM_LIQUIDITY

Where MINIMUM_LIQUIDITY = 1000 (1e-15 in 18-decimal terms)
```

The `sqrt(x * y)` formula (geometric mean) ensures that the value of LP tokens is independent of the ratio at which initial liquidity was deposited. The MINIMUM_LIQUIDITY (1000 wei) is permanently locked by sending it to the zero address. This prevents a manipulation attack where someone could make a single LP token share prohibitively expensive.

**Subsequent depositors:**

```
LP_tokens_minted = min(
    (amount0 * totalSupply) / reserve0,
    (amount1 * totalSupply) / reserve1
)
```

This ensures the depositor provides tokens at the current ratio. If they provide an imbalanced deposit, they get LP tokens based on the lesser ratio (the excess is effectively donated to the pool).

**Withdrawing (burning):**

When an LP wants to exit, they burn their LP tokens:

```
amount0_returned = (LP_tokens_burned * reserve0) / totalSupply
amount1_returned = (LP_tokens_burned * reserve1) / totalSupply
```

The LP receives their proportional share of **both** tokens at the **current** ratio, which may differ from when they deposited.

### How Fees Accrue to LPs

Fees in Uniswap v2 are not distributed directly. Instead, fees stay in the pool and increase the reserves. Since `k` grows with each swap (the fee portion of the input stays in the pool), the value backing each LP token increases over time.

```
Before 1000 swaps: k = 200,000
After 1000 swaps:  k = 210,000 (hypothetical, depends on volume)

LP tokens represent a share of the now-larger pool.
```

This design is gas-efficient: there is no need to track or claim fee distributions. LPs simply hold their LP tokens and the value accrues automatically.

### Worked Example: LP Fee Accumulation

Starting pool: 10 ETH + 20,000 USDC, LP supply = 447.21 tokens (sqrt(200,000)).

After a period of trading with $1,000,000 cumulative volume and 0.30% fees:
- Total fees collected: $3,000 worth of tokens added to reserves
- If you hold 10% of LP tokens, your share of accumulated fees is ~$300

When you burn your LP tokens, you receive your proportional share of the now-larger reserves.

---

## 4. Impermanent Loss

### What It Really Is

Impermanent Loss (IL) is the difference in value between:
1. **Holding tokens in the AMM pool** (the LP position)
2. **Simply holding the same tokens in a wallet** (the HODL strategy)

It occurs because the AMM's constant product formula forces the pool to rebalance: as one token's price rises, arbitrageurs buy the appreciating token from the pool (cheaply) and sell the depreciating token into it. The LP ends up with more of the depreciating token and less of the appreciating token compared to simply holding.

The term "impermanent" is used because the loss disappears if prices return to the original ratio. However, if you withdraw while prices have diverged, the loss becomes **permanent**.

### Mathematical Derivation

Let us derive the IL formula from first principles for a 50/50 constant product pool.

**Setup:**
- Initial reserves: `x_0` of token X, `y_0` of token Y
- Initial price of X in terms of Y: `P_0 = y_0 / x_0`
- Invariant: `k = x_0 * y_0`

**After price change:**
- New external price: `P_1 = P_0 * r` where `r` is the price ratio (e.g., r=2 means price doubled)
- Arbitrageurs rebalance the pool until the pool price matches the external price

From the constant product invariant and the new price:

```
x_1 * y_1 = k                     ... (invariant)
y_1 / x_1 = P_1 = P_0 * r        ... (price condition)
```

Solving simultaneously:

```
y_1 = x_1 * P_0 * r
x_1 * (x_1 * P_0 * r) = k
x_1^2 = k / (P_0 * r)
x_1 = sqrt(k / (P_0 * r))
y_1 = sqrt(k * P_0 * r)
```

**Value of LP position at new price:**

```
V_LP = x_1 * P_1 + y_1
     = x_1 * P_0 * r + y_1
     = sqrt(k / (P_0 * r)) * P_0 * r + sqrt(k * P_0 * r)
     = sqrt(k * P_0 * r) + sqrt(k * P_0 * r)
     = 2 * sqrt(k * P_0 * r)
```

**Value if simply held (HODL):**

```
V_HODL = x_0 * P_1 + y_0
       = x_0 * P_0 * r + y_0
```

Since `P_0 = y_0 / x_0`, we have `x_0 * P_0 = y_0`, so:

```
V_HODL = y_0 * r + y_0 = y_0 * (1 + r)
```

And since `k = x_0 * y_0` and `P_0 = y_0 / x_0`:

```
k * P_0 = x_0 * y_0 * (y_0 / x_0) = y_0^2
sqrt(k * P_0) = y_0
```

Therefore:

```
V_LP = 2 * y_0 * sqrt(r)
V_HODL = y_0 * (1 + r)
```

**The Impermanent Loss ratio:**

```
IL = V_LP / V_HODL - 1
   = [2 * sqrt(r) / (1 + r)] - 1
```

### The Final IL Formula

```
IL(r) = 2*sqrt(r) / (1 + r) - 1
```

Where `r = P_new / P_initial` (the price ratio change).

Key properties:
- IL is always negative or zero (LPs always lose relative to holding)
- IL = 0 when r = 1 (no price change)
- IL depends only on the magnitude of price change, not direction (r=2 and r=0.5 give the same IL)
- IL is symmetric in log-price space

### IL at Common Price Multiples

| Price Change (r) | IL (%) | Description |
|---|---|---|
| 1.00x | 0.00% | No change |
| 1.25x | -0.60% | 25% price increase |
| 1.50x | -2.02% | 50% price increase |
| 1.75x | -3.77% | 75% price increase |
| 2.00x | -5.72% | Price doubles |
| 3.00x | -13.40% | Price triples |
| 4.00x | -20.00% | Price quadruples |
| 5.00x | -25.46% | 5x price increase |
| 10.00x | -42.50% | 10x price increase |
| 0.50x | -5.72% | Price halves (same as 2x) |
| 0.25x | -20.00% | Price quarters (same as 4x) |
| 0.10x | -42.50% | 90% crash (same as 10x) |

### Verification: IL at r = 2

```
IL(2) = 2*sqrt(2) / (1 + 2) - 1
      = 2*1.4142 / 3 - 1
      = 2.8284 / 3 - 1
      = 0.9428 - 1
      = -0.0572
      = -5.72%
```

### When Does IL Become Permanent?

IL is only "impermanent" while you remain in the pool. It becomes a realized (permanent) loss when:

1. **You withdraw** while prices have diverged from your entry ratio.
2. **The token goes to zero** -- if token X drops 100%, your entire position becomes worthless token X (the pool has sold all of token Y as arbitrageurs extracted it).
3. **The protocol is exploited** or the token is rugged -- the rebalancing mechanism cannot help you.

**Critical insight**: Even if fees earned exceed IL, the LP still underperformed what they could have earned by holding and separately earning fees (e.g., through lending). IL represents a **real opportunity cost** that must be weighed against fee income.

---

## 5. Uniswap v3 Concentrated Liquidity

### The Core Innovation

Uniswap v2 distributes liquidity uniformly across the entire price curve from 0 to infinity. This means most liquidity sits at prices far from the current price and is never used. For a DAI/USDC pool trading at ~1.00, the liquidity allocated to the price ranges 0.01-0.99 and 1.01-100 is essentially wasted capital.

Uniswap v3 (March 2021) introduces **concentrated liquidity**: LPs choose a specific price range `[P_a, P_b]` in which their capital is active. Within that range, the position behaves like a much larger v2 position. Outside the range, the position is inactive and earns no fees.

### Mathematical Foundation

Uniswap v3 tracks prices using the **square root of price** (`sqrt(P)`) rather than price directly. This simplifies the math for calculating liquidity and token amounts.

The key concept is **liquidity** (`L`), defined as:

```
L = sqrt(k) = sqrt(x * y)
```

For a position concentrated between prices `P_a` and `P_b`, the virtual reserves are:

```
x_virtual = L / sqrt(P)    (token X reserves)
y_virtual = L * sqrt(P)    (token Y reserves)
```

The real reserves (actual tokens deposited) for a position between `P_a` and `P_b` at current price `P`:

```
If P_a <= P <= P_b:
    x_real = L * (1/sqrt(P) - 1/sqrt(P_b))
    y_real = L * (sqrt(P) - sqrt(P_a))

If P < P_a (price below range -- position is entirely token X):
    x_real = L * (1/sqrt(P_a) - 1/sqrt(P_b))
    y_real = 0

If P > P_b (price above range -- position is entirely token Y):
    x_real = 0
    y_real = L * (sqrt(P_b) - sqrt(P_a))
```

### The Tick System

Uniswap v3 discretizes the price space into **ticks**. Each tick `i` maps to a price:

```
P(i) = 1.0001^i
```

This means each tick represents a 0.01% (1 basis point) price movement. Ticks provide:

- **Efficient storage**: Only ticks with liquidity changes need to be stored on-chain.
- **Gas optimization**: Swaps only need to iterate through active ticks.
- **Fee tier coupling**: Tick spacing is determined by the fee tier:
  - 0.01% fee -> tick spacing 1 (every tick available)
  - 0.05% fee -> tick spacing 10
  - 0.30% fee -> tick spacing 60
  - 1.00% fee -> tick spacing 200

Higher fee tiers have wider tick spacing because the expected volatility is higher and precise range placement is less critical.

### Capital Efficiency

The capital efficiency gain from concentration is dramatic. Consider a position concentrated in the range `[P_a, P_b]` vs. a v2 position (range [0, infinity]):

```
Capital Efficiency = 1 / (1 - sqrt(P_a / P_b))
```

**Example: DAI/USDC pool, range [0.999, 1.001]**

```
Efficiency = 1 / (1 - sqrt(0.999/1.001))
           = 1 / (1 - sqrt(0.998))
           = 1 / (1 - 0.999)
           = 1 / 0.001
           = 1000x
```

A $1,000 concentrated position provides the same depth as a $1,000,000 v2 position within that range. The theoretical maximum is approximately **4,000x** for the tightest possible range.

**Example: ETH/USDC pool, range [1200, 2800] with current price 2000**

```
Efficiency = 1 / (1 - sqrt(1200/2800))
           = 1 / (1 - sqrt(0.4286))
           = 1 / (1 - 0.6547)
           = 1 / 0.3453
           = 2.90x -> approximately 4.24x (varies with exact calculation method)
```

### Tradeoffs of Concentrated Liquidity

| Advantage | Disadvantage |
|---|---|
| Up to 4000x capital efficiency | Positions go out of range and stop earning |
| Higher fee income per dollar of capital | Amplified impermanent loss within range |
| Better execution for traders | Requires active management or automation |
| Custom risk/return profiles | LP tokens are non-fungible (NFT positions) |
| Multiple fee tiers | More complex to reason about |

### Position Management

Unlike v2 LP tokens (fungible ERC-20s), v3 positions are represented as **NFTs** because each position has unique parameters (tick range, liquidity amount). This makes:

- **Passive LPing harder**: Positions can go out of range and stop earning.
- **Active management necessary**: LPs must monitor and rebalance positions.
- **Vault protocols emerge**: Projects like Arrakis Finance (formerly G-UNI) and Gamma Strategies automate v3 position management.

---

## 6. Curve Finance and the StableSwap Invariant

### The Problem with CPMM for Stablecoins

For assets that should trade at approximately equal value (stablecoins, wrapped assets like WETH/ETH, liquid staking derivatives), the constant product formula is highly inefficient. A USDC/USDT pool at x*y=k has the same price impact curve as an ETH/BTC pool, even though USDC/USDT should rarely deviate from 1:1.

A **constant sum** formula `x + y = k` would provide zero slippage at any trade size, but it has a critical flaw: the pool can be completely drained of one token. If USDC trades at $1.001 and USDT at $0.999, arbitrageurs would drain all USDT from the pool.

### The StableSwap Invariant

Curve's innovation (Michael Egorov, 2020) is a **hybrid** invariant that combines constant sum and constant product:

```
A * n^n * sum(x_i) + D = A * D * n^n + D^(n+1) / (n^n * prod(x_i))
```

Where:
- `n` = number of tokens in the pool (typically 2-4)
- `x_i` = reserve of token i
- `D` = the total amount of tokens when they have equal price (the invariant value)
- `A` = the **amplification coefficient** (the key parameter)

For a two-token pool, this simplifies to:

```
2A(x + y) + D = 2AD + D^3 / (4xy)
```

### The Amplification Coefficient (A)

The `A` parameter controls the shape of the bonding curve:

- **A = 0**: The formula degenerates to the constant product formula (x*y = k). High slippage, never drains.
- **A = infinity**: The formula approaches the constant sum (x + y = k). Zero slippage near peg, but can drain completely.
- **Practical A values**: Typically 10-5000 depending on the pool. Stablecoin pools often use A = 100-2000.

**Behavior:**

| Pool State | Curve Behavior |
|---|---|
| Balanced (near 1:1) | Acts like constant sum -- extremely low slippage |
| Slightly imbalanced | Gentle increase in slippage |
| Highly imbalanced | Degrades toward constant product -- high slippage protects pool |

This gives Curve pools **1000x better capital efficiency** than Uniswap for stablecoin swaps when the pool is balanced, while still protecting against complete drainage when the pool becomes imbalanced.

### Visual Intuition

Imagine a 2D bonding curve:
- The constant product curve is a hyperbola (high curvature everywhere).
- The constant sum curve is a straight line (zero curvature).
- The StableSwap curve looks like a rounded rectangle: nearly flat (low slippage) in the middle where assets are balanced, curving sharply at the edges to prevent drainage.

### Practical Impact

For a $100,000 USDC-USDT swap:
- **Uniswap v2 (constant product)**: ~0.30% slippage + 0.30% fee
- **Curve (StableSwap, A=200)**: ~0.001% slippage + 0.04% fee
- **Improvement**: >100x less slippage on stablecoin pairs

### Curve v2 (CryptoSwap)

Curve v2 extends the StableSwap concept to volatile/non-pegged asset pairs by adding:
- **Internal price oracle**: Tracks a moving average of the price to know where to concentrate liquidity.
- **Dynamic peg**: The curve re-centers around the oracle price, concentrating liquidity where it is needed.
- **Repegging mechanism**: Automatically adjusts the curve shape as the price moves.

This allows Curve v2 to compete with Uniswap v3 for volatile pairs while maintaining simpler UX (LPs do not need to set ranges).

---

## 7. Order Book DEXes

### Architecture and Design

Order book DEXes maintain a sorted list of buy (bid) and sell (ask) orders. They differ from AMMs fundamentally:

| Property | AMM | Order Book |
|---|---|---|
| Price discovery | Algorithmic (from reserves) | From individual orders |
| Liquidity provision | Passive (deposit and wait) | Active (place and manage orders) |
| Capital efficiency | Low (v2) to High (v3) | Very high (capital only used when matched) |
| Execution quality | Predictable, always available | Better spreads when liquid, empty when not |
| Gas costs | One tx per swap | Multiple txs for place/cancel/match |
| Best for | Long tail assets, passive LPs | High-volume pairs, professional traders |

### Solana Order Book DEXes

Solana's low fees (~$0.0001/tx) and high throughput (~400ms slots) make it the most viable L1 for on-chain order books.

#### Serum / OpenBook

Serum was the first major on-chain CLOB on Solana, created by FTX/Alameda Research. After FTX's collapse, the community forked it as **OpenBook**.

Key characteristics:
- Full on-chain central limit order book
- Required a "crank" mechanism -- an external agent to trigger order matching and settlement
- Other Solana DEXes (including early Raydium) built on top of Serum's order book for deeper liquidity
- OpenBook v2 improved the design with better capital efficiency and composability

#### Phoenix

Phoenix (by Ellipsis Labs) represents the next generation of Solana order books:

- **Crankless design**: Orders are matched and settled atomically in a single transaction, eliminating the need for external cranking
- **Instant settlement**: Trades settle within the transaction, not asynchronously
- **On-chain events**: All market data is written on-chain, enabling transparent analytics
- **Composable**: Designed for easy integration with other Solana DeFi protocols
- **Market maker friendly**: Low fees for order placement and modification, enabling tighter spreads

#### Why Solana Enables On-Chain Order Books

| Feature | Ethereum | Solana |
|---|---|---|
| Block time | ~12 seconds | ~400ms |
| Transaction cost | $1-50+ | ~$0.0001-0.01 |
| Throughput | ~15-30 TPS | ~4,000+ TPS |
| Order book viability | Impractical | Practical |

### dYdX

dYdX is a perpetual futures DEX that evolved across multiple architectures:

- **v1-v3**: Built on Ethereum using StarkEx (ZK-rollup) for order matching off-chain with on-chain settlement
- **v4 (current)**: Migrated to its own **appchain** built on Cosmos SDK (CometBFT consensus), with a fully decentralized off-chain order book where each validator runs the matching engine in-memory

This represents a third model: neither pure on-chain order book nor AMM, but a purpose-built blockchain optimized for order book exchange.

---

## 8. Advanced AMM Designs

### 8.1 Weighted Pools (Balancer)

Balancer generalizes the constant product formula to support:
- **N tokens** (not just 2)
- **Arbitrary weights** (not just 50/50)

#### The Weighted Invariant

```
V = prod(B_i ^ W_i)    for i = 1 to n
```

Where:
- `B_i` = balance of token i
- `W_i` = normalized weight of token i (all weights sum to 1)
- `V` = the invariant (must be preserved)

For two tokens with 50/50 weights, this reduces to `sqrt(x * y) = V`, which is equivalent to `x * y = V^2 = k` (Uniswap).

#### Spot Price

```
SP_i_o = (B_i / W_i) / (B_o / W_o)
```

Where `SP_i_o` is the spot price of token i in terms of token o.

#### Swap Formula (Exact In)

```
A_o = B_o * [1 - (B_i / (B_i + A_i))^(W_i / W_o)]
```

Where `A_o` is the output amount and `A_i` is the input amount.

#### Key Use Cases

| Configuration | Use Case |
|---|---|
| 80/20 ETH/USDC | Reduced IL for ETH holders (~55% less IL than 50/50) |
| 60/20/20 three-token pool | Index fund-like exposure |
| 95/5 governance/ETH | Liquidity Bootstrapping Pool (LBP) for token launches |
| Managed pool | Actively rebalanced treasury management |

#### Liquidity Bootstrapping Pools (LBPs)

Balancer's weighted pools enable **Dutch auction-style token launches**:
1. Start with 95/5 weight (project token / collateral)
2. Gradually shift to 50/50 over hours/days
3. The weight change creates natural selling pressure, preventing early buyers from pumping
4. Fair price discovery without front-running

### 8.2 Virtual AMMs (Perpetual Protocol)

Virtual AMMs (vAMMs) use the constant product formula `x * y = k` for **price discovery only**, without actual token reserves.

#### How vAMMs Work

1. **No real reserves**: The "pool" is virtual. No tokens sit in a vAMM contract.
2. **Collateral vault**: All margin collateral (e.g., USDC) is held in a separate vault contract.
3. **Simulated trading**: When a trader opens a long with 10x leverage on ETH, the vAMM simulates buying ETH-perp against the virtual reserves, moving the price.
4. **Price discovery**: The x*y=k formula determines the entry/exit price based on position size relative to virtual liquidity.
5. **Settlement**: PnL is calculated when positions close and settled from the collateral vault.

#### Advantages and Limitations

```
Advantages:
+ No need for LP capital -- no impermanent loss for LPs
+ Supports leverage and short positions natively
+ Can create markets for any asset (just needs a price feed)
+ Capital efficient -- only margin required

Limitations:
- Vulnerable to price manipulation if virtual liquidity is too thin
- Requires reliable oracle price feeds for funding rate calculations
- "Virtual" liquidity means slippage can be arbitrary (set by governance)
- Liquidation cascades can cause extreme price dislocations
```

Perpetual Protocol v1 used pure vAMMs; v2 switched to concentrated liquidity on Uniswap v3 for real on-chain liquidity and better price execution.

### 8.3 Concentrated Liquidity Market Makers (CLMMs) on Solana

#### Orca Whirlpools

Orca's Whirlpools is the leading CLMM on Solana, inspired by Uniswap v3 but built natively for the Solana Virtual Machine (SVM).

**Architecture:**
- Written from scratch in Rust for the SVM (not a port of Solidity code)
- Optimized 256-bit integer arithmetic for price/liquidity calculations
- Double-audited smart contract (open source)
- Position represented as NFTs (similar to Uni v3)

**Key features:**
- Concentrated liquidity with custom price ranges
- Multiple fee tiers (e.g., 0.01%, 0.05%, 0.30%, 1.00%)
- Built-in yield farming (rewards distributed to in-range positions)
- Integrated into Jupiter aggregator for optimal routing

**Developer integration:**
- Whirlpools SDK available in TypeScript and Rust
- Open-source program deployed on both Solana and Eclipse
- Composable with other Solana DeFi protocols

**Key differences from Uniswap v3:**
- Solana's account model vs. Ethereum's storage model requires different data structures
- Tick arrays are stored in separate accounts (Solana's account size limits)
- Transactions include both compute budget and account access optimizations
- Lower fees enable more frequent position rebalancing

#### Raydium CLMM

Raydium's Concentrated Liquidity Market Maker is another major CLMM on Solana.

**Architecture:**
- Open-source smart contract on Solana
- Positions represented as NFTs
- Integrated with Raydium's broader ecosystem (standard AMM pools, AcceleRaytor launchpad)

**Key features:**
- Custom price ranges for liquidity provision
- Multiple fee tiers
- Dual yield: swap fees + farm rewards
- Position NFTs can be used in other DeFi protocols

**Key operational details:**
- If price trades below position's min price: position becomes 100% base token
- If price trades above max price: position becomes 100% quote token (base fully sold)
- LP must actively manage or use automation tools

**Risk profile:**
- Out-of-range positions earn zero fees
- Concentrated IL is amplified compared to full-range positions
- Lost or burned position NFTs mean permanent loss of liquidity
- Better suited for experienced users or automated vault strategies

#### Comparison: Orca Whirlpools vs. Raydium CLMM

| Feature | Orca Whirlpools | Raydium CLMM |
|---|---|---|
| Architecture | Purpose-built for SVM | Purpose-built for SVM |
| Source | Open source | Open source |
| Position type | NFT | NFT |
| Fee tiers | Multiple | Multiple |
| Yield farming | Native integration | Native integration |
| SDK | TypeScript + Rust | TypeScript + Rust |
| Aggregator integration | Jupiter primary | Jupiter primary |
| UX philosophy | Guided, simpler | More advanced/manual |

Both are first-class citizens in Solana's DeFi ecosystem and are routed through Jupiter aggregator, which finds the best price across all liquidity sources.

---

## 9. MEV and Sandwich Attacks

### What is MEV?

**Maximal Extractable Value (MEV)** is the profit that can be extracted by reordering, inserting, or censoring transactions within a block. On public blockchains, pending transactions are visible in the mempool (or equivalent), enabling sophisticated actors to profit at the expense of regular users.

### Types of MEV on DEXes

#### 1. Sandwich Attacks

The most common DEX-specific MEV attack:

```
Step 1: Attacker monitors mempool for a large swap (e.g., buy 10 ETH on Uniswap)
Step 2: FRONTRUN -- Attacker buys ETH before victim, pushing price up
Step 3: VICTIM's trade executes at the now-inflated price
Step 4: BACKRUN -- Attacker sells ETH at the higher post-victim price

Result: Victim gets fewer tokens than expected; attacker profits the difference.
```

**Numerical example:**

```
Pool: 100 ETH / 200,000 USDC (price: 2,000 USDC/ETH)

Victim wants to buy 5 ETH (submitted with 2% slippage tolerance)

1. Attacker buys 2 ETH:
   New pool: 98 ETH / 204,081.63 USDC
   Cost: 4,081.63 USDC (effective price: 2,040.82)

2. Victim buys 5 ETH:
   New pool: 93 ETH / 215,247.31 USDC
   Cost: 11,165.68 USDC (effective price: 2,233.14)
   Without sandwich: would have paid ~10,526.32 (effective: 2,105.26)
   Victim overpays: ~$639

3. Attacker sells 2 ETH:
   New pool: 95 ETH / 210,786.59 USDC
   Receives: 4,460.72 USDC
   Profit: 4,460.72 - 4,081.63 = $379.09 (minus gas)
```

#### 2. Arbitrage

Less harmful -- arbitrageurs align prices across pools/exchanges:

```
Pool A: ETH = 2,000 USDC
Pool B: ETH = 2,050 USDC

Arbitrageur: Buy from A, sell on B, pocket $50 minus gas
This brings both pools to ~2,025 USDC
```

Arbitrage is generally considered "good MEV" because it improves price accuracy, but it does extract value from LPs (it is the mechanism that causes impermanent loss).

#### 3. Just-In-Time (JIT) Liquidity

Specific to concentrated liquidity AMMs (Uniswap v3, Orca Whirlpools):

```
1. Bot sees a large pending swap
2. Bot adds highly concentrated liquidity at the exact current tick
3. Swap executes, paying fees primarily to the bot's position
4. Bot removes liquidity immediately after

Result: Regular LPs earn fewer fees; JIT provider captures most fees.
```

#### 4. Liquidation MEV

Not DEX-specific but related: bots compete to be the first to liquidate undercollateralized positions on lending protocols, earning liquidation bonuses.

### MEV on Solana

Solana's architecture creates unique MEV dynamics:

**Key differences from Ethereum:**
- No public mempool (transactions go directly to the leader validator)
- Continuous block production (~400ms slots)
- Validators see transactions before they are finalized
- Lower latency creates a speed game rather than auction game

**Jito's Role:**

Jito provides MEV infrastructure on Solana:

- **Jito Bundles**: Allows users to submit atomic bundles of transactions with tips. This enables:
  - **MEV extraction** (searchers can frontrun/backrun atomically)
  - **MEV protection** (users can tip to have transactions processed privately)
- **Tip amount**: Typical tips are ~$0.04 per transaction during normal conditions
- **Protection mechanism**: Transactions submitted via Jito bundles are not visible to other searchers before execution

**Scale of the problem on Solana:**
- Sandwich bots extracted $370-500 million over a ~16 month period (2024-2025)
- Over 500,000 sandwich attack instances identified in early 2025
- "Wide sandwiches" (multi-slot attacks) account for 93% of sandwiches and extracted 529,000+ SOL in one year

**Multi-layered protection (2025+):**
- **Jito bundles**: Private transaction submission
- **Jito "Don't Front"**: Special account tag that rejects bundles where the tagged transaction is not first
- **Paladin + bloXroute**: Additional propagation channels for redundancy
- **Priority fees**: Higher priority fees reduce the window for MEV extraction

### Protecting Against Sandwich Attacks

As a developer building on Solana:

1. **Use Jupiter with slippage protection**: Jupiter's routing minimizes price impact and enforces slippage limits.
2. **Submit via Jito bundles**: Private transaction submission prevents mempool snooping.
3. **Set tight slippage tolerances**: Smaller slippage = smaller sandwich profit = less likely to be attacked.
4. **Split large trades**: Smaller trades have less MEV opportunity.
5. **Use limit orders when possible**: Order book trades on Phoenix cannot be sandwiched in the same way.
6. **Time-weighted trades**: For large positions, DCA or TWAP over multiple blocks.

---

## 10. The Fee Structure

### How LP Fees Work

LP fees are the primary incentive for providing liquidity. They are charged on every swap and distributed to liquidity providers proportional to their share of the pool.

#### Fee Tiers

Most modern DEXes offer multiple fee tiers:

| Fee Tier | Typical Use Case | Uniswap v3 Tick Spacing |
|---|---|---|
| 0.01% | Stablecoin/stablecoin (e.g., USDC/USDT) | 1 |
| 0.05% | Correlated pairs (e.g., WETH/stETH) | 10 |
| 0.30% | Standard volatile pairs (e.g., ETH/USDC) | 60 |
| 1.00% | Exotic/high-volatility pairs | 200 |

**The tradeoff:** Lower fees attract more volume but compensate LPs less per trade. Higher fees compensate LPs more but may push traders to other venues.

**Mathematical relationship:** A 0.01% fee pool requires **625x** the volume of a 1.00% fee pool to generate the same fee revenue:

```
Revenue = Volume * Fee Rate
For equal revenue: V_1 * 0.01% = V_2 * 1.00%
V_1 / V_2 = 1.00% / 0.01% = 100x

Wait, for 0.05% vs 1%: 1.00/0.05 = 20x
For 0.01% vs 1%: 1.00/0.01 = 100x
```

#### Fee Application Mechanics

In Uniswap v2 / constant product AMMs:

```
1. Trader submits swap: 1,000 USDC -> ETH
2. Fee deducted from input: 1,000 * 0.003 = 3 USDC fee
3. Effective input: 1,000 - 3 = 997 USDC
4. Constant product applied to 997 USDC
5. Fee (3 USDC) stays in pool, increasing reserves
6. k increases slightly with each trade
```

In Uniswap v3 / concentrated liquidity:

```
1. Fee is still deducted from input
2. Fee is allocated to in-range positions proportionally to their liquidity
3. Fees accrue per-position and must be claimed (or are auto-compounded by vault protocols)
4. Out-of-range positions earn zero fees
```

### Protocol Fees

Protocol fees are a portion of trading fees directed to the protocol treasury (controlled by governance).

| Protocol | LP Fee | Protocol Fee | Total Swap Fee |
|---|---|---|---|
| Uniswap v2 | 0.30% | 0% (optional 0.05%) | 0.30% |
| Uniswap v3 | Variable | 10-25% of LP fee (governance) | Varies by tier |
| Curve | ~0.04% | 50% of admin fee | 0.04% |
| Orca | Variable | Variable | Varies by tier |
| Raydium | Variable | ~12% of swap fee | Varies by tier |
| Balancer | Variable | 50% of swap fee (V2) | Varies by pool |

**Uniswap's "Fee Switch" debate:** Uniswap v2 included a dormant protocol fee mechanism (0.05% of the 0.30% fee could be redirected to a treasury). This "fee switch" was a major governance topic -- activating it would reduce LP earnings by ~16.7%, potentially driving liquidity away, but would generate protocol revenue.

### Fee Accumulation: v2 vs. v3

**Uniswap v2:**
- Fees increase reserves -> k grows -> LP token value increases
- No need to "claim" -- value accrues automatically
- Fungible LP tokens can be transferred, staked, used as collateral

**Uniswap v3 / CLMMs:**
- Fees tracked per-position (per-NFT)
- Must be collected/claimed via transaction
- Can be auto-compounded by vault protocols (Arrakis, Kamino on Solana)
- Fee income depends entirely on whether position is in-range

### Solana-Specific Fee Considerations

On Solana, the extremely low transaction costs (~$0.0001 base fee) change the fee dynamics:

- **Lower minimum viable fee tiers**: 0.01% tiers are viable even for moderate volume
- **More frequent rebalancing**: LPs can adjust positions cheaply
- **Compounding frequency**: Auto-compounders can compound more frequently without gas eating into returns
- **Priority fees for MEV**: During high activity, priority fees can be significant (but still far cheaper than Ethereum)

---

## 11. Solana DEX Ecosystem Overview

### Jupiter Aggregator

Jupiter is the dominant DEX aggregator on Solana. It does not hold liquidity itself but routes trades through all available liquidity sources to find the optimal execution.

**Routing Architecture:**
- **Juno**: Top-level aggregator combining multiple routing engines
- **Iris**: Latest routing engine optimized for best execution across all Solana liquidity
- **JupiterZ**: RFQ (Request for Quote) system connecting directly to market makers
- **Multi-hop routing**: Splits single trades across multiple pools for better composite prices

**Integrated liquidity sources:**
- Orca Whirlpools (CLMM)
- Raydium (CLMM + standard AMM)
- Phoenix (order book)
- OpenBook (order book)
- Lifinity (proactive market maker)
- Meteora (dynamic AMM)
- And many more

**Additional features:**
- Limit orders
- DCA (Dollar Cost Averaging)
- Perpetual futures (Jupiter Perps)
- Token launchpad

### Recommended Architecture for Solana DeFi Development

When building a DeFi application on Solana that needs trading functionality:

1. **For swaps**: Integrate Jupiter's API/SDK for optimal routing across all venues
2. **For liquidity provision**: Integrate with Orca Whirlpools or Raydium CLMM SDKs directly
3. **For order book trading**: Integrate Phoenix for professional-grade limit orders
4. **For MEV protection**: Use Jito bundles for transaction submission
5. **For automated LP management**: Consider Kamino Finance (auto-rebalancing CLMM vaults)

---

## 12. References

### Whitepapers and Primary Sources

1. [Uniswap v2 Core Whitepaper](https://app.uniswap.org/whitepaper.pdf) - Hayden Adams, Noah Zinsmeister et al.
2. [Uniswap v3 Core Whitepaper](https://app.uniswap.org/whitepaper-v3.pdf) - Hayden Adams, Noah Zinsmeister et al., March 2021
3. [StableSwap Whitepaper](https://berkeley-defi.github.io/assets/material/StableSwap.pdf) - Michael Egorov
4. [Liquidity Math in Uniswap v3 Technical Note](https://atiselsts.github.io/pdfs/uniswap-v3-liquidity-math.pdf) - Atis Elsts

### Protocol Documentation

5. [Uniswap v2 Documentation](https://docs.uniswap.org/contracts/v2/concepts/protocol-overview/how-uniswap-works)
6. [Uniswap v3 Concentrated Liquidity Docs](https://docs.uniswap.org/concepts/protocol/concentrated-liquidity)
7. [Curve Finance Technical Docs](https://docs.curve.finance/stableswap-exchange/overview/)
8. [Balancer Weighted Math Documentation](https://docs.balancer.fi/concepts/explore-available-balancer-pools/weighted-pool/weighted-math.html)
9. [Balancer Weighted Pools](https://docs-v2.balancer.fi/concepts/pools/weighted.html)
10. [Orca Whirlpools Documentation](https://docs.orca.so/)
11. [Orca Developer Documentation](https://dev.orca.so/)
12. [Raydium CLMM Documentation](https://docs.raydium.io/raydium/for-liquidity-providers/pool-types/clmm-concentrated)
13. [Raydium Concentrated Liquidity Intro](https://docs.raydium.io/raydium/liquidity-providers/providing-concentrated-liquidity-clmm/intro-on-concentrated-liquidity)
14. [Jupiter Routing Documentation](https://dev.jup.ag/docs/routing)
15. [Jito MEV Documentation](https://docs.jito.wtf/lowlatencytxnsend/)

### Source Code

16. [Orca Whirlpools GitHub](https://github.com/orca-so/whirlpools) - Open source CLMM on Solana
17. [Raydium CLMM GitHub](https://github.com/raydium-io/raydium-clmm) - Open source CLMM on Solana

### Technical Deep Dives

18. [Uniswap v3 Math Primer](https://blog.uniswap.org/uniswap-v3-math-primer) - Official Uniswap Labs blog
19. [Uniswap v3 Ticks Deep Dive](https://mixbytes.io/blog/uniswap-v3-ticks-dive-into-concentrated-liquidity) - MixBytes
20. [Concentrated Liquidity in Uniswap v3](https://rareskills.io/post/uniswap-v3-concentrated-liquidity) - RareSkills
21. [Uniswap v2 Mint and Burn Functions](https://rareskills.io/post/uniswap-v2-mint-and-burn) - RareSkills
22. [Curve StableSwap Mathematical Guide](https://xord.com/research/curve-stableswap-a-comprehensive-mathematical-guide/) - Xord
23. [Understanding StableSwap (Curve)](https://miguelmota.com/blog/understanding-stableswap-curve/) - Miguel Mota
24. [Understanding the Curve AMM StableSwap Invariant](https://atulagarwal.dev/posts/curveamm/stableswap/) - Atul Agarwal

### Impermanent Loss

25. [Impermanent Loss Full Derivation](https://medium.com/auditless/how-to-calculate-impermanent-loss-full-derivation-803e8b2497b7) - Peteris Erins
26. [Impermanent Loss Explained with Math](https://chainbulletin.com/impermanent-loss-explained-with-examples-math) - The Chain Bulletin
27. [Impermanent Loss Math Explained](https://speedrunethereum.com/guides/impermanent-loss-math-explained) - Speedrun Ethereum
28. [Understanding Returns in Uniswap v2](https://docs.uniswap.org/contracts/v2/concepts/advanced-topics/understanding-returns)

### MEV and Security

29. [Understanding MEV Sandwich Attacks FAQ](https://www.carbondefi.xyz/blog/understanding-mev-sandwich-attacks-frequently-asked-questions) - Carbon DeFi
30. [Quantifying the Threat of Sandwiching MEV on Jito](https://cnitarot.github.io/papers/imc26_solana.pdf) - IMC 2026 Research Paper
31. [Solana MEV Exposed: Sandwich Attacks Analysis](https://solanacompass.com/learn/accelerate-25/scale-or-die-at-accelerate-2025-the-state-of-solana-mev) - Solana Compass
32. [Solana MEV Economics: Jito, Bundles, and Liquid Staking](https://blog.quicknode.com/solana-mev-economics-jito-bundles-liquid-staking-guide/) - QuickNode
33. [Sandwich Attack - MEV Wiki](https://www.mev.wiki/attack-examples/sandwich-attack)

### AMM Comparisons and Analysis

34. [Constant Function Market Makers: DeFi's Zero to One Innovation](https://medium.com/bollinger-investment-group/constant-function-market-makers-defis-zero-to-one-innovation-968f77022159) - Dmitriy Berenzon
35. [Understanding AMMs Part 1: Price Impact](https://research.paradigm.xyz/amm-price-impact) - Paradigm Research
36. [AMMs: Math, Risks & Solidity Code](https://speedrunethereum.com/guides/automated-market-makers-math) - Speedrun Ethereum
37. [AMM vs Order Book DEXs](https://cryptorank.io/insights/analytics/amm-vs-order-book-dexs) - CryptoRank
38. [Order Book vs AMM: Which Model Works Better](https://coinbureau.com/education/order-book-vs-automated-market-maker) - Coin Bureau
39. [Deep Dive into Virtual AMMs (vAMM)](https://gov.perp.fi/t/deep-dive-into-our-virtual-amm-vamm/38) - Perpetual Protocol
40. [Automated Market Makers Explained](https://chain.link/education-hub/what-is-an-automated-market-maker-amm) - Chainlink

---

*Document compiled February 2026. DeFi protocols evolve rapidly -- always verify current implementations against official documentation.*
