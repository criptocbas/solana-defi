# Advanced DeFi Strategies, Mechanisms, and Emerging Trends

> A deep-dive reference for experienced Solana developers entering DeFi.
> Covers yield strategies, derivatives, MEV, bridges, composability, and 2024-2026 trends.
> Last updated: February 2026.

---

## Table of Contents

1. [Yield Strategies: How People Actually Make Money in DeFi](#1-yield-strategies-how-people-actually-make-money-in-defi)
2. [Liquid Staking and Restaking](#2-liquid-staking-and-restaking)
3. [Perpetual DEXes and Derivatives](#3-perpetual-dexes-and-derivatives)
4. [MEV Deep Dive](#4-mev-deep-dive)
5. [Cross-Chain DeFi and Bridges](#5-cross-chain-defi-and-bridges)
6. [DeFi Composability in Practice](#6-defi-composability-in-practice)
7. [Emerging DeFi Trends (2024-2026)](#7-emerging-defi-trends-2024-2026)
8. [References](#8-references)

---

## 1. Yield Strategies: How People Actually Make Money in DeFi

### 1.1 Looping / Recursive Leverage

**What it is:** Deposit collateral into a lending protocol, borrow against it, swap the borrowed asset back into collateral, re-deposit, and repeat. Each "loop" amplifies your effective exposure and the yield earned on the deposited asset.

**The mechanics step by step:**

```
Initial: Deposit 100 SOL into Kamino Lend (earns ~4% supply APY)
Loop 1:  Borrow 70 SOL worth of USDC (at 70% LTV) -> swap to SOL -> deposit
Loop 2:  Borrow against new 70 SOL -> swap -> deposit
Loop 3:  Borrow against new 49 SOL -> swap -> deposit
...
Result:  ~230 SOL effective exposure from 100 SOL initial capital (~2.3x leverage)
```

**Flash-loan accelerated looping:** Instead of multiple transactions, a single atomic transaction uses a flash loan to achieve the same result:

1. Take a flash loan for the maximum borrowable amount
2. Swap the flash loan proceeds into the desired collateral asset
3. Deposit all collateral (original + flash loan proceeds) into the lending protocol
4. Borrow against the full position to repay the flash loan
5. Net result: fully leveraged position created in one transaction

**Real yield numbers (2025):**
- stETH looping on Aave: The classic "stETH-ETH loop" historically earned ~8-12% APY when ETH staking yield was ~4% and borrow rates were ~2%. This strategy collapsed when borrow rates exceeded staking yield.
- JitoSOL looping on Kamino/Drift: With JitoSOL earning 7-9% and SOL borrow rates at ~3-5%, a 2x loop targets ~10-15% APY.
- Stablecoin looping with point incentives: Often the real draw is protocol points/airdrops, not raw APY.

**Risk profile:**
- Liquidation risk when health factor drops below 1.0. At 2.3x leverage with 70% LTV, a ~13% price drop triggers liquidation.
- Borrow rate spikes can turn profitable loops negative overnight.
- Smart contract risk is amplified because you have exposure to both the lending protocol and the swap venue.

**Safety guidelines:**
- For 70% collateral factor, keep leverage at 2-2.5x maximum
- Maintain 10-15% of capital in reserves for emergency debt repayment
- Monitor health factor continuously; set up alerts or use automated deleveraging tools
- Understand that gas/transaction costs on high-frequency rebalancing eat into returns

> **Sources:** [Contango - Looping Deep Dive](https://medium.com/contango-xyz/what-is-looping-78421c8a1367), [Airdrop Alert - Looping Strategies](https://airdropalert.com/blogs/looping-strategies-in-defi-how-to-multiply-yield-and-farm-airdrops/), [Drift x Allora AI Looping](https://www.allora.network/blog/drift-x-allora-ai-powered-looping-strategy)

---

### 1.2 Delta-Neutral Strategies

**What it is:** Earning yield from liquidity provision or farming while hedging away all price exposure ("delta") through offsetting short positions. The goal is to capture trading fees and incentives while being insensitive to whether the underlying asset goes up or down.

**How it works in practice:**

```
Step 1: Provide liquidity to a SOL/USDC pool on Orca (concentrated liquidity)
        -> Earns swap fees (~15-40% APY in volatile markets)
        -> But you now have LONG exposure to SOL (impermanent loss risk)

Step 2: Open a SHORT SOL perpetual position on Drift/Jupiter Perps
        -> Size it to match your SOL exposure from the LP position
        -> Now your net delta is ~0 (price-neutral)

Net result: You earn swap fees minus funding rate payments and rebalancing costs
```

**The math of hedging:**
- A standard x*y=k AMM LP position has a delta of approximately 0.5 (half the exposure of holding the asset outright)
- A concentrated liquidity position has higher delta within its range
- The hedge size must be dynamically adjusted as the price moves ("delta drifts")
- A 1% ETH drop hurts a hedged LP by only ~0.02%, compared to ~0.5% for an unhedged LP

**Key challenges:**
- Rebalancing frequency: Delta drifts as prices move. Too frequent rebalancing = high costs. Too infrequent = significant unhedged exposure.
- Funding rate risk: If you are short perps and funding turns negative (shorts pay longs), you're paying to maintain the hedge.
- Basis risk: The LP token price and the perp price may diverge.
- Complexity: Requires monitoring two positions across two protocols simultaneously.

**Protocols automating this:**
- Teahouse Finance: Automated delta-neutral vaults
- Arcana Finance: Liquidity pool hedging with automated rebalancing
- Cetra Finance: Delta-neutral strategies across multiple DEXes

**Realistic returns:** 10-25% APY on stablecoin equivalent after hedging costs, depending on market volatility (higher volatility = more swap fees = higher returns).

> **Sources:** [Panoptic - Delta-Neutral LP](https://panoptic.xyz/blog/delta-neutral-lp-hedge-uniswap-position), [Cetra Finance](https://medium.com/@cetrafinance/delta-neutral-defi-strategies-overview-d0617561a1f4), [Arcana Finance](https://docs.arcana.finance/delta-neutral-yield-strategies/liquidity-pool-hedging)

---

### 1.3 Basis Trading (Spot vs. Perpetual Arbitrage)

**What it is:** Exploiting the price difference ("basis") between spot markets and perpetual futures. When perp prices trade above spot (contango), you buy spot and short perps, collecting funding rate payments. This is considered one of the "cleanest edges" in crypto.

**The core trade:**

```
1. Buy 1 BTC spot at $60,000           (long exposure)
2. Short 1 BTC perpetual at $60,150     (short exposure, equal size)
3. Net exposure: ~zero (market-neutral)
4. Collect funding payments every 8 hours (longs pay shorts when perp > spot)
```

**Funding rate economics (2025):**
- Average funding rates: ~0.015% per 8-hour period for major pairs (BTC, ETH, SOL)
- Annualized: ~16-20% in bull markets, ~5-8% in neutral markets
- BTC+ETH combined funding rate averaged ~11% annualized in 2024, ~5% in 2025
- Volatility of returns: 8-15% annually vs. 80%+ for directional crypto

**Ethena's USDe: The basis trade at protocol scale:**

Ethena is the largest implementation of the basis trade, running it as the backing mechanism for a synthetic dollar stablecoin:

```
User deposits ETH/BTC/SOL collateral
  -> Ethena stakes the ETH (earns staking yield ~3-4%)
  -> Opens matching short perpetual position (earns funding rates)
  -> Mints USDe stablecoin against the delta-neutral position
  -> sUSDe (staked USDe) passes yield through to holders
```

- USDe grew to $15 billion in circulation within two years
- sUSDe annualized yield since inception: ~10%+ (variable: 4-15% in 2025)
- Revenue sources: perpetual futures funding rates + ETH staking yield + interest on stablecoin reserves (via BlackRock's BUIDL)
- Key risk: prolonged negative funding rates would erode the backing

**On-chain basis trading (doing it yourself):**
- Deposit SOL into JitoSOL (earn ~7-9% staking + MEV yield)
- Short SOL perps on Jupiter Perps or Drift (earn funding when positive)
- Combined yield: staking yield + funding rate - borrow costs
- Use Pendle to lock in fixed yields on the basis position

> **Sources:** [Gate.com - Funding Rate Arbitrage 2025](https://www.gate.com/learn/articles/perpetual-contract-funding-rate-arbitrage/2166), [FXEmpire - Spot-Perp Arbitrage](https://www.fxempire.com/education/article/market-neutral-crypto-income-made-simple-spot-perp-arbitrage-strategy-explained-1535487), [Ethena Mechanics - CoinMetrics](https://coinmetrics.substack.com/p/state-of-the-network-issue-335), [Multicoin - Ethena Analysis](https://multicoin.capital/2025/11/13/ethena-synthetic-dollars-challenge-stablecoins-duopoly/)

---

### 1.4 JLP-Style Strategies (LP-as-Counterparty)

**What it is:** Jupiter Liquidity Provider (JLP) token holders act as the "house" against perpetual futures traders on Jupiter Perps. You deposit assets into a multi-asset pool and earn fees from all trading activity. When traders lose, you gain (and vice versa).

**How JLP works:**

```
JLP Pool Composition:
  - SOL  (~40-45%)
  - ETH  (~10%)
  - WBTC (~10%)
  - USDC (~25-30%)
  - USDT (~5-10%)

Revenue streams for JLP holders:
  1. Trading fees from perpetual futures trades
  2. Swap fees from spot trading
  3. Hourly borrow fees from leveraged traders
  4. Native staking yield on SOL held in pool
  5. JupUSD yield (migrated from USDC portion)
```

**The counterparty dynamic:**
- When traders are net short and market goes up: traders profit, JLP loses
- When traders are net long and market goes down: traders lose, JLP profits
- Historically, retail traders lose money on aggregate, so the "house" (JLP) tends to profit over time
- This is analogous to being the casino, not the gambler

**Yield numbers:**
- JLP has historically earned ~30-60% APY during high-volume periods
- During lower volume: ~15-25% APY
- Yield is highly variable and depends on trading volume and trader PnL

**2025 developments:**
- JLP Loans: JLP holders can now deposit JLP as collateral to borrow USDC, enabling leverage on the JLP position itself
- JupUSD integration: A portion of USDC migrated to JupUSD, with generated yield flowing back into the pool
- JLP can be used across Solana DeFi as collateral (Kamino, Marginfi, etc.)

**Risks:**
- Adverse selection: If traders are systematically profitable, JLP holders lose
- Asset price exposure: JLP is a basket of volatile assets (SOL, BTC, ETH), so it has directional risk
- Smart contract risk across Jupiter's perpetual engine
- The pool can experience drawdowns during sharp market moves where traders are profitable

**Similar models:**
- GMX/GLP on Arbitrum: The original LP-as-counterparty model. GLP is a multi-asset pool with zero impermanent loss (no x*y=k curve) that acts as counterparty to GMX traders. Uses oracle pricing, so trades execute with zero slippage.
- Gains Network (gTrade): Similar concept on Polygon/Arbitrum
- Flash Trade on Solana: Asset-backed perpetuals with composability focus

> **Sources:** [CoinMarketCap - JLP Explained](https://coinmarketcap.com/cmc-ai/jupiter-perps-lp/what-is/), [Jupiter JLP PDF](https://assets.ctfassets.net/i0qyt2j9snzb/4ueTPh7PwgopwILuLbhoYL/2fc57577db5f223edfaacf20bd557de1/JLP_-_Jupiter-s_Juicy_Yield.pdf), [GMX GLP Analysis](https://itsa-global.medium.com/itsa-defi-insight-gmx-on-chain-perpetuals-and-glp-935bb3168f0a)

---

### 1.5 Vault Strategies (Automated Yield Optimization)

**What it is:** Smart contracts that accept user deposits and automatically execute yield strategies, compounding returns and socializing gas costs across all depositors. Think "robo-advisors for DeFi."

**How vaults work:**

```
User deposits USDC into a Vault
  -> Vault strategy deploys across protocols:
     - 40% into Aave lending (earning supply rate)
     - 30% into Curve stablecoin pool (earning swap fees + CRV rewards)
     - 20% into Morpho optimized lending (earning better rates via P2P matching)
     - 10% kept liquid for withdrawals
  -> Keeper bots auto-compound rewards (harvest CRV -> sell -> reinvest)
  -> Rebalance across strategies when rates shift
```

**Vault categories:**
1. **Lending vaults:** Deposit into money markets, auto-compound interest (Yearn, Morpho)
2. **LP vaults:** Manage concentrated liquidity positions, rebalance ranges (Kamino, Meteora)
3. **Leveraged vaults:** Automated looping strategies with risk management (Contango)
4. **Multi-strategy vaults:** Rotate capital across strategies based on conditions (Summer.fi Lazy Summer)
5. **Delta-neutral vaults:** LP + hedge positions managed together (Teahouse, Arcana)

**Market scale (2025):**
- Aggregate automated vault AUM: ~$17.5 billion (surpassing 2021 DeFi Summer highs)
- Morpho: Over $9.4 billion TVL, the largest vault infrastructure protocol
- Yearn Finance: Continues with strategies built around Aave, Curve, and Convex

**Agent-powered vaults (the 2025 innovation):**
- Summer.fi's Lazy Summer: AI keeper agents monitor spreads and reallocate capital automatically, including cross-chain rebalancing
- kpk's dual-agent architecture: A "Rebalancing Agent" optimizes returns while an "Exit Agent" moves funds to safety when risk signals emerge
- Drift x Allora: AI-powered looping that adjusts leverage based on predictive market intelligence

**Yield ranges:**
- Stablecoin vaults: 5-15% APY depending on market conditions
- ETH/SOL vaults: 10-30% APY (includes token incentives)
- Leveraged vaults: 15-40% APY (with proportionally higher risk)

> **Sources:** [Keyrock - Automated Yield Strategies](https://keyrock.com/knowledge-hub/automated-onchain-yield-strategies-a-guide/), [Zircuit - DeFi Vaults](https://www.zircuit.com/blog/defi-vaults-how-they-work-and-yield-strategies), [Alchemy - DeFi Yield Aggregators](https://www.alchemy.com/dapps/best/defi-yield-aggregators)

---

### 1.6 Carry Trades in DeFi

**What it is:** Borrowing at a low interest rate and deploying capital into a higher-yielding opportunity, pocketing the spread. The DeFi version of the traditional finance carry trade.

**Examples of DeFi carry trades:**

```
Strategy 1: Stablecoin Rate Differential
  Borrow USDC on Aave at ~4% APY
  Deposit into Ethena sUSDe earning ~10% APY
  Net carry: ~6% APY (minus gas and smart contract risk)

Strategy 2: Cross-Protocol Rate Arbitrage
  Borrow USDT on Morpho at 3%
  Deposit into Colend (Core blockchain) earning 21-22% APY
  Borrow CORE tokens at 4% -> deposit in vault earning 15%
  Combined yield: 30%+ APY (carries bridging and protocol risk)

Strategy 3: Fixed vs Variable Rate Carry
  Borrow at variable rate on Aave (~4%)
  Lock in fixed rate via Pendle PT tokens (~8%)
  Net carry: ~4% APY with rate certainty on the long side

Strategy 4: LST Carry
  Borrow SOL at ~3-5% variable rate
  Stake into JitoSOL earning ~7-9%
  Net carry: ~2-6% APY (amplifiable via looping)
```

**Pendle: The DeFi fixed-income layer:**

Pendle is critical infrastructure for carry trades because it enables fixed-rate positions:

- **Principal Tokens (PT):** Represent the principal of a yield-bearing asset redeemable at maturity. Buy PT at a discount = lock in a fixed yield. Example: Buy PT-stETH at 0.95 ETH, redeem for 1 ETH at maturity = 5% guaranteed return.
- **Yield Tokens (YT):** Represent the future yield stream. Buy YT to speculate on yields increasing. Sell YT to lock in current yields.
- Pendle TVL grew from ~$5B to over $8.9B in a single month (August 2025)
- Pendle Boros (V3): Margin-enabled yield trading launched on Arbitrum in early 2025

**Stablecoin yield landscape (2025):**
- Conservative: 4-6% APY (Aave, Compound - USDC/USDT lending)
- Moderate: 8-15% APY (Ethena sUSDe, Pendle PT positions, Morpho optimized)
- Aggressive: 15-30%+ APY (leveraged strategies, exotic protocols, point farming)
- Yield-bearing stablecoins: Elevated US interest rates made Treasury-backed stablecoins attractive, with reserves invested in short-term Treasury instruments

> **Sources:** [Transfi - Stablecoin Yields 2025](https://www.transfi.com/blog/stablecoin-yields-in-2025-mapping-risk-return-and-protocol-dominance), [Pendle Documentation](https://docs.pendle.finance/Introduction), [Nansen - Pendle Explained](https://www.nansen.ai/post/what-is-pendle-finance-yield-tokenization-explained-how-to-earn)

---

## 2. Liquid Staking and Restaking

### 2.1 How Liquid Staking Works

**The problem it solves:** Native staking locks your SOL for an epoch (~2-3 days). During that time, you cannot use the capital for anything else. Liquid staking solves this by giving you a derivative token that represents your staked position.

**The flow:**

```
User deposits 100 SOL into Marinade Finance
  -> SOL is delegated to a diversified set of validators
  -> User receives ~100 mSOL (exchange rate appreciates over time to reflect staking rewards)
  -> mSOL can be:
     - Used as collateral on Kamino/Marginfi/Solend
     - Provided as liquidity in mSOL/SOL pools on Orca
     - Traded on any Solana DEX
     - Used in leveraged yield strategies
  -> When user wants SOL back: swap mSOL -> SOL or unstake (2-3 day delay)
```

**How the exchange rate works:**
- At launch: 1 mSOL = 1 SOL
- After one year at 7% staking yield: 1 mSOL = ~1.07 SOL
- The mSOL token price increases relative to SOL over time
- This is a "rebase by exchange rate" model (not a token supply increase)

### 2.2 Major Solana LSTs (2025)

| Token | Protocol | APY | Differentiator | TVL |
|-------|----------|-----|-----------------|-----|
| JitoSOL | Jito | 7-9% | MEV rewards included (+1-2%) | Largest by TVL |
| mSOL | Marinade | 6-7% | Pioneer, diversified validator set | Second largest |
| bSOL | BlazeStake | 6-7% | Community-focused, governance | Growing |
| INF | Sanctum | 7-8% | Multi-LST aggregator, best yield | Fastest growing |
| stSOL | Lido | 6-7% | Cross-chain brand | Declining share |
| jupSOL | Jupiter | 7-8% | Jupiter ecosystem integration | New entrant |

**Solana liquid staking market (2025):**
- Total TVL: Over $10.7 billion
- 13.3% of all staked SOL is liquid (57 million SOL)
- JitoSOL dominates due to MEV yield integration

### 2.3 The Liquid Staking Yield Stack

The real power of liquid staking is composability. Here is how DeFi users "stack" yields:

```
Layer 1: Base Staking Yield
  Stake SOL -> receive JitoSOL (7-9% APY from staking + MEV)

Layer 2: Lending Yield
  Deposit JitoSOL as collateral on Kamino Lend
  Borrow USDC at 4% -> deploy elsewhere
  Effective additional yield: spread between USDC deployment and borrow rate

Layer 3: Liquidity Provision
  Provide JitoSOL/SOL liquidity on Orca
  Earn swap fees (~5-15% APY additional)
  Minimal impermanent loss since JitoSOL/SOL are correlated

Layer 4: Leverage Loop
  Deposit JitoSOL -> borrow SOL -> stake into more JitoSOL -> repeat
  Each loop amplifies the staking yield spread

Layer 5: Point Farming
  Many of these activities earn protocol points (Kamino, Jupiter, etc.)
  Points may convert to valuable airdrops

Theoretical combined yield: 15-40%+ APY (varies with risk level)
```

### 2.4 Restaking

**What it is:** Taking assets that are already staked (securing one network) and "re-staking" them to simultaneously secure additional networks or services. Earn additional yield without additional capital.

**EigenLayer (Ethereum) - The Originator:**
- Launched 2023, pioneered the restaking concept
- Users deposit stETH/ETH -> restake to secure "Actively Validated Services" (AVSs)
- AVSs include: oracles, bridges, data availability layers, sequencers
- Additional yield comes from AVS fees paid to restakers
- Risk: slashing on both the base layer AND the AVS layer

**Solana Restaking Ecosystem:**

| Protocol | Approach | Focus |
|----------|----------|-------|
| Solayer | EigenLayer-equivalent for Solana | Native on-chain dApps, validator coordination |
| Jito Restaking | Extension of Jito's MEV infrastructure | Exogenous AVSs (bridges, oracles, sequencers) |
| Picasso | Cross-chain restaking | Bridge between Solana and other ecosystems |

**Solayer specifics:**
- Raised $12M seed at $80M valuation
- LAYER token rose over 300% in 2025
- Unlike EigenLayer (which started with cross-chain bridges and oracles), Solayer initially focuses on native Solana on-chain dApps
- Allows SOL stakers to provide economic security to DeFi applications built on Solana

**Jito Restaking:**
- Released code for Solana restaking network in 2024-2025
- Extends Jito's validator infrastructure to enable staked assets to secure additional services
- Focuses on exogenous AVSs: cross-chain bridges, oracles, shared sequencers
- 88% of staked SOL is on the Jito-Solana client, giving Jito massive distribution

### 2.5 Liquid Restaking Tokens (LRTs)

**What they are:** Just as liquid staking tokens (mSOL, JitoSOL) represent staked SOL, Liquid Restaking Tokens represent restaked assets. They maintain liquidity while the underlying capital secures multiple services.

**The yield layers:**

```
Layer 0: Hold SOL
Layer 1: Stake SOL -> LST (JitoSOL) -> Earn staking + MEV yield (~7-9%)
Layer 2: Restake JitoSOL -> LRT -> Earn AVS security fees (additional 2-5%?)
Layer 3: Use LRT in DeFi -> Lend, LP, leverage -> Earn DeFi yield
```

### 2.6 The Layered Yield Problem

**The concern:** Each additional yield layer adds:
- Smart contract risk (another protocol that could be exploited)
- Liquidity risk (each derivative is less liquid than the one below)
- Correlation risk (a problem in one layer cascades upward)
- Complexity risk (harder to reason about the total risk profile)

**Real-world example of cascading risk:**
- Q4 2025: A $258 million whale liquidation on Solana triggered cascading losses across DeFi protocols and validator staking pools
- October 10, 2025: Crypto market's largest liquidation event ($19.5B) coincided with network instability
- 72% of Solana TVL concentrated in just five protocols as of Q1 2025

**Practical advice:**
- Understand every layer of risk you are taking
- Never assume correlation between layers is zero
- Keep some capital outside the yield stack for emergencies
- Be skeptical of yields that seem too high -- they usually come with hidden risks

> **Sources:** [Nansen - Solana Liquid Staking 2025](https://www.nansen.ai/post/solana-liquid-staking-everything-you-need-to-know-in-2025), [Sanctum - LST Guide](https://sanctum.so/blog/solana-liquid-staking-guide), [KuCoin - Restaking on Solana](https://www.kucoin.com/learn/crypto/restaking-on-solana-comprehensive-guide), [Blockworks - Jito Restaking](https://blockworks.co/news/lightspeed-newsletter-jito-restaking-solana), [CCN - Solayer Explained](https://www.ccn.com/education/crypto/solayer-layer-restaking-protocol-solana/)

---

## 3. Perpetual DEXes and Derivatives

### 3.1 How On-Chain Perpetual Futures Work

**Perpetual futures** are derivative contracts that let you go long or short an asset with leverage, with no expiry date. Unlike traditional futures that expire on a set date, perps use a **funding rate mechanism** to stay anchored to the spot price.

**Why they matter:** Perp DEXes are the largest category of DeFi derivatives. In 2025, daily perp DEX volume hit $96.97 billion, with monthly turnover reaching ~$1.064 trillion. Total 2025 perp DEX volume: ~$7.9 trillion (up from $1.5 trillion in 2024).

**Core components of a perp DEX:**

```
1. Price Feed:     Oracle-based (Pyth, Switchboard) or order book derived
2. Margin System:  Smart contracts managing collateral deposits
3. Matching:       Order book, vAMM, or LP pool
4. Funding Rate:   Periodic payment between longs and shorts
5. Liquidation:    Automated position closure when margin is insufficient
6. Settlement:     All on-chain via smart contracts
```

### 3.2 Trading Models

#### Virtual AMM (vAMM) Model

**Used by:** Drift Protocol (Solana), Perpetual Protocol (historical)

```
How a vAMM works:
  - Uses the x*y=k curve formula, but with NO real liquidity in the pool
  - The "virtual" liquidity is a mathematical construct for price discovery
  - Traders deposit real collateral, but trade against the virtual curve
  - The protocol (or insurance fund) is the counterparty

Drift's DAMM (Dynamic AMM) Enhancement:
  - Recalibrates virtual liquidity based on demand
  - Reduces slippage dynamically
  - Increases capital efficiency over static vAMMs
```

**Drift Protocol's hybrid approach (three liquidity sources):**
1. **JIT Auction (~5 second window):** Market makers compete to fill orders before they hit the vAMM
2. **Decentralized Limit Order Book (DLOB):** Resting limit orders from users
3. **vAMM backstop:** Always-available liquidity for any unfilled orders

Advantage: Best price discovery from competition between three sources.

#### LP-as-Counterparty Model

**Used by:** Jupiter Perps (JLP), GMX (GLP/GM), Gains Network

```
How it works:
  - LPs deposit assets into a multi-asset pool (the "house")
  - Traders trade against this pool
  - Oracle pricing (no AMM curve) = zero slippage, zero price impact
  - LPs earn fees but bear the risk of trader profits

Key insight: If traders are net unprofitable (which is historically true),
             LPs earn trading fees PLUS trader losses. This is why JLP/GLP
             yields can be very high (30-60% APY during volatile markets).
```

**GMX specifics:**
- GLP: Multi-asset pool (ETH, BTC, USDC, etc.)
- Zero impermanent loss (no AMM curve)
- Oracle pricing from Chainlink
- 70% of trading fees go to GLP holders
- GMX V2 introduced isolated pools (GM tokens) for better risk management

#### Order Book Model

**Used by:** Hyperliquid, dYdX, Aster DEX

```
How it works:
  - Traditional order book with bids and asks
  - Market makers place limit orders providing liquidity
  - Takers match against the book
  - Can be fully on-chain (Hyperliquid) or hybrid (off-chain matching, on-chain settlement)

Hyperliquid specifics:
  - Custom L1 chain with HyperBFT consensus
  - Sub-second finality (~0.2s)
  - Up to 200,000 orders per second throughput
  - Commanded ~70% of perp DEX market share through mid-2025
  - Generated $800M+ in annualized revenue
```

### 3.3 Funding Rates

**Purpose:** Keep perpetual futures prices aligned with spot prices without an expiry date forcing convergence.

**Mechanism:**

```
When perp price > spot price (contango / bullish sentiment):
  -> Longs pay shorts (positive funding rate)
  -> This incentivizes closing longs / opening shorts
  -> Pushes perp price down toward spot

When perp price < spot price (backwardation / bearish sentiment):
  -> Shorts pay longs (negative funding rate)
  -> This incentivizes closing shorts / opening longs
  -> Pushes perp price up toward spot

Payment frequency: Every 1 hour (Jupiter Perps) or every 8 hours (most CEXes)
Typical rate: 0.01% - 0.03% per 8-hour period during normal markets
```

**Why funding rates matter for strategies:**
- Basis traders earn funding by being on the "receiving" side
- During bull markets, positive funding rates are persistent (longs pay shorts)
- 2024 average BTC+ETH funding: ~11% annualized
- 2025 average: ~5% annualized (lower due to less extreme bullish sentiment)

### 3.4 Options Protocols

**Why on-chain options are hard:**
- Options require complex pricing models (Black-Scholes, etc.)
- Liquidity fragmented across strikes and expiries
- Decentralized options account for less than 1% of total crypto options volume
- UX is significantly worse than CEX options trading

**Major protocols:**
- **Derive (formerly Lyra):** On-chain options + perpetuals + structured products on Arbitrum
- **Dopex/Stryke:** Options AMM with novel liquidity mechanisms
- **Aevo:** Off-chain order book with on-chain settlement
- **Hegic:** Simplified options buying (call/put) with pooled liquidity

**Options vault strategies (Structured Products):**
- **Covered call vaults:** Deposit ETH, vault automatically sells call options, earns premium. Works best in sideways/slightly bullish markets. Typical yield: 10-30% APY.
- **Put-selling vaults:** Sell put options on assets you're willing to buy at lower prices. Earns premium while waiting.
- **Straddle/strangle vaults:** Sell both calls and puts, profiting from low volatility periods.

**2025 growth drivers:**
- Better UX on L2s and app-specific chains
- Lower transaction costs
- More sophisticated automated strategies
- Integration with structured product platforms

> **Sources:** [LCX - Perpetual DEXs](https://lcx.com/en/understanding-perpetual-dexs-the-future-of-on-chain-derivatives), [Atomic Wallet - Perp DEXs 2025](https://atomicwallet.io/academy/articles/perpetual-dexs-2025), [Drift Protocol Docs](https://docs.drift.trade/about-v2/drift-amm), [21Shares - Perp DEX Wars](https://www.21shares.com/en-us/research/the-perpetual-dex-wars-hyperliquid-aster-and-lighter-in-focus), [Opium - DeFi Options 2024-2025](https://medium.com/opium-network/defi-options-derivatives-in-2024-2025-trends-and-key-platforms-2579f1e45927)

---

## 4. MEV Deep Dive

### 4.1 What MEV Really Is

**Maximal Extractable Value (MEV)** is the additional profit that can be captured by reordering, including, or excluding transactions within a block. Originally called "Miner Extractable Value" (when miners ordered transactions), it was renamed to "Maximal" as the concept applies to any block producer.

**Why it exists:** Blockchain transactions are not executed instantly. Between when a transaction is submitted and when it is included in a block, there is a window where someone who can influence transaction ordering can profit from that knowledge.

**Scale:** In 2025, MEV-related transaction volume on major chains reached billions of dollars. On Solana specifically, MEV drives approximately 3.5% of the total 11.78% staking reward rate, representing ~30% of the Staking Reward Rate for delegators.

### 4.2 Types of MEV

#### Arbitrage
```
The simplest and most "benign" form of MEV.

Example:
  SOL/USDC on Orca: $150.00
  SOL/USDC on Raydium: $150.50

  Searcher: Buy SOL on Orca -> Sell on Raydium -> Profit $0.50 per SOL

  This actually HELPS the market by equalizing prices across venues.
  Arbitrage accounted for a significant portion of MEV in 2025.
```

#### Sandwich Attacks
```
The most "predatory" form of MEV.

Step 1: Victim submits a large buy order for SOL (visible in mempool/transaction stream)
Step 2: Attacker front-runs with their own buy order (pushing price up)
Step 3: Victim's buy executes at the higher price
Step 4: Attacker back-runs with a sell order (profiting from the price impact)

Result: Victim pays more, attacker profits from the difference.

2025 numbers:
  - 208,149 sandwich attacks identified in one study period
  - $289.76 million in sandwich attack volume (51.56% of total MEV)
  - Entry barrier: Initial capital of only ~6x the target swap volume
  - 5.7x more common than JIT liquidity attacks
```

#### Just-In-Time (JIT) Liquidity
```
A sophisticated form of MEV in concentrated liquidity AMMs.

Step 1: Searcher sees a large swap about to execute
Step 2: Adds concentrated liquidity just before the swap (at the exact price range)
Step 3: Swap executes, generating fees for the just-added liquidity
Step 4: Immediately removes liquidity after the swap

Result: The JIT provider earns the majority of fees for that swap,
        "sniping" yield from passive LPs.

2025 numbers:
  - 36,671 JIT attacks identified over a 20-month study
  - Total profit: 7,498 ETH
  - Considered less harmful than sandwiching (swapper gets better execution)
```

#### Liquidations
```
Competing to liquidate under-collateralized positions.

Step 1: Searcher monitors lending protocols for positions near liquidation threshold
Step 2: Watches for oracle price updates that will trigger liquidation
Step 3: Submits liquidation transaction immediately after price update
Step 4: Earns the liquidation bonus (typically 5-15% of collateral)

This is generally "beneficial" MEV -- it keeps lending protocols solvent.
But the competition can lead to priority gas auctions and network congestion.
```

#### Back-Running
```
Inserting transactions immediately AFTER a target transaction.

Example: After a large DEX trade creates a price discrepancy,
         back-run with an arbitrage trade to capture the rebalancing profit.

Less harmful than front-running because it doesn't worsen the original trade's execution.
```

### 4.3 MEV on Solana vs. Ethereum: Key Differences

| Aspect | Ethereum | Solana |
|--------|----------|--------|
| **Mempool** | Public mempool where pending txs are visible | No traditional mempool; txs go directly to leader |
| **Block production** | Unknown next block proposer (in PoS) | Known leader schedule (predictable who builds next block) |
| **MEV approach** | Flashbots/MEV-Boost: block builder market | Jito: bundle auction system |
| **Transaction ordering** | Builders order by priority fee | Leader processes in arrival order + Jito bundles |
| **Timing** | ~12 second block times | ~400ms block times |
| **Spam approach** | Gas price wars (expensive to spam) | Low fees = massive spam (searchers bombard validators) |
| **Key challenge** | Block builder centralization | Validator-searcher collusion, spam |

**Solana's unique MEV characteristics:**
- Known leader schedule means searchers know which validator to target
- Low transaction fees make spam attacks cheap (historically, searchers sent thousands of transactions hoping to land at the right time)
- No traditional mempool means front-running requires different techniques (co-location with validators, observing the transaction pipeline)
- Continuous block production (not discrete like Ethereum) creates different timing dynamics

### 4.4 Jito's MEV Architecture on Solana

**The problem Jito solves:** Before Jito, MEV on Solana was chaotic. Searchers would spam the network with duplicate transactions, degrading performance for everyone.

**Jito's solution: An orderly MEV marketplace**

```
Architecture Components:

1. RELAYER
   - Receives transactions from all sources (wallets, searchers, aggregators)
   - Holds transactions for ~200ms before forwarding to validator
   - This 200ms window allows the Block Engine to process bundles
   - Prevents searchers from racing directly to the validator

2. BLOCK ENGINE
   - Sits between Relayer and Validator
   - Receives bundles from searchers
   - Simulates each bundle to verify profitability and validity
   - Ranks bundles by tip amount
   - Selects winning bundles and forwards to validator

3. JITO-SOLANA VALIDATOR
   - Modified Solana validator with additional stages:
     * RelayerStage: receives forwarded transactions
     * BlockEngineStage: receives winning bundles
     * BundleStage: integrates bundles into block production
   - Includes bundles in blocks alongside normal transactions

4. SEARCHERS
   - Submit bundles (groups of up to 5 transactions)
   - Include a "tip" (minimum 1,000 lamports SOL) in the last transaction
   - Tips incentivize validators to include the bundle
   - Bundles execute atomically: all-or-nothing, sequential order
```

**Bundle constraints:**
- Maximum 5 transactions per bundle
- All transactions execute sequentially and atomically
- If any transaction fails, the entire bundle is dropped
- Tips are paid via transfer instruction in the last transaction
- Can chain transactions that individually exceed compute limits

**Economic impact (2025):**
- Jito bundles account for over 22% of total validator rewards in Q1 2025
- Over 65% of Solana validators run Jito's client
- MEV at all-time highs: ~3.5% of 11.78% staking reward rate
- This compares to 0.01% just one year prior
- Tips are redistributed to validators and their stakers (via JitoSOL)

### 4.5 MEV as Yield Source vs. MEV as Attack

**MEV as yield (beneficial):**
- Arbitrage: Equalizes prices across venues, improves market efficiency
- Liquidations: Keeps lending protocols solvent
- JitoSOL stakers earn MEV tips as additional yield (1-2% extra APY)
- Validators earn tips, increasing staking rewards for delegators
- JIT liquidity can improve execution for large swaps

**MEV as attack (harmful):**
- Sandwich attacks: Directly tax users by worsening execution prices
- Front-running: Extracts value from informed traders
- Time-bandit attacks: Re-org blocks to capture past MEV (theoretical on Solana)
- Searcher spam: Degrades network performance for all users
- Centralizing force: Sophisticated MEV extraction favors well-resourced actors

### 4.6 MEV Mitigation Strategies for Protocol Designers

**1. Private/Encrypted Transaction Submission:**
```
- Private RPC endpoints that bypass public transaction streams
- Transactions submitted directly to block builders/validators
- By mid-2025, >50% of high-value Ethereum txs routed through private channels
- On Solana: Jito bundles can be used for private transaction inclusion
```

**2. Batch Auctions:**
```
- CoW Protocol: Aggregates multiple orders, settles together
- Eliminates priority ordering (no front-running possible)
- Uniform clearing price for all participants
- Orders matched off-chain, settled on-chain
```

**3. Fair Ordering / Sequencing:**
```
- Chainlink's Fair Sequencing Services (FSS)
- Decentralized oracle network establishes FIFO ordering
- Removes ability for any single party to reorder for profit
```

**4. Encrypted Mempools:**
```
- Shutter Network: Transactions encrypted until inclusion
- Commit-reveal schemes: Submit hash first, reveal content later
- Prevents MEV extraction by hiding transaction intent
```

**5. MEV-Aware Protocol Design (Solana-specific):**
```
- Use slippage protection (set tight slippage tolerance)
- Implement time-weighted average prices (TWAP) for large orders
- Use Jito bundles for multi-step transactions
- Consider integrating with MEV-protected RPCs
- Design oracle update mechanisms that are resistant to front-running
```

> **Sources:** [Helius - Solana MEV Introduction](https://www.helius.dev/blog/solana-mev-an-introduction), [QuickNode - MEV on Solana](https://www.quicknode.com/guides/solana-development/defi/mev-on-solana), [QuickNode - Jito Bundles Guide](https://www.quicknode.com/guides/solana-development/transactions/jito-bundles), [Jito Docs](https://docs.jito.wtf/), [Eclipse Labs - How Jito Works](https://www.eclipselabs.io/blogs/how-jito-works---a-deep-dive), [Ancilar - MEV Protection 2025](https://medium.com/@ancilartech/implementing-effective-mev-protection-in-2025-c8a65570be3a)

---

## 5. Cross-Chain DeFi and Bridges

### 5.1 How Bridges Work

Bridges enable assets and data to move between different blockchains. There are three primary models:

#### Lock-and-Mint
```
Source Chain (Ethereum)              Destination Chain (Solana)
  |                                    |
  | 1. User locks 1 ETH in            |
  |    bridge contract                 |
  |         |                          |
  |    [Bridge Validators/Relayers verify the lock]
  |         |                          |
  |                                    | 2. Bridge mints 1 "Wrapped ETH"
  |                                    |    on Solana
  |                                    |
  | To go back:                        |
  |                                    | 3. User burns Wrapped ETH
  |    [Bridge verifies the burn]      |
  | 4. Bridge unlocks original ETH     |

  Used by: Wormhole, most traditional bridges
  Risk: Locked funds are a honeypot for attackers
```

#### Burn-and-Mint
```
Source Chain                          Destination Chain
  |                                    |
  | 1. User burns native tokens        |
  |         |                          |
  |    [Bridge verifies the burn]      |
  |         |                          |
  |                                    | 2. Bridge mints same native
  |                                    |    tokens on destination

  Used by: Circle's CCTP (for USDC native transfers)
  Advantage: No wrapped tokens; native assets on both chains
  Requirement: Token issuer must support multi-chain minting
```

#### Lock-and-Unlock (Liquidity Networks)
```
Source Chain                          Destination Chain
  |                                    |
  | 1. User locks tokens               | (Liquidity pool exists on
  |         |                          |  destination with pre-funded tokens)
  |    [Bridge verifies]               |
  |         |                          |
  |                                    | 2. Bridge unlocks native tokens
  |                                    |    from liquidity pool

  Used by: Stargate, Across, Hop Protocol
  Advantage: Users receive native tokens, not wrapped versions
  Disadvantage: Requires deep liquidity pools on both sides
  LPs earn fees for providing bridge liquidity
```

### 5.2 Bridge Security Models

| Model | Trust Assumption | Examples | Security Level |
|-------|-----------------|----------|----------------|
| **Externally Verified** | Trust a set of validators/guardians | Wormhole (19 guardians) | Medium |
| **Natively Verified** | Trust the chains' consensus | IBC (Cosmos) | High |
| **Optimistically Verified** | Trust 1 honest watcher (fraud proofs) | Across, Connext | Medium-High |
| **ZK-Verified** | Trust math (zero-knowledge proofs) | zkBridge, Succinct | Highest |
| **MPC-based** | Trust threshold of key holders | Symbiosis | Medium |

**The spectrum:** More trust assumptions = faster/cheaper but less secure. Trustless verification (ZK proofs) is the holy grail but computationally expensive.

### 5.3 Why Bridges Are the Most Attacked DeFi Primitive

**The numbers:** Over $2.8 billion stolen from bridges since 2022. Cross-chain breaches accounted for the majority of DeFi hacks, with over $2.3 billion lost in H1 2025 alone, surpassing all of 2024.

**Why bridges are targeted:**
1. **Massive honeypots:** Bridges hold enormous locked liquidity pools (billions of dollars)
2. **Complex attack surface:** Must be secure on multiple chains simultaneously
3. **Trust assumptions:** Many bridges rely on multi-sig or validator committees that can be compromised
4. **Cross-chain verification is hard:** Verifying state from one chain on another is fundamentally difficult
5. **More interactions:** Bridges require more contract interactions and approvals than other DeFi protocols

**Major bridge hacks:**

```
Ronin Network - March 2022 - $620M
  Attack: Social engineering (fake LinkedIn job offer)
  Vector: Compromised 5 of 9 validator private keys
  Attacker: North Korea's Lazarus Group
  Lesson: Multi-sig with low threshold is a single point of failure

Wormhole - February 2022 - $320M
  Attack: Smart contract exploit
  Vector: Attacker spoofed a guardian signature, minted 120,000 wETH
  Resolution: Jump Crypto backstopped the loss, later counter-exploited the hacker
  Lesson: Signature verification code must be bulletproof

Nomad - August 2022 - $190M
  Attack: Smart contract configuration error
  Vector: A code update allowed anyone to "spoof" valid transactions
  Unique: Over 40 different attackers copy-pasted the exploit
  Lesson: Upgrade processes need extreme verification; initialization bugs are deadly

BNB Bridge - October 2022 - $586M
  Attack: Proof verification bypass
  Vector: Attacker forged proof to mint 2M BNB
  Lesson: Cross-chain proof verification is a critical attack surface
```

### 5.4 Intent-Based Bridges and Solvers

**The evolution:** Traditional bridges are slow (wait for finality on both chains) and risky (large locked pools). Intent-based bridges flip the model:

```
Traditional Bridge:
  User locks funds -> Wait for verification -> Funds appear on destination
  Time: Minutes to hours
  Risk: Smart contract holding locked funds

Intent-Based Bridge:
  User signs intent: "I want 1 ETH on Arbitrum, willing to pay 1.001 ETH on Ethereum"
  Solver: "I already have ETH on Arbitrum, I'll fill this intent"
  User gets ETH on Arbitrum immediately from solver's inventory
  Solver collects user's ETH on Ethereum
  Time: Seconds
  Risk: Distributed across solvers, no single large pool
```

**Key intent-based bridge protocols:**
- **Across:** Optimistic verification with competitive solver network
- **deBridge:** Fast cross-chain with solver competition
- **Mayan Swift:** Solana-focused intent bridge
- **LI.FI:** Aggregator that routes across multiple bridges and solvers

**Advantages:**
- Faster execution (solvers pre-fund)
- Reduced smart contract risk (no massive locked pools)
- Better pricing through solver competition
- MEV protection (solvers handle the complexity)

> **Sources:** [Chainlink - Cross Chain Bridge](https://chain.link/education-hub/cross-chain-bridge), [HackenProof - Bridge Hacks](https://hackenproof.com/blog/for-hackers/web3-bridge-hacks), [CertiK - Wormhole Analysis](https://www.certik.com/resources/blog/wormhole-bridge-exploit-incident-analysis), [CoinCryptoRank - DeFi Bridge Security](https://coincryptorank.com/blog/defi-bridge-security-cross-chain-protection), [LI.FI - Solvers](https://li.fi/knowledge-hub/with-intents-its-solvers-all-the-way-down/)

---

## 6. DeFi Composability in Practice

### 6.1 How Protocols Compose on Solana (CPI Chains)

**Cross-Program Invocation (CPI)** is Solana's mechanism for programs to call other programs within a single transaction. This is the foundation of DeFi composability.

```
Example: Flash loan arbitrage in a single Solana transaction

Instruction 1: Borrow 1000 USDC from Port Finance (flash loan)
  -> CPI call to Port Finance's lending program

Instruction 2: Swap 1000 USDC -> SOL on Orca
  -> CPI call to Orca's swap program

Instruction 3: Swap SOL -> 1005 USDC on Raydium (price is better here)
  -> CPI call to Raydium's swap program

Instruction 4: Repay 1000 USDC + fee to Port Finance
  -> CPI call to Port Finance's repay function

Net profit: ~5 USDC minus fees (all in one atomic transaction)
If any step fails, the ENTIRE transaction reverts (nothing happens)
```

**CPI Depth Limit:** Solana allows a maximum of 4 levels of CPI calls. This is an important constraint:

```
Level 0: Your program
  Level 1: -> calls Lending Protocol
    Level 2: -> which calls Token Program
      Level 3: -> which calls another program
        Level 4: Maximum depth reached
```

**Working around the CPI limit:**
- The FLUF (Flash Loan Unlimited Facility) Protocol addresses this by restructuring the call chain
- Design patterns that flatten CPI hierarchies instead of nesting them
- Use multiple instructions in a single transaction instead of nested CPI calls

**Transaction composition:**
- A single Solana transaction can contain up to ~1232 bytes of instructions
- Complex multi-protocol strategies often use multiple instructions within one transaction
- Versioned transactions and Address Lookup Tables increase the number of accounts accessible

### 6.2 Flash Loan-Enabled Composability

**What flash loans enable on Solana:**

```
1. Arbitrage: Borrow -> swap on DEX A -> swap back on DEX B -> repay + profit
2. Self-liquidation: Borrow to repay debt and avoid liquidation penalty
3. Collateral swap: Flash borrow -> repay loan -> withdraw collateral ->
                     deposit new collateral -> take new loan -> repay flash loan
4. Position migration: Move entire leveraged position from one protocol to another
5. Debt restructuring: Refinance loans across protocols atomically
```

**Key flash loan providers on Solana:**
- Port Finance: Established flash loan infrastructure
- Solend: Flash loan capabilities integrated with lending
- Kamino: Flash loans through lending vaults

**Why flash loans are particularly powerful on Solana:**
- Low fees: Flash loan transactions cost fractions of a cent (vs. $50+ on Ethereum L1)
- Fast execution: ~400ms block times mean flash loan strategies execute quickly
- High throughput: Multiple flash loan arbitrage opportunities per second

### 6.3 Risks of Composability

#### Dependency Chains
```
Your strategy depends on:
  Protocol A (lending) which depends on:
    Oracle X (price feed) which depends on:
      Data source Y
  AND Protocol B (DEX) which depends on:
    Liquidity from LPs who also use:
      Protocol C (yield farming) which depends on:
        Token Z's price

If ANY link in this chain fails, your strategy can break.
```

#### Cascading Failures (Real Examples)

**UST/Luna Collapse (May 2022):**
```
UST depeg -> Anchor Protocol depositors panic withdraw ->
  Luna price crashes -> More UST sells -> More Luna minting ->
    Death spiral across every protocol that used UST as collateral
    $40+ billion destroyed in days
```

**Solana DeFi Cascading Liquidation (Q4 2025):**
```
Large whale position ($258M) liquidated ->
  Cascading liquidations across DeFi protocols ->
    Validator staking pool losses ->
      Part of $19.5B total crypto liquidation event
```

**USX Stablecoin Depeg (December 2025):**
```
USX collapsed to $0.10 despite >100% collateralization ->
  Liquidity crisis (not solvency crisis) ->
    Protocols using USX as collateral affected ->
      Recovery required external liquidity injection
```

#### Oracle Dependency Risk
```
If a price oracle is manipulated or delayed:
  - Lending protocols may allow under-collateralized borrows
  - Perp positions may be incorrectly liquidated
  - Arbitrage bots may extract value from stale prices

72% of Solana TVL is concentrated in just 5 protocols (Q1 2025),
amplifying the impact of any single oracle failure.
```

### 6.4 Real Examples of Multi-Protocol Strategies

**Strategy 1: JitoSOL Leverage Yield Stack**
```
1. Stake SOL -> JitoSOL on Jito (earn 7-9% staking + MEV)
2. Deposit JitoSOL into Kamino Lend (earn supply rate)
3. Borrow SOL against JitoSOL
4. Stake borrowed SOL -> more JitoSOL
5. Repeat steps 2-4 (looping)
6. Effective yield: 15-25% APY on SOL at 2x leverage
Protocols involved: Jito, Kamino, Jupiter (for swaps)
```

**Strategy 2: Delta-Neutral JLP**
```
1. Deposit into JLP pool on Jupiter (earn ~30-60% APY from trading fees)
2. JLP has long exposure to SOL/BTC/ETH (from pool composition)
3. Short SOL/BTC/ETH on Drift Protocol to hedge
4. Net result: Fee yield with reduced directional exposure
Protocols involved: Jupiter Perps (JLP), Drift Protocol
```

**Strategy 3: Pendle Fixed Rate + Leverage**
```
1. Deposit stETH into Pendle
2. Buy PT-stETH at discount (lock in ~8% fixed yield)
3. Use PT-stETH as collateral on Aave/Morpho
4. Borrow stablecoins against it
5. Deploy stablecoins into another yield source
Protocols involved: Lido, Pendle, Aave/Morpho
```

**Strategy 4: Flash Loan Arbitrage Bot**
```
1. Monitor price discrepancies between Orca and Raydium
2. When spread > threshold:
   a. Flash borrow from lending protocol
   b. Buy cheap on one DEX
   c. Sell expensive on other DEX
   d. Repay flash loan + fee
   e. Keep profit
3. All in one atomic Solana transaction
Protocols involved: Port Finance/Kamino (flash loan), Orca, Raydium
```

> **Sources:** [Chainscore Labs - Solana Composability](https://www.chainscorelabs.com/en/blog/solana-and-the-rise-of-high-performance-chains/solana-virtual-machine-svm-deep-dive/the-future-of-defi-lies-in-solanas-atomic-transaction-composability), [GitHub - Flash Loan Unlimited Solana](https://github.com/jordan-public/flash-loan-unlimited-solana), [Kenson Investments - Solana DeFi 2025 Risks](https://kensoninvestments.com/solana-defi-in-2025-risks-rewards-and-regulatory-considerations/)

---

## 7. Emerging DeFi Trends (2024-2026)

### 7.1 Intent-Based Trading

**The paradigm shift:** Instead of users specifying HOW to execute a trade (which DEX, which route, what gas price), they specify WHAT outcome they want. Specialized "solvers" compete to deliver the best execution.

```
Traditional:
  User: "Swap 1000 USDC for SOL on Orca, max slippage 0.5%, gas priority: high"
  Result: User must know which DEX has best price, set parameters correctly

Intent-Based:
  User signs: "I want at least 6.5 SOL for my 1000 USDC within 30 seconds"
  Solvers compete to fill the intent using ANY source of liquidity
  Result: User gets best execution without needing to know the plumbing
```

**How solvers work:**
- Run algorithms to find optimal routes across all liquidity sources
- Can access off-chain liquidity: CEX inventory, RFQ systems, proprietary liquidity
- Compete in auctions; best price wins
- Handle gas/fees (factored into execution price)

**Benefits for users:**
- MEV protection (less information exposed to bots)
- Better execution (solvers access deeper liquidity)
- Gas abstraction (no need to manage gas tokens)
- Simpler UX (express what you want, not how to get it)

**Market growth (2025):**
- NEAR Intents: From $3M to $6B cumulative volume (200,000% increase in 2025)
- CoW Swap: Monthly volume reached $10B/month (5x increase from late 2024)
- Jupiter Exchange: Dominant intent-based trading on Solana
- UniswapX: Uniswap's intent-based layer with Dutch auction pricing

**Key protocols:**
- CoW Protocol: Batch auctions, MEV protection, pioneered intent-based DeFi
- 1inch Fusion: Intent-based swaps with resolver network
- Jupiter Exchange: Solana's dominant aggregator with intent-like routing
- Across Protocol: Cross-chain intents with competitive solver network
- 0x/Matcha: Intent infrastructure for developers

> **Sources:** [CoW DAO - Understanding Crypto Intents](https://cow.fi/learn/understanding-crypto-intents-the-future-of-your-de-fi-trades), [0x - Intents Fundamentals](https://0x.org/post/intents-in-defi), [Eco - Intent-Based DEX Guide](https://eco.com/support/en/articles/11852634-best-intent-based-dex-platforms-complete-2025-comparison-guide)

---

### 7.2 Modular DeFi

**What it means:** Instead of monolithic protocols that do everything, DeFi is decomposing into specialized modules:

```
Monolithic (old):
  One protocol handles: Execution + Settlement + Data Availability + Security

Modular (new):
  Execution Layer:      App-specific rollup (dYdX Chain, Hyperliquid L1)
  Settlement Layer:     Ethereum mainnet or Solana
  Data Availability:    Celestia, EigenDA, or the base chain
  Security Layer:       Inherited from settlement + restaking
```

**DeFi-specific modularity:**
- **Morpho:** Modular lending infrastructure - separates risk management from lending mechanics
- **Euler V2:** Modular vault system where anyone can create custom lending markets
- **Uniswap V4 Hooks:** Modular AMM where custom logic can be inserted at any point in the swap lifecycle

**Appchains for DeFi:**
- dYdX V4: Migrated from Ethereum to its own Cosmos chain for full control over block production and MEV
- Hyperliquid: Built a custom L1 chain for maximum performance
- Aster DEX: App-specific chain for perpetual trading

**Rollups-as-a-Service (RaaS) for DeFi:**
- RaaS platforms enable DeFi protocols to launch their own chains with one-click deployment
- Cosmos SDK: 60+ production appchains by early 2026
- Arbitrum Orbit, OP Stack, Polygon CDK: Frameworks for custom DeFi rollups

> **Sources:** [NadCab - DeFi and Modular Blockchains](https://www.nadcab.com/blog/defi-and-the-rise-of-modular-blockchains), [Alchemy - Custom Rollups RaaS](https://www.alchemy.com/blog/deploying-custom-rollup-raas-2025)

---

### 7.3 Real World Assets (RWA) Integration

**The "RWA Super-Cycle":** Tokenized real-world assets represent the largest growth narrative in DeFi for 2025-2026.

**Market numbers (2025):**
- Total RWA value on-chain: $33 billion (5x growth from $7.9B in two years)
- Tokenized US Treasuries: Over $8B in AUM (25% of total RWA value)
- RWA tokens: Most profitable asset class in 2025 with 185% average growth

**Major tokenized asset categories:**

```
1. US Treasury Bills/Bonds (Largest category)
   - BlackRock BUIDL: $2.8B+ AUM, largest tokenized Treasury fund
   - Ondo OUSG/USDY: ~17% market share, second largest
   - Franklin Templeton BENJI: Early institutional mover

2. Private Credit (~$17B tokenized)
   - Loans to real businesses, tokenized for on-chain investors
   - Higher yields than Treasuries (8-15%) but higher risk
   - Centrifuge, Maple Finance, Goldfinch

3. Real Estate
   - Fractional ownership of properties via tokens
   - Lower entry barriers ($100 vs. $100K+ for traditional real estate)
   - Still early stage, regulatory challenges

4. Carbon Credits, Commodities, Art
   - Emerging categories with growing on-chain presence
```

**Why RWA matters for DeFi developers:**
- BUIDL as DeFi collateral: BlackRock's BUIDL is now used as reserve collateral for Ethena's USDtb and Ondo's OUSG
- Yield-bearing stablecoins: Treasury-backed stablecoins pass real-world yields to holders
- DAO treasuries: DAOs can hold tokenized Treasuries for yield on reserves
- Lending collateral: Tokenized RWAs accepted as collateral on DeFi lending protocols

**On Solana specifically:**
- Ondo has deployed USDY on Solana
- Maple Finance operates on Solana
- RWA integration benefits from Solana's low fees and fast finality

> **Sources:** [Brickken - RWA Tokenization 2025](https://www.brickken.com/post/rwa-tokenization-trends-2025), [Yellow.com - Tokenized Treasuries](https://yellow.com/en-US/research/tokenized-us-treasuries-hit-dollar73b-in-2025-complete-guide-to-digital-treasury-bonds), [CoinGecko - Top Crypto Narratives](https://www.coingecko.com/learn/crypto-narratives)

---

### 7.4 AI Agents in DeFi (DeFAI)

**The convergence:** Autonomous AI systems that execute DeFi strategies, manage risk, and operate 24/7 without human intervention. This is the most explosive narrative heading into 2026.

**What AI agents actually do in DeFi:**

```
Level 1: Natural Language Execution
  User: "Rebalance my portfolio into high-yield stablecoins across three chains"
  Agent: Analyzes positions, finds best rates, executes multi-step transactions
  Protocols: Hey Anon, Griffain

Level 2: Automated Strategy Execution
  Agent monitors markets and executes predefined strategies:
  - Rebalance LP positions when out of range
  - Harvest and compound yield farming rewards
  - Adjust leverage based on market conditions
  - Move funds between protocols when rates change
  Protocols: Allora (AI-powered looping), Yearn (AI keepers)

Level 3: Autonomous Portfolio Management
  Agent makes independent investment decisions:
  - Allocates capital across DeFi protocols
  - Manages risk based on predictive models
  - Executes arbitrage when opportunities arise
  - Governs DAO participation
  Protocols: AutoFi layers (Supra, Fetch.ai)
```

**Market adoption (2025-2026):**
- x402 protocol: Decentralized payment standard for AI agents, adopted by Google Cloud, AWS, and Anthropic
- Coinbase x402: Processed $50M+ in cumulative agentic payments
- Yield-generating and trading agents are the dominant AI-DeFi categories
- Industry prediction: Agents could manage trillions in TVL by mid-2026+

**DeFAI for Solana developers:**
- Building AI agents that interact with Solana DeFi protocols via CPI
- AI-powered keepers for vault rebalancing
- Natural language interfaces for complex DeFi operations
- Predictive models for funding rate arbitrage and yield optimization

**Risks:**
- "Hallucination" risk: AI making incorrect financial decisions
- Smart contract interaction bugs (AI-generated transactions could be malformed)
- Adversarial attacks specifically targeting AI agents
- Regulatory uncertainty around autonomous financial agents

> **Sources:** [Ledger - DeFAI Explained](https://www.ledger.com/academy/topics/defi/defai-explained-how-ai-agents-are-transforming-decentralized-finance), [Metaverse Post - Cambrian Report](https://mpost.io/cambrian-report-ai-driven-defi-agents-reach-50m-in-payments-volume-on-x402-rails-while-yield-and-trading-bots-dominate-2026-market-2/), [Medium - Agentic AI in DeFi](https://medium.com/thecapital/agentic-ai-in-defi-the-dawn-of-autonomous-on-chain-finance-584652364d08)

---

### 7.5 Privacy in DeFi (Confidential Transfers)

**The problem:** All DeFi transactions are public. Your wallet balance, trade history, and strategy are visible to anyone. This creates:
- MEV extraction (bots see your pending trades)
- Front-running (competitors copy your strategies)
- Personal security risks (wealthy addresses are targets)
- Institutional hesitance (firms cannot trade without revealing positions)

**Solana's Confidential Balances (launched 2025):**

Solana introduced "Confidential Balances" as token extensions using zero-knowledge proofs:

```
What it enables:
  - Transfer tokens without revealing the transfer amount
  - Encrypted token balances (only owner can see their balance)
  - Confidential minting and burning
  - Optional auditor key for compliance

How it works (technical):
  - Uses homomorphic encryption (compute on encrypted data)
  - Zero-knowledge proofs verify transaction validity without revealing amounts
  - Built into the Token-2022 standard as extensions
  - No separate privacy chain needed; works on Solana mainnet

Extension Suite:
  1. Confidential Transfers: Hide transfer amounts
  2. Confidential Transfer Fees: Hide fee amounts
  3. Confidential Mint and Burn: Hide issuance/redemption amounts
  4. Auditor Key: Optional compliance mechanism
```

**Current status (early 2026):**
- Implementation-ready for Rust-based backends
- JavaScript ZK-proof libraries expected for browser/mobile wallets
- Wallets-as-a-Service providers integrating confidential features
- Regulatory compliance via optional auditor keys

**For DeFi protocol designers:**
- Can integrate confidential transfers for privacy-preserving trading
- Compliance-friendly: Auditor key allows designated parties to decrypt
- Reduces MEV attack surface (hidden transaction amounts)
- Enables institutional DeFi participation (trade without revealing positions)

> **Sources:** [Solana Docs - Confidential Transfer](https://solana.com/docs/tokens/extensions/confidential-transfer), [Helius - Confidential Balances](https://www.helius.dev/blog/confidential-balances), [The Block - Confidential Balances Launch](https://www.theblock.co/post/350076/solana-developers-launch-new-confidential-balances-token-extensions-to-improve-onchain-privacy)

---

### 7.6 Account Abstraction and DeFi UX

**The UX problem:** DeFi is powerful but hostile to new users:
- Seed phrases (lose them = lose everything)
- Gas management (need native tokens to do anything)
- Complex transaction signing (approve + swap is two transactions)
- No recovery mechanism (send to wrong address = gone forever)

**Account Abstraction (AA) solves this:**

```
Traditional EOA (Externally Owned Account):
  - Controlled by a single private key
  - Must pay gas in native token (ETH/SOL)
  - No programmable logic
  - No recovery if key is lost

Smart Contract Account (via AA):
  - Controlled by programmable logic (code)
  - Can pay gas in any token (or have someone else pay)
  - Batch multiple operations in one transaction
  - Social recovery (trusted contacts can help recover)
  - Spending limits and session keys
  - Passkey/biometric authentication
```

**Key standards:**
- **ERC-4337 (Ethereum, March 2023):** Smart contract accounts without core protocol changes
- **EIP-7702 (Ethereum Pectra upgrade, May 2025):** Lets existing EOAs temporarily act as smart contracts
- Over 200 million smart accounts deployed across Ethereum and L2s
- 40 million deployed in 2024 alone (10x increase from 2023)

**On Solana:**
- Solana's account model is already more flexible than Ethereum's EOA model
- Programs (smart contracts) can own and manage accounts
- Session keys allow dApps to execute transactions without repeated signing
- Squads Protocol provides multi-sig smart wallets
- Backpack wallet integrating advanced account features

**Impact on DeFi:**

```
Before AA:
  Step 1: Buy SOL on exchange for gas
  Step 2: Set up wallet, write down seed phrase
  Step 3: Approve USDC spending on Kamino
  Step 4: Deposit USDC into Kamino vault
  Step 5: Approve JLP spending on Jupiter
  Step 6: Swap into JLP position
  (6 separate actions, each requiring understanding of gas, approvals, etc.)

After AA:
  Step 1: Login with email/passkey
  Step 2: Click "Deploy Strategy" (one button)
  (All approvals, swaps, and deposits batched into one gasless transaction)
```

**AA Infrastructure providers:**
- Alchemy, Biconomy, Safe (Gnosis), Argent, Privy, thirdweb, Stackup
- Embedded wallets: Allow apps to create wallets for users behind the scenes
- Passkey wallets: Use device biometrics instead of seed phrases

**Prediction:** Smart wallets will become the default standard, replacing traditional EOAs by the end of the decade.

> **Sources:** [DEV Community - Web3 UX 2026](https://dev.to/wildanzr/web3-ux-finally-feels-normal-in-2026-smart-wallets-account-abstraction-and-the-end-of-seed-2okf), [BlockEden - Account Abstraction Mainstream](https://blockeden.xyz/blog/2026/01/20/account-abstraction-smart-wallets-erc-4337-eip-7702-mainstream/), [FinancialContent - Crypto UX Revolution](https://markets.financialcontent.com/stocks/article/marketminute-2025-11-8-cryptos-ux-revolution-smart-wallets-and-account-abstraction-supercharge-defi-user-experience)

---

## 8. References

### Yield Strategies
- [Contango - What Is Looping (Recursive Borrowing)](https://medium.com/contango-xyz/what-is-looping-78421c8a1367)
- [Airdrop Alert - Looping Strategies in DeFi](https://airdropalert.com/blogs/looping-strategies-in-defi-how-to-multiply-yield-and-farm-airdrops/)
- [Allora Network - AI-Powered Looping Strategy](https://www.allora.network/blog/drift-x-allora-ai-powered-looping-strategy)
- [Panoptic - Delta-Neutral LP Hedging](https://panoptic.xyz/blog/delta-neutral-lp-hedge-uniswap-position)
- [Cetra Finance - Delta-Neutral Strategies Overview](https://medium.com/@cetrafinance/delta-neutral-defi-strategies-overview-d0617561a1f4)
- [Gate.com - Perpetual Contract Funding Rate Arbitrage 2025](https://www.gate.com/learn/articles/perpetual-contract-funding-rate-arbitrage/2166)
- [FXEmpire - Spot-Perp Arbitrage Strategy](https://www.fxempire.com/education/article/market-neutral-crypto-income-made-simple-spot-perp-arbitrage-strategy-explained-1535487)
- [Multicoin Capital - Ethena Synthetic Dollars](https://multicoin.capital/2025/11/13/ethena-synthetic-dollars-challenge-stablecoins-duopoly/)
- [CoinMetrics - Ethena and USDe Mechanics](https://coinmetrics.substack.com/p/state-of-the-network-issue-335)

### Jupiter/JLP
- [CoinMarketCap - What Is JLP](https://coinmarketcap.com/cmc-ai/jupiter-perps-lp/what-is/)
- [Jupiter JLP Yield Analysis PDF](https://assets.ctfassets.net/i0qyt2j9snzb/4ueTPh7PwgopwILuLbhoYL/2fc57577db5f223edfaacf20bd557de1/JLP_-_Jupiter-s_Juicy_Yield.pdf)
- [Coincept - Understanding JLP](https://coincept.substack.com/p/understanding-jupiter-liquidity-provider)

### Vaults and Yield Optimization
- [Keyrock - Automated Onchain Yield Strategies Guide](https://keyrock.com/knowledge-hub/automated-onchain-yield-strategies-a-guide/)
- [Zircuit - DeFi Vaults Explained](https://www.zircuit.com/blog/defi-vaults-how-they-work-and-yield-strategies)
- [Alchemy - DeFi Yield Aggregators List](https://www.alchemy.com/dapps/best/defi-yield-aggregators)

### Liquid Staking and Restaking
- [Nansen - Solana Liquid Staking 2025](https://www.nansen.ai/post/solana-liquid-staking-everything-you-need-to-know-in-2025)
- [Sanctum - Solana Liquid Staking Guide](https://sanctum.so/blog/solana-liquid-staking-guide)
- [Phantom - Solana Liquid Staking Ultimate Guide](https://phantom.com/learn/crypto-101/solana-liquid-staking)
- [KuCoin - Restaking on Solana Comprehensive Guide](https://www.kucoin.com/learn/crypto/restaking-on-solana-comprehensive-guide)
- [Blockworks - Jito Restaking](https://blockworks.co/news/lightspeed-newsletter-jito-restaking-solana)
- [CCN - Solayer Explained](https://www.ccn.com/education/crypto/solayer-layer-restaking-protocol-solana/)
- [Phemex - What Is JitoSOL](https://phemex.com/academy/what-is-jito-network-jitosol)

### Perpetual DEXes
- [LCX - Understanding Perpetual DEXs](https://lcx.com/en/understanding-perpetual-dexs-the-future-of-on-chain-derivatives)
- [Atomic Wallet - Perpetual DEXs 2025](https://atomicwallet.io/academy/articles/perpetual-dexs-2025)
- [21Shares - Perpetual DEX Wars: Hyperliquid, Aster, Lighter](https://www.21shares.com/en-us/research/the-perpetual-dex-wars-hyperliquid-aster-and-lighter-in-focus)
- [Drift Protocol Documentation](https://docs.drift.trade/about-v2/drift-amm)
- [Shoal Research - Drift Protocol Deep Dive](https://www.shoal.gg/p/drift-protocol-solanas-largest-perpetual)
- [ITSA - GMX On-Chain Perpetuals and GLP](https://itsa-global.medium.com/itsa-defi-insight-gmx-on-chain-perpetuals-and-glp-935bb3168f0a)

### Pendle/Yield Tokenization
- [Nansen - Pendle Finance Explained](https://www.nansen.ai/post/what-is-pendle-finance-yield-tokenization-explained-how-to-earn)
- [Pendle Documentation](https://docs.pendle.finance/Introduction)
- [Greythorn - Pendle 2025: Building DeFi's Fixed Income Layer](https://0xgreythorn.medium.com/pendle-2025-building-defis-fixed-income-layer-175a5eeb10fd)
- [MixBytes - Yield Tokenization: Pendle](https://mixbytes.io/blog/yield-tokenization-protocols-how-they-re-made-pendle)

### MEV
- [Helius - Solana MEV Introduction](https://www.helius.dev/blog/solana-mev-an-introduction)
- [QuickNode - MEV on Solana Guide](https://www.quicknode.com/guides/solana-development/defi/mev-on-solana)
- [QuickNode - Solana MEV Economics: Jito, Bundles, Liquid Staking](https://blog.quicknode.com/solana-mev-economics-jito-bundles-liquid-staking-guide/)
- [QuickNode - Jito Bundles Guide](https://www.quicknode.com/guides/solana-development/transactions/jito-bundles)
- [Jito Labs Documentation](https://docs.jito.wtf/)
- [Eclipse Labs - How Jito Works Deep Dive](https://www.eclipselabs.io/blogs/how-jito-works---a-deep-dive)
- [Thogiti - How Jito-Solana Works](https://thogiti.github.io/2025/01/01/How-Jito-Solana-Works.html)
- [Ancilar - Implementing Effective MEV Protection 2025](https://medium.com/@ancilartech/implementing-effective-mev-protection-in-2025-c8a65570be3a)
- [Arkham - MEV Guide](https://info.arkm.com/research/beginners-guide-to-mev)
- [Figment - Jito MEV Driving Staking Rewards](https://figment.io/insights/jito-solana-and-maximal-extractable-value-mev-driving-all-time-high-staking-reward-rates-with-figment/)

### Bridges
- [Chainlink - What Is A Cross Chain Bridge](https://chain.link/education-hub/cross-chain-bridge)
- [HackenProof - 5 Loudest Web3 Bridge Hacks](https://hackenproof.com/blog/for-hackers/web3-bridge-hacks)
- [CertiK - Wormhole Bridge Exploit Analysis](https://www.certik.com/resources/blog/wormhole-bridge-exploit-incident-analysis)
- [CoinCryptoRank - DeFi Bridge Security Guide](https://coincryptorank.com/blog/defi-bridge-security-cross-chain-protection)
- [LI.FI - Solvers All The Way Down](https://li.fi/knowledge-hub/with-intents-its-solvers-all-the-way-down/)

### Composability
- [Chainscore Labs - Solana Atomic Transaction Composability](https://www.chainscorelabs.com/en/blog/solana-and-the-rise-of-high-performance-chains/solana-virtual-machine-svm-deep-dive/the-future-of-defi-lies-in-solanas-atomic-transaction-composability)
- [GitHub - Flash Loan Unlimited Facility (FLUF) Solana](https://github.com/jordan-public/flash-loan-unlimited-solana)
- [Kenson Investments - Solana DeFi 2025 Risks](https://kensoninvestments.com/solana-defi-in-2025-risks-rewards-and-regulatory-considerations/)

### Emerging Trends
- [CoinGecko - Top 9 Crypto Narratives 2026](https://www.coingecko.com/learn/crypto-narratives)
- [MEXC - 2026 Playbook: AI Agents to RWA Super-Cycle](https://blog.mexc.com/news/the-2026-playbook-from-ai-agents-to-the-rwa-super-cycle-here-are-the-narratives-defining-the-next-bull-run/)
- [Blockchain Council - Top Crypto Trends 2026](https://www.blockchain-council.org/cryptocurrency/top-crypto-trends/)
- [CoW DAO - Understanding Crypto Intents](https://cow.fi/learn/understanding-crypto-intents-the-future-of-your-de-fi-trades)
- [0x - Intents in DeFi](https://0x.org/post/intents-in-defi)
- [Ledger Academy - DeFAI Explained](https://www.ledger.com/academy/topics/defi/defai-explained-how-ai-agents-are-transforming-decentralized-finance)
- [Brickken - RWA Tokenization Trends 2025](https://www.brickken.com/post/rwa-tokenization-trends-2025)
- [Solana Docs - Confidential Transfers](https://solana.com/docs/tokens/extensions/confidential-transfer)
- [Helius - Confidential Balances on Solana](https://www.helius.dev/blog/confidential-balances)
- [DEV Community - Web3 UX in 2026](https://dev.to/wildanzr/web3-ux-finally-feels-normal-in-2026-smart-wallets-account-abstraction-and-the-end-of-seed-2okf)
- [BlockEden - Account Abstraction Goes Mainstream](https://blockeden.xyz/blog/2026/01/20/account-abstraction-smart-wallets-erc-4337-eip-7702-mainstream/)

---

*This research document was compiled in February 2026 from extensive web research. DeFi moves fast -- yields, protocols, and market conditions described here will change. Always verify current numbers before deploying capital.*
