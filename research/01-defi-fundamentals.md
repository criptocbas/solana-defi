# DeFi Fundamentals: A Comprehensive Technical Reference

> Written for experienced Solana developers entering the DeFi space.
> Last updated: February 2026

---

## Table of Contents

1. [What Is DeFi?](#1-what-is-defi)
2. [History and Evolution](#2-history-and-evolution)
3. [Core DeFi Primitives](#3-core-defi-primitives)
   - [3.1 Decentralized Exchanges (DEXes)](#31-decentralized-exchanges-dexes)
   - [3.2 Lending and Borrowing Protocols](#32-lending-and-borrowing-protocols)
   - [3.3 Stablecoins](#33-stablecoins)
   - [3.4 Derivatives](#34-derivatives)
   - [3.5 Insurance](#35-insurance)
   - [3.6 Yield Aggregators](#36-yield-aggregators)
   - [3.7 Liquid Staking](#37-liquid-staking)
4. [Composability: The Money Legos Paradigm](#4-composability-the-money-legos-paradigm)
5. [TVL (Total Value Locked)](#5-tvl-total-value-locked)
6. [Smart Contract Risk and the DeFi Trust Model](#6-smart-contract-risk-and-the-defi-trust-model)
7. [Key Terminology Glossary](#7-key-terminology-glossary)
8. [Solana-Specific Considerations](#8-solana-specific-considerations)
9. [References](#9-references)

---

## 1. What Is DeFi?

**Decentralized Finance (DeFi)** is the collective term for a set of financial services and instruments built on programmable, permissionless blockchains. The core thesis is straightforward: every financial primitive that exists in traditional finance -- lending, borrowing, trading, insurance, derivatives, asset management -- can be recreated as open-source smart contracts (or "programs" in Solana terminology) that execute deterministically without relying on trusted intermediaries like banks, brokerages, or clearinghouses.

### The Core Properties

| Property | Traditional Finance | DeFi |
|---|---|---|
| **Custody** | Intermediary holds your assets | User retains full custody via private keys |
| **Access** | Permissioned (KYC, credit checks, geography) | Permissionless (anyone with a wallet) |
| **Transparency** | Opaque internal ledgers | All state is on-chain and publicly auditable |
| **Settlement** | T+1 to T+3 days | Atomic, within a single transaction or block |
| **Composability** | Siloed systems, proprietary APIs | Open protocols that can call each other |
| **Operating Hours** | Business hours, weekdays | 24/7/365 |
| **Upgradability** | Centralized decisions | Governed by token holders or immutable code |

### What DeFi Is NOT

DeFi is not simply "crypto." Holding Bitcoin in a wallet is not DeFi. DeFi specifically refers to the layer of *financial applications* built on top of blockchain infrastructure -- protocols that actively do something with deposited assets: lend them, trade them, insure them, or compose them into structured products.

### The Trust Shift

In traditional finance, you trust **institutions** (banks, regulators, courts). In DeFi, you trust **code** (smart contracts), **cryptography** (digital signatures, hash functions), and **economic incentives** (game theory, staking penalties). This is not "trustless" -- it is a different trust model with its own risk profile, which we cover in depth in [Section 6](#6-smart-contract-risk-and-the-defi-trust-model).

---

## 2. History and Evolution

### Phase 0: The Conceptual Foundations (1990s-2008)

- **1994**: Nick Szabo articulates the concept of **smart contracts** -- self-executing agreements where the terms are encoded in software. The infrastructure to implement them did not yet exist.
- **1998**: Szabo also proposes "Bit Gold," a decentralized digital currency concept that prefigured Bitcoin.
- **2008**: Satoshi Nakamoto publishes the Bitcoin whitepaper, solving the double-spend problem through proof-of-work consensus.

### Phase 1: Bitcoin -- Programmable Money (2009-2014)

- **January 2009**: The Bitcoin network launches. It proves that decentralized consensus is achievable and that digital scarcity can be maintained through cryptographic protocols.
- Bitcoin's scripting language (Script) is intentionally limited -- it supports basic multi-signature transactions and time-locks, but it is not Turing-complete. This makes complex financial logic impractical on Bitcoin directly.
- **2012-2013**: Colored Coins and Mastercoin (later Omni) attempt to layer additional financial functionality on top of Bitcoin, but the scripting limitations prove constraining.
- **2014**: Rune Christensen begins work on **MakerDAO**, inspired by BitShares, with the goal of creating a decentralized stablecoin. This is arguably the first DeFi project, though it would not launch on Ethereum until later.

### Phase 2: Ethereum and the Smart Contract Revolution (2015-2017)

- **November 2013**: Vitalik Buterin publishes the Ethereum whitepaper, proposing a blockchain with a Turing-complete programming language.
- **July 2015**: Ethereum mainnet launches. For the first time, developers can deploy arbitrary logic to a decentralized network. The ERC-20 token standard enables anyone to create fungible tokens.
- **2016**: The DAO (Decentralized Autonomous Organization) raises $150M in ETH, then is exploited via a reentrancy vulnerability, leading to the Ethereum/Ethereum Classic hard fork. This event foreshadows the critical importance of smart contract security.
- **2017**: EtherDelta launches as one of the first on-chain order-book DEXes. The ICO boom drives massive experimentation with token issuance on Ethereum.
- **December 2017**: MakerDAO launches Single-Collateral DAI (SCD) on Ethereum mainnet, allowing users to mint the DAI stablecoin by locking ETH as collateral.

### Phase 3: The DeFi Primitives Emerge (2018-2019)

- **2018**: **Uniswap V1** launches, pioneering the Automated Market Maker (AMM) model -- no order books, just a constant-product formula (`x * y = k`). Hayden Adams builds it based on a concept proposed by Vitalik Buterin.
- **2018**: **Compound** launches as a lending/borrowing protocol with algorithmically determined interest rates.
- **November 2019**: MakerDAO upgrades to Multi-Collateral DAI (MCD), accepting multiple collateral types beyond just ETH.
- **January 2020**: **Aave** launches on Ethereum mainnet, introducing features like flash loans -- uncollateralized loans that must be repaid within a single transaction.

### Phase 4: DeFi Summer and Explosive Growth (2020)

- **June 2020**: Compound distributes the **COMP** governance token to users, pioneering **liquidity mining** (retroactively rewarding protocol users with tokens). This triggers "DeFi Summer."
- TVL across DeFi protocols surges from under $1 billion in January 2020 to over $15 billion by December 2020.
- **Yearn Finance** launches, creating the yield aggregator category -- automatically moving user funds between lending protocols to maximize returns.
- **SushiSwap** forks Uniswap, demonstrating both the power and vulnerability of open-source code in DeFi (the "vampire attack").
- **Curve Finance** launches, optimizing AMM design specifically for stablecoin-to-stablecoin swaps with minimal slippage.

### Phase 5: Multi-Chain Expansion and Maturation (2021-2023)

- **2021**: DeFi expands beyond Ethereum to Solana (Raydium, Serum, Marinade), Avalanche, BSC, Polygon, and others. TVL peaks above $180 billion in November 2021.
- **May 2022**: The **Terra/UST collapse** -- the algorithmic stablecoin UST loses its peg, triggering a $40B+ implosion that cascades across the ecosystem (Three Arrows Capital, Celsius, Voyager). This event permanently changes how the industry evaluates stablecoin designs.
- **2022-2023**: Focus shifts toward sustainability, real yield (protocol revenue distributed to token holders rather than inflationary emissions), and improved security practices.

### Phase 6: The Current Era (2024-2026)

- TVL recovers to over $120 billion by end of 2024 and continues growing.
- **Liquid staking** becomes the largest DeFi category by TVL.
- **Restaking** (EigenLayer and similar) introduces a new primitive: reusing staked assets to secure additional protocols.
- **Intent-based architectures** emerge, where users express desired outcomes and solvers compete to fulfill them.
- Solana's DeFi ecosystem matures significantly, frequently handling over 50% of global DEX volume.
- Stablecoin market cap surpasses $250 billion, with growing regulatory clarity in major jurisdictions.

---

## 3. Core DeFi Primitives

Each DeFi primitive mirrors a function from traditional finance but operates through smart contracts without intermediaries. These primitives are the building blocks from which all DeFi products are composed.

### 3.1 Decentralized Exchanges (DEXes)

A DEX enables peer-to-peer trading of tokens without a centralized intermediary. There are two primary architectures:

#### Order Book DEXes

Traditional exchanges match buy and sell orders in an order book. On-chain order books are expensive in terms of transaction costs (every order placement, cancellation, and modification is a transaction), but Solana's high throughput and low fees make this model viable. The original **Serum** (now **OpenBook**) on Solana was a fully on-chain central limit order book (CLOB).

#### Automated Market Makers (AMMs)

AMMs replace order books with **liquidity pools** -- smart contracts holding reserves of two (or more) tokens. Prices are determined algorithmically.

**The Constant Product Formula (Uniswap model):**

```
x * y = k
```

Where:
- `x` = reserve of Token A in the pool
- `y` = reserve of Token B in the pool
- `k` = a constant that must remain unchanged by trades (it only changes when liquidity is added or removed)

When a trader swaps Token A for Token B, they deposit some amount of A into the pool and withdraw some amount of B, such that the product `x * y` remains equal to `k`. The marginal price of Token A in terms of Token B at any point is approximately `y / x`.

**Key property:** Larger trades relative to pool reserves execute at exponentially worse rates (higher price impact/slippage). This is by design -- the formula ensures there is always *some* price at which any trade can execute, but the pool can never be fully drained.

**Fee mechanism:** Each swap pays a fee (e.g., 0.30% in Uniswap V2) that accrues to liquidity providers, increasing the value of `k` over time and compensating LPs for impermanent loss risk.

**AMM variants:**
- **Constant Product** (Uniswap V2, Raydium): `x * y = k` -- general-purpose, works for any token pair
- **Concentrated Liquidity** (Uniswap V3, Orca Whirlpools): LPs can specify price ranges, dramatically improving capital efficiency but increasing complexity
- **Stable Swap** (Curve, Saber on Solana): Optimized curve for assets that should trade near 1:1 (e.g., USDC/USDT), providing much lower slippage for like-kind assets
- **Weighted Pools** (Balancer): Generalize the constant product to arbitrary weight ratios (e.g., 80/20 instead of 50/50)

#### Solana DEX Landscape

Solana's architecture enables hybrid approaches. Jupiter, the dominant DEX aggregator on Solana, routes trades across dozens of liquidity sources (Raydium, Orca, Phoenix, Lifinity, OpenBook, and more) to find optimal execution. Phoenix uses a fully on-chain CLOB. Raydium combines an AMM with integration into the OpenBook order book.

---

### 3.2 Lending and Borrowing Protocols

Lending protocols are the second major DeFi primitive. They enable users to:
- **Lend** assets to earn interest (supplying liquidity to a pool)
- **Borrow** assets against posted collateral

#### How It Works

1. **Lenders** deposit tokens into a smart-contract-managed pool and receive interest-bearing receipt tokens (e.g., aTokens in Aave, cTokens in Compound).
2. **Borrowers** lock up collateral (typically worth significantly more than the loan value) and borrow from the pool.
3. **Interest rates** are determined algorithmically based on the **utilization rate** (how much of the supplied capital is currently borrowed). Higher utilization means higher rates for both borrowers and lenders.

#### Overcollateralization

Because DeFi lending is pseudonymous and there are no credit scores or legal recourse for defaults, all borrowing must be **overcollateralized**. A user might post $150 in collateral to borrow $100. The key parameters are:

- **Loan-to-Value (LTV)**: The maximum ratio of borrowed value to collateral value (e.g., 75% means you can borrow up to $75 per $100 of collateral)
- **Liquidation Threshold**: The collateral ratio at which a position becomes eligible for liquidation (e.g., 80%)
- **Health Factor**: `(Collateral Value * Liquidation Threshold) / Borrowed Value`. A health factor below 1.0 triggers liquidation.

#### Liquidation

If a borrower's collateral value falls (or their debt value rises) past the liquidation threshold, third-party **liquidators** can repay a portion of the debt and seize the corresponding collateral at a discount (the **liquidation bonus**, typically 5-10%). This mechanism protects lenders' capital but can be costly for borrowers in volatile markets.

```
Example:
- Deposit 100 SOL ($15,000) as collateral
- Borrow 7,500 USDC (50% LTV)
- Liquidation threshold: 80%
- SOL drops 40% to $90/SOL -> Collateral = $9,000
- Health Factor = ($9,000 * 0.80) / $7,500 = 0.96 (< 1.0)
- Position is now liquidatable
```

#### Key Protocols

- **Ethereum**: Aave, Compound, Morpho
- **Solana**: MarginFi, Kamino, Solend, Save (formerly Solend v2)

---

### 3.3 Stablecoins

Stablecoins are tokens designed to maintain a stable value, typically pegged to a fiat currency like the US dollar. They are the connective tissue of DeFi -- the base unit of account, the preferred collateral, and the medium of exchange in an otherwise volatile ecosystem.

#### Types of Stablecoins

**1. Fiat-Backed (Custodial)**

Backed 1:1 by fiat currency or short-term government securities held in reserve by a centralized custodian. Every token in circulation has a corresponding dollar (or equivalent) in a bank account.

| Stablecoin | Issuer | Reserves |
|---|---|---|
| USDC | Circle | Cash + US Treasuries |
| USDT | Tether | Commercial paper, Treasuries, cash equivalents |

- **Tradeoffs**: Most capital-efficient and stable, but introduce centralization risk (the issuer can freeze tokens, reserves must be trusted/audited, regulatory exposure).

**2. Crypto-Backed (Decentralized)**

Minted by depositing crypto collateral into a smart contract. Overcollateralized to absorb volatility.

| Stablecoin | Protocol | Collateral |
|---|---|---|
| DAI | MakerDAO | ETH, USDC, wBTC, and others |
| sUSD | Synthetix | SNX tokens |

- **Tradeoffs**: More decentralized but less capital-efficient (overcollateralization means $150+ locked per $100 of stablecoins). Susceptible to cascading liquidations during sharp market downturns.

**3. Algorithmic**

Maintain their peg through algorithmic supply adjustments -- minting or burning tokens in response to price deviations -- without holding equivalent reserves.

- **Critical Warning**: The collapse of TerraUSD (UST) in May 2022, which triggered a $40B+ loss across the ecosystem, demonstrated the fragility of purely algorithmic designs. UST's "death spiral" occurred when confidence broke and the mint/burn mechanism accelerated the depeg rather than correcting it.
- Some hybrid algorithmic designs (like FRAX) have survived by partially backing with real reserves.

**4. Emerging: Meta-Stablecoins and Yield-Bearing Stablecoins**

A new category is emerging in 2024-2025: stablecoins backed by baskets of other stablecoins, lending deposits, or liquid-staked positions. These aim to earn yield while maintaining a stable peg.

#### Stablecoin Market (2025-2026)

As of early 2025, stablecoin market cap exceeds $250 billion. Stablecoins have become the primary on-ramp for DeFi activity and increasingly for real-world payments, remittances, and trade settlement.

---

### 3.4 Derivatives

DeFi derivatives allow users to gain leveraged or synthetic exposure to assets without holding the underlying asset directly. They mirror traditional financial derivatives but settle on-chain.

#### Perpetual Futures ("Perps")

The dominant derivative instrument in DeFi. A perpetual future is a contract that tracks the price of an underlying asset with no expiry date. Traders can go long or short with leverage.

**Funding Rate Mechanism:** Perpetuals use a periodic funding rate to keep the contract price anchored to the spot price. When the perp price is above spot, longs pay shorts; when below, shorts pay longs. This creates arbitrage incentives that converge the two prices.

**Architecture models:**
- **Order-book based**: dYdX (runs its own appchain), Drift Protocol (Solana)
- **Pool-based / Oracle-priced**: GMX (Arbitrum), Jupiter Perps (Solana) -- traders trade against a liquidity pool, with oracle-fed prices, reducing slippage but introducing oracle risk
- **vAMM (Virtual AMM)**: Perpetual Protocol -- uses a virtual AMM for price discovery without requiring real liquidity in the pool

#### Options

On-chain options protocols allow trading calls and puts. Platforms like Lyra, Dopex, and Derive offer decentralized options markets. Options are more complex to implement on-chain due to the need for sophisticated pricing models (Black-Scholes or variants) and the capital intensity of option writing.

#### Synthetic Assets

Protocols like Synthetix create synthetic tokens ("Synths") that track the price of real-world assets (stocks, commodities, forex) via oracle price feeds. Users gain price exposure without holding the actual asset.

#### Key Solana Derivatives Protocols

- **Jupiter Perps**: Pool-based perpetual futures integrated into the Jupiter ecosystem
- **Drift Protocol**: Full-featured perpetual futures with an on-chain order book
- **Zeta Markets**: Options and futures on Solana

---

### 3.5 Insurance

DeFi insurance protocols provide coverage against risks inherent to the ecosystem -- primarily smart contract exploits, but also oracle failures, stablecoin depegs, and bridge hacks.

#### How DeFi Insurance Works

1. **Cover buyers** pay premiums to purchase protection for specific risks (e.g., "smart contract failure on Aave").
2. **Capital providers** (underwriters) stake assets into risk pools, earning premiums in return for bearing the payout risk.
3. **Claims assessment** is typically handled through decentralized governance (token-holder votes) or parametric triggers (automatic payout when a predefined event occurs).

#### Key Protocols

- **Nexus Mutual**: The largest DeFi insurance protocol, structured as a decentralized mutual. Over $6 billion in digital assets protected since 2019. Members stake NXM tokens to underwrite risk and vote on claims.
- **InsurAce**: Multi-chain coverage across 20+ chains and 140+ protocols. Offers both smart contract cover and custodial risk cover.
- **Neptune Mutual**: Parametric cover with rule-based payouts that do not require individual claims assessment.

#### Limitations

- Coverage is still limited relative to total DeFi TVL (most DeFi positions are uninsured).
- Claims processes can be slow or contentious.
- Correlated risk: a major systemic event (like the Terra collapse) can overwhelm insurance pools.
- The insurance protocols themselves are smart contracts and thus carry their own smart contract risk.

---

### 3.6 Yield Aggregators

Yield aggregators automate the process of finding and executing the highest-yield opportunities across DeFi. They are the "robo-advisors" of DeFi.

#### How They Work

1. Users deposit assets into a yield aggregator **vault**.
2. The vault's strategy contract automatically allocates funds across lending protocols, liquidity pools, and farming opportunities.
3. Returns are auto-compounded (harvested and reinvested) to maximize yield.
4. The aggregator typically charges a performance fee (e.g., 10-20% of profits).

#### Strategy Examples

- Deposit USDC into a vault that lends on whichever protocol (Aave, Compound, Morpho) currently offers the highest rate, automatically rebalancing as rates change.
- Deposit ETH-USDC LP tokens into a vault that auto-compounds trading fees and farms reward tokens, selling them for more LP tokens.
- Multi-step leveraged farming: deposit collateral, borrow stablecoins, deploy to a farm, use farm rewards to repay debt, repeat.

#### Key Protocols

- **Yearn Finance** (Ethereum): Pioneer of the yield aggregator category. Its "vaults" (now V3) have processed billions in deposits.
- **Kamino Finance** (Solana): Automated liquidity management and yield optimization for concentrated liquidity positions on Orca and Raydium.
- **Beefy Finance**: Multi-chain yield optimizer.

#### Risks

- **Smart contract risk** is compounded because aggregators interact with multiple protocols -- a vulnerability in any one of them can affect the vault.
- **Strategy risk**: Automated strategies can underperform or take on more risk than expected.
- **Composability risk**: Dependencies on external protocols create cascading failure scenarios.

---

### 3.7 Liquid Staking

Liquid staking solves a fundamental problem: when you stake tokens to secure a Proof-of-Stake network, those tokens are locked and cannot be used for anything else. Liquid staking protocols issue a **derivative token** representing your staked position, which can then be freely used in DeFi.

#### How It Works

1. User deposits SOL (or ETH, etc.) into a liquid staking protocol.
2. The protocol stakes the tokens with validators on the user's behalf.
3. User receives a **Liquid Staking Token (LST)** -- e.g., mSOL (Marinade), jitoSOL (Jito), bSOL (BlazeStake), stSOL (Lido).
4. The LST appreciates in value relative to the base token as staking rewards accrue (or rebases to reflect rewards).
5. The LST can be used as collateral for lending, provided as liquidity in DEX pools, or composed into other DeFi strategies.

#### Why It Matters

Liquid staking unlocks **capital efficiency**: instead of choosing between staking rewards OR DeFi yields, users can earn both simultaneously. On Solana, where native staking yields ~6-8% APY, the ability to also use staked SOL in DeFi is extremely valuable.

#### Restaking: The Next Evolution

**Restaking** (pioneered by EigenLayer on Ethereum, emerging on Solana) takes this further: staked assets (or LSTs) can be "restaked" to provide security to additional protocols or services, earning additional yield layers. This creates powerful capital efficiency but also introduces additional smart contract and slashing risk.

#### Key Solana Liquid Staking Protocols

| Protocol | LST Token | Notable Feature |
|---|---|---|
| Marinade Finance | mSOL | Largest Solana LST, delegates across 400+ validators |
| Jito | jitoSOL | Includes MEV rewards from Jito's validator client |
| BlazeStake | bSOL | Community-focused, supports smaller validators |
| Sanctum | Various (INF, etc.) | LST infrastructure layer, enables instant LST-to-SOL conversion |

---

## 4. Composability: The Money Legos Paradigm

### What Is Composability?

Composability is the ability of DeFi protocols to interact with, build upon, and integrate each other in a permissionless manner. Because DeFi protocols are open-source smart contracts deployed on shared public blockchains, any protocol can call any other protocol's functions in a single atomic transaction.

This property is often called **"Money Legos"** -- each protocol is a building block that can be snapped together with others to create financial products that would be impossible (or prohibitively expensive) in traditional finance.

### Why It Matters

In traditional finance, integrating two financial systems requires:
- Bilateral legal agreements
- API access negotiations
- Settlement reconciliation across days
- Regulatory approval for new product combinations

In DeFi, a developer can compose protocols together in a single smart contract deployment, and the integration is **atomic** -- either the entire composed operation succeeds or it all reverts. No partial failures, no settlement risk.

### Concrete Composability Examples

**Example 1: Leveraged Yield Farming (3 protocols in 1 transaction)**
1. Deposit SOL into Marinade -> receive mSOL (liquid staking)
2. Deposit mSOL into MarginFi as collateral -> borrow USDC (lending)
3. Swap USDC for more SOL on Jupiter -> stake again (DEX)
4. Result: leveraged SOL staking exposure

**Example 2: Flash Loan Arbitrage (executed atomically)**
1. Borrow 1M USDC via flash loan from Aave (no collateral needed)
2. Buy Token X on DEX A where it is cheaper
3. Sell Token X on DEX B where it is more expensive
4. Repay 1M USDC + fee to Aave
5. Keep the profit
6. If any step fails, the entire transaction reverts (no risk to the lender)

**Example 3: Curve-Compound Composition**
Curve maintains AMM pools for stablecoin trading. While assets sit idle in Curve pools, they are simultaneously supplied to Compound (or Aave) to earn lending interest. Liquidity providers earn both trading fees AND lending yield on the same capital.

### Composability on Solana

Solana's programming model is particularly well-suited to composability:
- **Cross-Program Invocations (CPIs)** allow any Solana program to invoke any other program within the same transaction.
- **Transaction-level atomicity** ensures composed operations either fully succeed or fully revert.
- Solana's high throughput and low fees make multi-step composed transactions economically viable (on Ethereum, the gas cost of a 5-step composed transaction could be prohibitive).

### The Risks of Composability

Composability creates **dependency chains**. If Protocol A depends on Protocol B, which depends on Protocol C, a bug or exploit in Protocol C can cascade upward. This is sometimes called **"composability risk"** or **"DeFi contagion."** The Terra/UST collapse demonstrated this vividly -- protocols that held UST as collateral or used it in strategies all suffered losses simultaneously.

---

## 5. TVL (Total Value Locked)

### What It Is

**Total Value Locked (TVL)** measures the total value of crypto assets deposited into a DeFi protocol's smart contracts. It is expressed in USD terms and is the most widely used metric for gauging the size and adoption of DeFi protocols and the ecosystem as a whole.

```
TVL = Sum of all assets deposited in a protocol's smart contracts, valued at current market prices
```

**DeFiLlama** (defillama.com) is the most widely used TVL aggregator, tracking thousands of protocols across all chains.

### Why It Matters

- **Adoption proxy**: Higher TVL generally indicates more users trust the protocol with their capital.
- **Liquidity depth**: For DEXes and lending protocols, TVL directly correlates with execution quality (lower slippage, better rates).
- **Security signal**: Protocols with high TVL have been "battle-tested" with real capital at stake, though this is not a guarantee of safety.
- **Ecosystem comparison**: TVL by chain gives a rough sense of where DeFi activity is concentrated (Ethereum, Solana, Arbitrum, BSC, etc.).

### Historical Context

| Date | Total DeFi TVL |
|---|---|
| January 2020 | ~$600 million |
| December 2020 | ~$15 billion |
| November 2021 (peak) | ~$180 billion |
| June 2022 (post-Terra) | ~$40 billion |
| December 2024 | ~$120 billion |

### Limitations and Criticisms

TVL is a useful but deeply flawed metric. Understanding its limitations is critical:

**1. Price Sensitivity (Reflexivity)**
TVL is denominated in USD, so when crypto prices rise, TVL rises even if no new capital enters the system. A 50% increase in ETH price mechanically inflates the TVL of every protocol holding ETH. This makes TVL partially a proxy for market sentiment rather than real adoption.

**2. Double-Counting**
The same assets can be counted multiple times across protocols. If you deposit ETH into Lido (counted in Lido's TVL), receive stETH, then deposit stETH into Aave (counted in Aave's TVL), the same underlying ETH is counted twice. Across the ecosystem, double-counting can inflate aggregate TVL significantly.

**3. Lack of Standardization**
Different aggregators calculate TVL differently. Some include staking, some do not. Some count borrowed assets, some do not. There is no universally agreed-upon standard. A BIS (Bank for International Settlements) study found that 10.5% of DeFi protocols rely on off-chain data sources, making independent verification difficult.

**4. Manipulation / Mercenary Capital**
TVL can be artificially inflated through incentive programs (token emissions that attract "mercenary capital" which leaves when incentives end). High TVL driven by unsustainable emissions is a red flag, not a health signal.

**5. Does Not Reflect Revenue or Efficiency**
A protocol can have $10B in TVL but generate minimal revenue. TVL says nothing about capital efficiency, profitability, or sustainability. Better metrics include:
- **Revenue**: Protocol fees collected
- **TVL / Revenue ratio**: How efficiently capital is deployed
- **Organic TVL**: TVL remaining after removing incentivized (subsidized) liquidity

**6. Concentration Risk**
A protocol's TVL may come from a small number of whale depositors. If a few large players withdraw, TVL can collapse overnight, impacting liquidity and user confidence.

### Better Metrics to Consider Alongside TVL

| Metric | What It Measures |
|---|---|
| Protocol Revenue | Actual fees generated by the protocol |
| DEX Volume | Real trading activity (not just parked capital) |
| Active Addresses | Number of unique users interacting with the protocol |
| Fees to TVL Ratio | Capital efficiency |
| Organic TVL | TVL minus incentive-driven deposits |

---

## 6. Smart Contract Risk and the DeFi Trust Model

### The Fundamental Tradeoff

DeFi eliminates intermediary risk but introduces **smart contract risk**. Instead of trusting a bank not to mismanage your money, you trust that:
1. The smart contract code is correct and free of exploitable bugs
2. The protocol's economic design is sound under all market conditions
3. Any privileged roles (admin keys, upgrade authority) are properly secured
4. The dependencies (oracles, bridges, other protocols) are reliable

### The Scale of the Problem

Since 2016, on-chain losses from exploits and hacks have exceeded **$7.5 billion**, with $5.7 billion stolen specifically from DeFi protocols. Notable incidents include:

| Incident | Year | Loss | Attack Vector |
|---|---|---|---|
| The DAO | 2016 | $60M | Reentrancy |
| Poly Network | 2021 | $611M | Access control flaw (funds returned) |
| Ronin Bridge (Axie) | 2022 | $625M | Compromised validator keys |
| Wormhole Bridge | 2022 | $320M | Signature verification bypass |
| Nomad Bridge | 2022 | $190M | Initialization bug allowed arbitrary withdrawals |
| Euler Finance | 2023 | $197M | Flash loan + accounting logic flaw |
| Mixin Network | 2023 | $200M | Cloud infrastructure compromise |

### Common Vulnerability Categories

**1. Reentrancy Attacks**
A malicious contract calls back into the victim contract before the first execution is complete, allowing repeated withdrawals. The DAO hack (2016) was a reentrancy attack. Modern Solidity has protections (checks-effects-interactions pattern, reentrancy guards), but variants continue to appear.

Note for Solana developers: Solana's runtime model makes traditional reentrancy attacks more difficult because CPI calls cannot re-enter the calling program in the same instruction. However, cross-program reentrancy through intermediate programs and logical reentrancy across transactions are still concerns.

**2. Oracle Manipulation**
Protocols that rely on on-chain price feeds (e.g., using a DEX pool's spot price as an oracle) are vulnerable to manipulation. An attacker can use a flash loan to temporarily distort a pool's price, then exploit protocols that read that price. This is why dedicated oracle networks (Chainlink, Pyth) exist -- they aggregate prices from multiple independent sources, making manipulation far more expensive.

**3. Flash Loan Attacks**
Flash loans themselves are not inherently malicious -- they are a powerful tool. But they enable attacks that would otherwise require prohibitive capital. An attacker can borrow millions, manipulate a market, exploit a vulnerability, and repay the loan -- all in one atomic transaction with no upfront capital.

**4. Access Control Flaws**
Missing or incorrect permission checks that allow unauthorized users to call privileged functions (e.g., minting tokens, withdrawing reserves, upgrading contracts).

**5. Logic Errors / Economic Exploits**
Bugs in the protocol's business logic or economic design that allow value extraction. These are often the hardest to catch because they require understanding the protocol's intended behavior under all conditions.

**6. Bridge Vulnerabilities**
Cross-chain bridges are disproportionately targeted because they hold large amounts of locked assets and their security models are complex (requiring trust assumptions across multiple chains).

### The Audit Landscape

- **Security audits** by firms like Trail of Bits, OpenZeppelin, Halborn, OtterSec (Solana-focused), and Neodyme are standard practice before mainnet deployment.
- Audits are necessary but **not sufficient**: auditors review code at a point in time, may miss edge cases, and cannot guarantee safety. Multiple audits from different firms provide better coverage.
- **Bug bounty programs** (Immunefi is the largest platform) incentivize white-hat hackers to find and report vulnerabilities. Major protocols offer bounties ranging from $100K to $10M+.
- **Formal verification** -- mathematically proving that code meets its specification -- is the gold standard but is expensive and only practical for core protocol logic.

### Evaluating DeFi Protocol Risk (A Framework)

When assessing a DeFi protocol, consider:

1. **Code**: Is it audited? By whom? How many audits? Is the code open-source? Is there a bug bounty?
2. **Time**: How long has the protocol been live with significant TVL? Battle-tested code is (somewhat) safer.
3. **Upgradeability**: Can the contracts be upgraded? By whom? Is there a timelock? Upgradeable contracts are more flexible but introduce admin key risk.
4. **Oracle dependency**: What oracle does the protocol use? How is it configured? What happens if the oracle fails?
5. **Admin controls**: Does a multisig or DAO control critical parameters? What is the multisig threshold? Is there a timelock for changes?
6. **Economic design**: Does the protocol work under extreme market conditions (90%+ drawdowns, oracle failures, mass liquidations)?
7. **Dependencies**: What other protocols does this depend on? What is the weakest link in the dependency chain?

---

## 7. Key Terminology Glossary

### Liquidity Pool

A smart contract that holds reserves of two or more tokens, enabling decentralized trading. Liquidity pools are the foundational infrastructure for AMM-based DEXes. Instead of a traditional order book, traders swap against the pool's reserves, and prices adjust according to the AMM's bonding curve formula.

### Liquidity Provider (LP)

A user who deposits tokens into a liquidity pool. LPs earn a share of the trading fees generated by the pool, proportional to their share of the pool's total liquidity. In return, they take on impermanent loss risk and smart contract risk.

When you deposit into a pool, you typically receive **LP tokens** (or an NFT position in concentrated liquidity systems like Uniswap V3 / Orca Whirlpools) representing your proportional share of the pool.

### Slippage

The difference between the expected price of a trade and the actual executed price. Slippage occurs because:
1. **Price impact**: Your trade changes the pool's reserves, moving the price against you (larger trades = more slippage)
2. **Market movement**: The price moves between when you submit and when your transaction is confirmed

**Slippage tolerance** is a parameter you set (e.g., 0.5%) that defines the maximum acceptable deviation. If actual slippage exceeds your tolerance, the transaction reverts.

### Impermanent Loss (IL)

The loss in value that a liquidity provider experiences compared to simply holding the deposited tokens outside the pool. It occurs because AMM pools maintain a fixed ratio of assets, so when one token's price changes relative to the other, arbitrageurs rebalance the pool, leaving the LP with more of the depreciating token and less of the appreciating one.

**Formula for a standard 50/50 pool:**

```
IL = (2 * sqrt(price_ratio)) / (1 + price_ratio) - 1
```

Where `price_ratio` = new price / original price of the volatile token.

**Key impermanent loss values:**

| Price Change | Impermanent Loss |
|---|---|
| 1.25x (25% up) | -0.6% |
| 1.50x (50% up) | -2.0% |
| 2.00x (100% up) | -5.7% |
| 3.00x (200% up) | -13.4% |
| 5.00x (400% up) | -25.5% |

Note: The loss is **symmetrical** -- a 2x price increase and a 50% price decrease both result in ~5.7% IL.

The loss is "impermanent" because if prices return to their original ratio, the loss disappears. However, in practice, prices often do not return, making the loss very much permanent. Some in the community prefer the term **"divergence loss"** for this reason.

LPs profit when trading fee income exceeds impermanent loss. High-volume pools with relatively stable price pairs (e.g., USDC/USDT) tend to be the most reliably profitable for LPs.

### Flash Loans

Uncollateralized loans that must be borrowed and repaid within a single atomic transaction (a single block on Ethereum, a single transaction on Solana). If the borrower fails to repay, the entire transaction reverts as if it never happened -- the lender faces zero risk.

**Use cases:**
- **Arbitrage**: Exploit price differences across DEXes without needing upfront capital
- **Self-liquidation**: Users can flash-loan to repay their own debt and retrieve collateral in one transaction, avoiding liquidation penalties
- **Collateral swaps**: Swap the collateral backing a loan without closing the position
- **Exploits**: Unfortunately, flash loans are also used to fund attacks (oracle manipulation, economic exploits) since they remove the capital barrier

Flash loans were introduced by **Aave** on Ethereum. On Solana, flash loans are implemented by various protocols, though Solana's transaction model (which processes all instructions atomically within a transaction) provides a natural framework for flash-loan-like patterns.

### Oracles

Oracles are systems that feed external (off-chain) data into smart contracts. Smart contracts cannot natively access data outside their blockchain -- they cannot call an API, scrape a website, or read a price from an exchange. Oracles bridge this gap.

**The Oracle Problem**: How do you get reliable, tamper-resistant real-world data into a trustless system? Any centralized data feed becomes a single point of failure. Solutions include:

- **Chainlink**: The dominant oracle network on Ethereum and many EVM chains. Uses a decentralized network of node operators who aggregate data from multiple sources. An attacker would need to compromise 50%+1 of nodes on a price feed to manipulate it.
- **Pyth Network**: The dominant oracle on Solana. Uses a "pull" model where data is only delivered on-chain when requested (saving costs). Sources data from 120+ institutional market participants (trading firms, exchanges). Offers sub-second price updates with confidence intervals.
- **Switchboard**: Another major Solana oracle, offering customizable data feeds.

**Oracle manipulation** remains one of the most common DeFi attack vectors. Protocols that use on-chain spot prices from a single DEX as their oracle (rather than a dedicated oracle network) are particularly vulnerable.

**Best practices:**
- Use time-weighted average prices (TWAPs) rather than spot prices
- Aggregate from multiple independent oracle sources
- Implement circuit breakers (halt operations if price deviates beyond a threshold)
- Use oracle confidence intervals (Pyth provides these natively)

### Governance Tokens

Tokens that grant holders voting rights over a protocol's parameters and development. Governance tokens are the mechanism through which DeFi protocols achieve decentralized decision-making.

**How governance works:**
1. Token holders submit proposals (parameter changes, treasury allocations, upgrades)
2. The community votes (typically 1 token = 1 vote)
3. If a proposal passes quorum and threshold requirements, it is executed -- often automatically via smart contract

**Notable governance tokens:**
- **UNI** (Uniswap): Controls the Uniswap protocol's fee switch, treasury, and upgrades
- **AAVE** (Aave): Governs risk parameters, asset listings, and protocol upgrades
- **JUP** (Jupiter): Governs the Jupiter ecosystem on Solana
- **MKR** (MakerDAO): Governs DAI stability parameters, collateral types, and risk management

**Challenges:**
- **Plutocracy**: Voting power is proportional to token holdings, concentrating power among wealthy holders ("whales")
- **Low participation**: Most token holders do not vote, leading to governance by a small minority
- **Voter apathy / rational ignorance**: The cost of researching proposals often exceeds the individual benefit of voting
- **Short-termism**: Token holders may vote for short-term yield extraction over long-term protocol health

**Delegation** has emerged as a partial solution: token holders delegate their voting power to informed representatives ("delegates") who vote on their behalf.

### MEV (Maximal Extractable Value)

The value that can be extracted by block producers (validators/miners) or specialized bots ("searchers") by reordering, inserting, or censoring transactions within a block.

**Common MEV strategies:**
- **Front-running**: Seeing a pending large trade and placing your own trade first to profit from the price impact
- **Back-running**: Placing a trade immediately after a large trade to capture the price correction
- **Sandwich attacks**: Placing a buy before AND a sell after a victim's trade, profiting from the slippage the victim experiences. The attacker's buy pushes the price up before the victim buys, then the attacker sells at the inflated price.
- **Liquidation sniping**: Racing to be the first to liquidate an undercollateralized position and claim the liquidation bonus
- **Arbitrage**: Detecting and executing cross-DEX price discrepancies

**MEV on Solana**: Jito Labs operates the dominant MEV infrastructure on Solana. Jito's validator client allows searchers to submit transaction bundles (ordered groups of transactions), with tips paid to validators. This has centralization implications but also enables MEV to be partially redistributed to stakers (jitoSOL holders earn MEV rewards).

**Protection**: Users can protect themselves by using low slippage tolerances, private transaction submission (bypassing the public mempool), and MEV-aware DEX aggregators.

### Automated Market Maker (AMM)

A protocol that uses mathematical formulas to price assets in liquidity pools, replacing the role of human market makers and order books. The most common formula is the **constant product** (`x * y = k`), but many variants exist (concentrated liquidity, stable swap curves, weighted pools). AMMs are the dominant exchange mechanism in DeFi because they provide continuous liquidity without requiring active market makers.

### Yield Farming / Liquidity Mining

The practice of depositing assets into DeFi protocols to earn rewards, typically in the form of the protocol's governance token. First popularized by Compound's COMP distribution in June 2020. Yield farming can refer broadly to any strategy that seeks to maximize returns across DeFi protocols -- including lending, LP-ing, staking, and combining multiple strategies.

### Rug Pull

A type of exit scam where the creators of a DeFi protocol drain liquidity or exploit admin privileges to steal user funds. Common in unaudited, anonymous-team projects. Warning signs include: unrenounced admin keys, no audit, no timelock, anonymous team, unsustainably high APY promises.

### Timelock

A smart contract mechanism that enforces a delay between when a protocol change is proposed and when it takes effect (e.g., 48-hour timelock). This gives users time to review changes and withdraw funds if they disagree -- a critical safety measure for upgradeable protocols.

### Health Factor

In lending protocols, the ratio that indicates how close a borrowing position is to liquidation. Calculated as `(Collateral Value * Liquidation Threshold) / Total Debt`. Values above 1.0 are safe; at or below 1.0, the position can be liquidated.

### Utilization Rate

In lending protocols, the ratio of borrowed assets to total supplied assets in a pool. Higher utilization means higher interest rates (more demand for borrowing relative to supply). Most protocols target a specific utilization rate and sharply increase rates above that threshold (the "kink" in the interest rate curve) to incentivize new deposits.

---

## 8. Solana-Specific Considerations

As a Solana developer, there are architectural differences that affect how DeFi works on Solana versus Ethereum:

### Programming Model

- Solana uses **programs** (not "smart contracts"), written in Rust (or via the Anchor framework). Programs are stateless; state is stored in separate **accounts**.
- Solana's **account model** requires explicitly declaring all accounts a transaction will read or write, enabling parallel transaction processing. This is fundamentally different from Ethereum's global state model.
- **Cross-Program Invocations (CPIs)** are Solana's equivalent of Ethereum's contract-to-contract calls. CPIs enable composability but require the caller to pass all necessary accounts.

### Performance Advantages

- **~400ms block times** vs. Ethereum's ~12s -- enabling near-real-time DeFi applications
- **Sub-cent transaction fees** vs. Ethereum's $1-50+ gas fees -- making micro-transactions and complex multi-step DeFi operations economically viable
- **High throughput** -- Solana regularly processes thousands of transactions per second, enabling on-chain order books and high-frequency trading strategies that are impractical on Ethereum L1

### Solana DeFi Ecosystem (Key Protocols)

| Category | Protocols |
|---|---|
| DEX Aggregator | Jupiter |
| AMM DEX | Raydium, Orca (Whirlpools), Lifinity |
| Order Book DEX | Phoenix, OpenBook |
| Lending | MarginFi, Kamino, Save (Solend) |
| Liquid Staking | Marinade, Jito, BlazeStake, Sanctum |
| Perpetuals | Jupiter Perps, Drift Protocol |
| Oracle | Pyth Network, Switchboard |
| Yield | Kamino, Tulip |
| Stablecoin | USDC (native), UXD |

### MEV on Solana

Solana's continuous block production (no mempool in the traditional sense) and parallel execution create a different MEV landscape than Ethereum. Jito's bundle infrastructure is the primary MEV channel, where searchers pay tips to validators for priority transaction ordering. Understanding Solana MEV is essential for building protocols that protect users from value extraction.

---

## 9. References

### General DeFi Fundamentals
- [Decentralized Finance - Wikipedia](https://en.wikipedia.org/wiki/Decentralized_finance)
- [DeFi Fundamentals: A Beginner's Guide to Decentralised Finance (2025) - Bitcoinsensus](https://www.bitcoinsensus.com/learn/beginners-guides/defi-fundamentals-a-beginners-guide-to-decentralised-finance-2025)
- [DeFi Basics: Decentralized Finance and How it Works - Blockpit](https://www.blockpit.io/blog/what-is-defi-decentralized-finance)
- [Decentralized Finance in 2025: Know the Risks and Rewards - G2](https://learn.g2.com/decentralized-finance)
- [What is 'decentralized finance' and what can it actually do? - World Economic Forum](https://www.weforum.org/stories/2025/10/decentralized-finance-financial-markets-in-practice/)

### History and Evolution
- [History of DeFi - From Inception to 2021 and Beyond - Finematics](https://finematics.com/history-of-defi-explained/)
- [A Brief History of DeFi - Decrypt](https://decrypt.co/resources/a-brief-history-of-defi-learn)
- [When Did DeFi Begin? A Timeline of Decentralized Finance - Bitfinity](https://www.blog.bitfinity.network/when-did-defi-begin-timeline-decentralized-finance/)
- [History of DeFi From Bitcoin to Modern Decentralized Finance - NadCab](https://www.nadcab.com/blog/history-of-defi)
- [History of Crypto: DeFi revolution during a global crisis - Cointelegraph](https://cointelegraph.com/news/de-fi-revolution-global-crisis-2020-2021)

### DeFi Primitives and Building Blocks
- [Financial Primitives in DeFi Explained Building Blocks - NadCab](https://www.nadcab.com/blog/financial-primitives-in-defi)
- [The Technology of Decentralized Finance (DeFi) - BIS Working Papers](https://www.bis.org/publ/work1066.pdf)
- [DeFi Ecosystem: Primitives and Technology Stack - Medium](https://medium.com/@kaishinaw/defi-ecosystem-primitives-and-technology-stack-85401fdd62ad)
- [What Is DeFi 2.0? - Chainlink](https://chain.link/education-hub/defi-2-0)

### AMMs and DEXes
- [How Uniswap Works - Uniswap Docs](https://docs.uniswap.org/contracts/v2/concepts/protocol-overview/how-uniswap-works)
- [What is an Automated Market Maker? - Uniswap Blog](https://blog.uniswap.org/what-is-an-automated-market-maker)
- [Constant Function Market Maker - Uniswap V3 Development Book](https://uniswapv3book.com/milestone_0/constant-function-market-maker.html)
- [Automated Market Makers (AMMs): Math, Risks & Solidity Code - Speedrun Ethereum](https://speedrunethereum.com/guides/automated-market-makers-math)

### Composability
- [What are "Money Legos" in DeFi? Composability Explained - Boxmining](https://www.boxmining.com/defi-money-legos/)
- [DeFi Composability: Protocol Integration and the Money Legos Paradigm - CoinCryptoRank](https://coincryptorank.com/blog/defi-composability-protocol-integration)
- [Composability - "Money Legos" and Beyond - Alchemy University](https://www.alchemy.com/university/intro-to-blockchain/composability)
- [Money Legos - IQ.wiki](https://iq.wiki/wiki/money-legos)

### TVL
- [What Total Value Locked (TVL) and Why Users Monitor This Metric - CoinGecko](https://www.coingecko.com/learn/total-value-locked)
- [Why Total Value Locked (TVL) Isn't the Best Metric for DeFi Success - Amberdata](https://blog.amberdata.io/total-value-locked-why-its-not-a-great-defi-metric)
- [Towards verifiability of total value locked (TVL) in decentralized finance - BIS](https://www.bis.org/publ/work1268.htm)

### Smart Contract Security
- [Smart Contract Vulnerabilities: How Hackers Exploit Flaws in DeFi - OSL](https://www.osl.com/hk-en/academy/article/smart-contract-vulnerabilities-how-hackers-exploit-flaws-in-defi)
- [Smart Contract Vulnerabilities, Risks and How to Mitigate Them - QuillAudits](https://www.quillaudits.com/blog/smart-contract/smart-contract-vulnerabilities)
- [The Biggest Hacks and Exploits in DeFi History - DeFi Planet](https://defi-planet.com/2025/05/the-biggest-hacks-and-exploits-in-defi-history-what-we-can-learn-from-them/)
- [Comprehensive List of DeFi Hacks & Exploits - ChainSec](https://chainsec.io/defi-hacks/)
- [Hacks - DeFiLlama](https://defillama.com/hacks)

### Stablecoins
- [Types of Stablecoins Explained (2025) - Token Metrics](https://www.tokenmetrics.com/blog/types-of-stablecoins-a-complete-guide-for-2025)
- [Fiat vs Algorithmic Stablecoins: What You Need to Know - USDC](https://www.usdc.com/learn/fiat-backed-vs-algorithmic-stablecoins)

### Lending and Liquidation
- [DeFi Lending: Liquidations and Collateral - RareSkills](https://rareskills.io/post/defi-liquidations-collateral)
- [What is Overcollateralization? - Cube Exchange](https://www.cube.exchange/what-is/overcollateralization)
- [DeFi Liquidation Protocols: How They Work - Krayon Digital](https://www.krayondigital.com/blog/defi-liquidation-protocols-how-they-work)

### Derivatives
- [Deep Dive into DeFi Derivatives - MixBytes](https://mixbytes.io/blog/deep-dive-into-defi-derivatives)
- [Understanding Perpetual DEXs: The Future of On-Chain Derivatives - LCX](https://lcx.com/en/understanding-perpetual-dexs-the-future-of-on-chain-derivatives)
- [Derivatives in DeFi Explained - Finematics](https://finematics.com/derivatives-in-defi-explained/)

### Insurance
- [DeFi Insurance Protocols: How Nexus Mutual and InsurAce Mitigate Risks - Mitosis University](https://university.mitosis.org/defi-insurance-protocols-how-nexus-mutual-and-insurace-mitigate-risks-in-decentralized-finance/)
- [DeFi Insurance Protocols: Risks and Rewards - Three Sigma](https://threesigma.xyz/blog/infrastructure/defi-insurance-guide-risks-rewards)

### Oracles
- [Market Manipulation vs. Oracle Exploits - Chainlink](https://chain.link/education-hub/market-manipulation-vs-oracle-exploits)
- [The Full Guide to Price Oracle Manipulation Attacks - Cyfrin](https://www.cyfrin.io/blog/price-oracle-manipulation-attacks-with-examples)
- [How Pyth Network Brings Secure, On-Demand Price Feeds to DeFi - CCN](https://www.ccn.com/education/crypto/pyth-network-secure-on-demand-price-feeds-defi/)

### Impermanent Loss
- [Impermanent Loss Explained: The Math Behind DeFi's Hidden Risk - Speedrun Ethereum](https://speedrunethereum.com/guides/impermanent-loss-math-explained)
- [Decentralized Finance and Impermanent Loss - Gemini](https://www.gemini.com/cryptopedia/decentralized-finance-impermanent-loss-defi)

### MEV
- [Understanding MEV Attacks - CoW Protocol](https://cow.fi/learn/mev-attacks-explained)
- [What is MEV and How to Protect Your Transactions on Solana - QuickNode](https://www.quicknode.com/guides/solana-development/defi/mev-on-solana)

### Governance
- [What is a Governance Token? - Coinbase](https://www.coinbase.com/learn/crypto-basics/what-is-a-governance-token)
- [Governance Tokens Explained: How Voting Rights Shape DeFi and DAOs - OpenSea](https://opensea.io/learn/token/what-are-governance-tokens)

### Solana vs. Ethereum
- [Solana vs. Ethereum: Performance, Architecture, and Potential - Ledger](https://www.ledger.com/academy/topics/crypto/solana-vs-ethereum-performance-guide)
- [Solana vs Ethereum: Key Differences, Pros, and Cons Compared - CoinTracker](https://www.cointracker.io/blog/solana-vs-ethereum)

### DeFi Terminology
- [DeFi Crypto Glossary: 100+ Terms and Definitions - tastycrypto](https://www.tastycrypto.com/defi/defi-glossary/)
- [The Complete DeFi Glossary - Tangem](https://tangem.com/en/blog/post/defi-glossary/)

### Liquid Staking
- [Liquid Staking in DeFi: Ultimate Beginners Guide - tastycrypto](https://www.tastycrypto.com/defi/liquid-staking/)
- [Liquid Staking Derivatives and Their Place in DeFi - 1inch](https://blog.1inch.com/liquid-staking-derivatives-and-their-place-in-defi/)

### Yield Aggregators
- [Advanced Applications of DeFi: Complete Guide - Blockpit](https://www.blockpit.io/blog/advanced-defi-applications)
- [What is a DeFi Aggregator? - CCN](https://www.ccn.com/education/crypto/defi-aggregator-decentralized-finance-tools/)
