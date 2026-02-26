# Stablecoins & DeFi Failures: A Comprehensive Research Guide

> **Audience**: Experienced Solana developer new to DeFi
> **Purpose**: Deep understanding of stablecoin mechanics, historical DeFi failures, and pattern recognition for avoiding catastrophic losses
> **Last Updated**: February 2026

---

## Table of Contents

- [Part 1: Stablecoins](#part-1-stablecoins)
  - [1. Types of Stablecoins](#1-types-of-stablecoins)
  - [2. Stability Mechanisms](#2-stability-mechanisms)
  - [3. Risks by Type](#3-risks-by-type)
- [Part 2: DeFi Failures](#part-2-defi-failures)
  - [4. Terra/Luna/UST Collapse (May 2022)](#4-terralunaust-collapse-may-2022---the-defining-failure)
  - [5. Other Major DeFi Failures & Exploits](#5-other-major-defi-failures--exploits)
  - [6. Common Patterns in DeFi Failures](#6-common-patterns-in-defi-failures)
  - [7. Warning Signs & Red Flags Checklist](#7-warning-signs--red-flags-checklist)
- [References](#references)

---

# Part 1: Stablecoins

## 1. Types of Stablecoins

Stablecoins are cryptocurrencies designed to maintain a stable value relative to a reference asset, typically the US dollar. They are the foundational primitive of DeFi -- virtually every lending protocol, DEX, and yield strategy depends on them. Understanding the different designs, their trade-offs, and their failure modes is essential before building anything in this space.

### 1.1 Fiat-Collateralized Stablecoins

**Examples**: USDC (Circle), USDT (Tether), BUSD (Paxos/Binance -- discontinued), PYUSD (PayPal)

**How They Work**:

1. A centralized issuer (e.g., Circle for USDC, Tether Ltd for USDT) holds reserves of fiat currency and fiat-equivalent assets (cash, US Treasuries, money market funds, commercial paper) in traditional bank accounts.
2. For every stablecoin token in circulation, the issuer claims to hold $1 (or equivalent) in reserves.
3. Authorized participants (institutional minters) can mint new tokens by depositing USD 1:1, and redeem tokens by burning them to receive USD back.
4. This mint/redeem mechanism creates a hard floor and ceiling around $1 for large participants, while smaller holders trade on secondary markets (DEXs, CEXs).

**Trust Assumptions**:

- **Custodial risk**: You trust the issuer actually holds the reserves they claim. Tether has been historically opaque about its reserves and was fined $41M by the CFTC in 2021 for misrepresenting them.
- **Banking risk**: Reserves sit in banks. If those banks fail, the peg can break. This happened to USDC in March 2023 when Circle had $3.3 billion stuck in the collapsing Silicon Valley Bank -- USDC depegged to $0.87.
- **Regulatory/censorship risk**: The issuer can freeze tokens on any address (Circle and Tether both have blacklist functionality in their smart contracts). They can be compelled by governments to do so.
- **Counterparty risk**: You depend on the issuer remaining solvent, honest, and operationally competent.

**Solana-Specific Notes**: USDC is natively issued on Solana via Circle's Cross-Chain Transfer Protocol (CCTP). USDT also has a native Solana issuance. These are SPL tokens, not bridged -- which eliminates bridge risk but retains all custodial/issuer risk.

### 1.2 Crypto-Overcollateralized Stablecoins

**Examples**: DAI (MakerDAO/Sky), LUSD (Liquity), sUSD (Synthetix), crvUSD (Curve)

**How They Work -- The CDP Model (MakerDAO/DAI)**:

1. A user deposits crypto collateral (ETH, wBTC, stETH, etc.) into a smart contract called a **Vault** (historically called a CDP -- Collateralized Debt Position).
2. The user can then borrow/mint DAI against that collateral, up to a maximum ratio. For example, with a 150% collateralization ratio, depositing $1,500 of ETH lets you mint up to 1,000 DAI.
3. The user pays an ongoing **stability fee** (interest rate) on their borrowed DAI.
4. If the collateral value drops and the Vault's collateralization ratio falls below the **liquidation threshold**, the Vault is **liquidated** -- the collateral is automatically auctioned off to repay the DAI debt, plus a **liquidation penalty** (typically 13%).

**Liquidation Mechanics (MakerDAO Liquidation 2.0)**:

- MakerDAO uses **Dutch auctions** for liquidation. When a Vault becomes undercollateralized, the `Dog` contract initiates an auction starting at a high price that declines over time.
- Liquidators (called "keepers") can buy the collateral when the price becomes attractive, paying DAI which is used to cover the outstanding debt.
- Unlike English auctions (used in Liquidation 1.0), Dutch auctions settle instantly -- no capital lockup waiting for outbids.
- The liquidation penalty (e.g., 13%) means the system recovers more DAI than the debt, with the excess going to the surplus buffer.

**Why Overcollateralization Works**:

- The collateral is *exogenous* -- ETH's value does not depend on DAI's success. Even if MakerDAO ceased to exist, ETH would retain value.
- The overcollateralization buffer absorbs price drops. A 150% ratio means ETH can drop ~33% before liquidation.
- Liquidation incentives attract keepers/bots who compete to liquidate undercollateralized positions, maintaining system health.

**Stability Mechanisms for DAI**:

- **Peg Stability Module (PSM)**: Allows direct 1:1 swaps between DAI and USDC (with minimal fees). This provides a hard floor/ceiling near $1 by enabling arbitrage.
- **DAI Savings Rate (DSR)**: When DAI trades below $1, MakerDAO can increase the DSR to incentivize holding DAI, reducing supply. When above $1, they can lower it.
- **Stability Fees**: Adjusting interest rates on Vaults influences supply -- higher fees discourage minting (reduce supply), lower fees encourage it (increase supply).

### 1.3 Algorithmic Stablecoins

**Examples**: UST (Terra -- collapsed), FRAX (partially algorithmic, now hybrid), AMPL (Ampleforth -- rebase model), ESD (Empty Set Dollar -- collapsed), BAC (Basis Cash -- collapsed)

**The Theory**:

Algorithmic stablecoins attempt to maintain a $1 peg *without* requiring full collateral backing. Instead, they use market incentives, arbitrage mechanisms, and supply expansion/contraction algorithms:

- **When price > $1**: The protocol mints new stablecoins, increasing supply to push the price down. The new tokens are distributed as rewards or sold.
- **When price < $1**: The protocol contracts supply by incentivizing users to burn stablecoins (often in exchange for a secondary token or "bonds" redeemable later).

**The Reality**:

The fundamental problem is what academics call the **endogenous collateral problem**. When the backing asset's value is derived from the same system it backs, you get a circular dependency:

1. The stablecoin's credibility depends on the governance/backing token having value.
2. The governance/backing token's value depends on the stablecoin being credible and widely used.
3. When confidence breaks, both collapse simultaneously in a **death spiral**.

As the Wake Forest Law Review put it: "Algorithmic stablecoins exist in a system that will be prone to runs, destabilization, and failure when reality deviates from the assumptions underlying the embedded incentive structure."

Every pure algorithmic stablecoin has either collapsed, abandoned its algorithmic model, or remained tiny and irrelevant. This is not a coincidence -- it is a structural inevitability.

### 1.4 Hybrid Approaches

**Examples**: FRAX (Frax Finance), GHO (Aave), UXD (Solana-native)

Hybrid stablecoins combine elements of different models:

- **FRAX**: Originally partially algorithmic (e.g., 90% USDC-collateralized, 10% algorithmic). After UST collapsed, Frax moved to 100% collateral ratio, effectively abandoning the algorithmic component. FRAX v3 aims to be backed entirely by real-world assets and crypto collateral.
- **GHO**: Aave's stablecoin is minted by Aave borrowers using their deposited collateral. It inherits Aave's overcollateralized lending model but adds GHO-specific stability mechanisms like interest rate adjustments by the DAO.
- **UXD Protocol** (Solana): Attempted to create a stablecoin backed by delta-neutral positions (combining spot assets with short perpetual futures to create a position insensitive to price). Novel but exposes users to funding rate risk and smart contract risk on the derivatives platform.

The trend across the industry post-2022 is clearly toward *more* collateral, not less. The algorithmic experiment has largely been abandoned by serious projects.

---

## 2. Stability Mechanisms

### 2.1 How Pegs Are Maintained

A stablecoin peg is maintained through a combination of:

**Primary Mechanism -- Redemption Rights**:
For fiat-collateralized stablecoins, the ultimate anchor is the ability to redeem 1 token for $1. This creates a credible price floor. If USDC trades at $0.99, authorized participants can buy it on the open market and redeem from Circle for $1.00, pocketing $0.01 per coin. This arbitrage pushes the price back up.

**Secondary Mechanism -- Market Making/Liquidity**:
Deep liquidity pools on DEXs (Curve, Orca, Raydium) and CEXs create resistance to depegging. A trade that would move the price of USDC by 1% on a pool with $10M liquidity would barely register on a pool with $1B.

**Tertiary Mechanism -- Psychological Confidence**:
If the market believes a stablecoin will hold its peg, it tends to. Confidence creates self-fulfilling stability. The reverse is also true -- loss of confidence creates self-fulfilling instability (bank runs).

### 2.2 Arbitrage Loops

**Fiat-Collateralized Arbitrage Loop**:
```
If USDC < $1.00:
  1. Buy USDC on open market at discount
  2. Redeem from Circle for $1.00
  3. Profit = ($1.00 - market_price) per USDC
  4. Buying pressure pushes price back to $1.00

If USDC > $1.00:
  1. Deposit USD with Circle, mint USDC at $1.00
  2. Sell USDC on open market at premium
  3. Profit = (market_price - $1.00) per USDC
  4. Selling pressure pushes price back to $1.00
```

**Crypto-Collateralized Arbitrage Loop (DAI/PSM)**:
```
If DAI < $1.00:
  1. Buy DAI on open market at discount
  2. Use PSM to swap DAI -> USDC at 1:1
  3. Sell USDC for $1.00
  4. Profit = ($1.00 - DAI_market_price)

If DAI > $1.00:
  1. Buy USDC for $1.00
  2. Use PSM to swap USDC -> DAI at 1:1
  3. Sell DAI on open market at premium
  4. Profit = (DAI_market_price - $1.00)
```

**Algorithmic Arbitrage Loop (Terra/UST -- now defunct)**:
```
If UST < $1.00:
  1. Buy UST on open market at discount (e.g., $0.95)
  2. Burn UST via protocol, receive $1.00 worth of newly minted LUNA
  3. Sell LUNA on open market
  4. Profit = ($1.00 - $0.95) = $0.05 per UST

If UST > $1.00:
  1. Burn $1.00 worth of LUNA via protocol, receive 1 UST
  2. Sell UST on open market at premium
  3. Profit = (UST_market_price - $1.00)
```

The critical difference: in the fiat and PSM cases, the arbitrage is backed by *real dollars* or *real USDC*. In the algorithmic case, you are receiving newly minted LUNA -- a token whose value depends entirely on confidence in the system. When confidence breaks, the LUNA you receive is worth less and less, destroying the arbitrage incentive precisely when it is needed most.

### 2.3 Peg Stability Modules

A Peg Stability Module (PSM) is a mechanism that allows direct, fixed-rate swaps between a stablecoin and another stable asset:

- **MakerDAO PSM**: Allows swapping DAI <-> USDC at 1:1 (minus a small fee of 0-0.1%). This effectively makes DAI partially backed by USDC, providing a hard peg but introducing USDC custodial risk.
- **Curve StableSwap**: Not a PSM per se, but Curve's specialized AMM for stable-to-stable swaps uses a concentrated liquidity curve that keeps prices very close to 1:1, with minimal slippage for normal-sized trades. Curve's 3pool (DAI/USDC/USDT) historically served as a major stability anchor.

---

## 3. Risks by Type

### 3.1 Fiat-Collateralized Risks

| Risk | Description | Real-World Example |
|------|-------------|-------------------|
| **Custodial/Reserve Risk** | Issuer may not hold adequate reserves | Tether's murky reserve history; CFTC $41M fine |
| **Banking Risk** | Reserves held in banks that can fail | USDC depeg to $0.87 during SVB collapse (March 2023) |
| **Censorship/Blacklisting** | Issuer can freeze any address | Circle/Tether have frozen addresses under government order |
| **Regulatory Risk** | Government can shut down the issuer | BUSD discontinued after SEC action against Paxos |
| **Single-Issuer Risk** | Centralized point of failure | If Circle goes bankrupt, USDC redemptions stop |
| **Contagion Risk** | Problems spread between stablecoins via PSMs | DAI depegged during SVB crisis because it held USDC |

### 3.2 Crypto-Overcollateralized Risks

| Risk | Description | Real-World Example |
|------|-------------|-------------------|
| **Smart Contract Risk** | Bugs in the protocol code | MakerDAO "Black Thursday" (March 2020) -- liquidation bots failed, $8.3M in undercollateralized debt |
| **Oracle Risk** | Price feeds can be manipulated or fail | If ETH oracle reports wrong price, liquidations fire incorrectly |
| **Collateral Volatility** | Rapid price drops can outpace liquidations | Flash crashes can create bad debt before liquidators act |
| **Governance Risk** | Token holders make bad risk parameter decisions | Onboarding risky collateral types to chase yield |
| **Scalability** | Overcollateralization limits capital efficiency | Need $1.50 locked up to create $1 of stablecoin |
| **Dependency on other stablecoins** | PSMs create exposure to fiat stablecoins | DAI effectively partially backed by USDC |

### 3.3 Algorithmic Stablecoin Risks

| Risk | Description | Real-World Example |
|------|-------------|-------------------|
| **Death Spiral** | Reflexive collapse of both stablecoin and backing token | Terra/UST collapse -- $45B wiped out in a week |
| **Endogenous Collateral** | Backing token's value depends on system confidence | LUNA's value depended on UST demand |
| **Bank Run Dynamics** | Rational to exit first when confidence drops | Anchor withdrawals accelerated the UST depeg |
| **No Hard Floor** | No real assets backing redemptions | When LUNA minting accelerated, supply went from 350M to 6.5T tokens |
| **Untested in Crisis** | Algorithms designed for normal conditions fail in extremes | Iron Finance TITAN collapsed despite "working as designed" |
| **Regulatory Target** | Post-Terra, regulators specifically target algo stablecoins | Multiple jurisdictions have banned or restricted them |

---

# Part 2: DeFi Failures

> "Those who cannot remember the past are condemned to repeat it." -- George Santayana
>
> In DeFi, those who cannot analyze past failures will build the next one.

---

## 4. Terra/Luna/UST Collapse (May 2022) -- THE DEFINING FAILURE

This is the single most important event in DeFi history. The Terra/Luna collapse destroyed approximately $45 billion in market capitalization within a week, triggered a crypto-wide cascade that took down Three Arrows Capital, Celsius, Voyager, BlockFi, and contributed to the conditions that led to FTX's collapse. Terraform Labs founder Do Kwon was sentenced to 15 years in prison in December 2025 for orchestrating what the DOJ called a "$40 billion fraud."

### 4.1 How UST Worked -- The Mint/Burn Mechanism

**The Core Mechanism**:

Terra's UST stablecoin was designed around a simple swap:

- **To mint 1 UST**: Burn $1 worth of LUNA tokens (at current market price).
- **To redeem 1 UST**: Burn 1 UST, receive $1 worth of newly minted LUNA tokens.

Example when UST = $1.00 and LUNA = $80.00:
```
Mint: Burn 0.0125 LUNA ($1.00 worth) -> Receive 1 UST
Redeem: Burn 1 UST -> Receive 0.0125 LUNA ($1.00 worth)
```

**The Peg Maintenance Theory**:

- If UST drops below $1.00 (e.g., $0.95): Arbitrageurs buy cheap UST, burn it for $1.00 worth of LUNA, sell LUNA for profit. Buying pressure on UST pushes it back to $1.00.
- If UST rises above $1.00 (e.g., $1.05): Arbitrageurs burn LUNA to mint UST at $1.00 value, sell UST for $1.05. Selling pressure on UST pushes it back down.

**The Fatal Flaw**:

The value backing UST was *LUNA* -- a token whose value derived entirely from the Terra ecosystem. LUNA's market cap needed to exceed UST's supply for redemptions to be credible. But LUNA's value depended on people believing UST was stable, creating a circular dependency.

When UST started depegging:
1. People burned UST for LUNA (increasing LUNA supply).
2. They immediately sold the LUNA they received (crashing LUNA's price).
3. LUNA's price drop meant more LUNA had to be minted per UST redeemed.
4. More LUNA minting meant even more selling pressure.
5. This created **hyperinflation** of LUNA -- supply went from ~350 million to 6.5 TRILLION tokens.

### 4.2 Anchor Protocol and the 20% APY Promise

**What Was Anchor**:

Anchor Protocol was a lending/borrowing platform on Terra that offered ~19.5% APY on UST deposits. It was the primary reason people held UST -- at its peak, **72% of all UST in existence was deposited in Anchor** (~$14 billion).

**Where the Yield "Came From"**:

Anchor generated income from three sources:
1. **Interest from borrowers**: Borrowers paid interest on UST loans.
2. **Staking rewards from collateral**: Borrowers deposited collateral (bLUNA, bETH) which earned staking yields that Anchor captured.
3. **Liquidation fees**: When borrower positions were liquidated, Anchor collected penalty fees.

**Why It Was Unsustainable**:

The math never worked:
- Anchor had ~$14B in deposits requiring ~$2.7B/year in interest payments at 19.5% APY.
- Borrowing demand was only ~$3B, generating far less income.
- The protocol was **losing approximately $4 million per day** from its yield reserve.

The structural problem was that in bear markets:
- Fewer people wanted to borrow (less income for Anchor).
- More people wanted to deposit for the "safe" 19.5% yield (more expenses).
- The gap widened precisely when the system was most stressed.

**The Yield Reserve Depletion**:

| Date | Yield Reserve | Event |
|------|--------------|-------|
| **Late Jan 2022** | ~$35M UST | Reserve declining ~$1.25M/day |
| **Early Feb 2022** | ~$6.56M UST | Near depletion |
| **Feb 18, 2022** | Refilled to ~$460M | LFG emergency injection of 450M UST |
| **March 2022** | Declining again | Proposal 20 passed: rate to decrease 1.5%/month |
| **April 2022** | Declining at ~$4M/day | Projected depletion by June 2022 |
| **May 7, 2022** | N/A | Collapse began before reserve exhaustion |

The yield reserve was essentially a subsidy pool -- burning through hundreds of millions to maintain the fiction of sustainable 20% yields.

### 4.3 The Luna Foundation Guard Bitcoin Reserve

In January 2022, the Luna Foundation Guard (LFG) was established with Do Kwon as director. Its purpose was to accumulate a Bitcoin reserve to defend the UST peg -- an implicit acknowledgment that the algorithmic mechanism alone was insufficient.

- **LFG accumulated 80,394 BTC** (worth ~$3.5 billion at peak).
- The reserve also held other assets (AVAX, LUNA, UST).
- The idea was to sell BTC to buy UST if the peg came under pressure.

**The Reserve's Failure**: Between May 8-10, 2022, LFG liquidated nearly its entire Bitcoin reserve -- going from 80,394 BTC to just 313 BTC -- in a failed attempt to defend the peg. The $2.4 billion in Bitcoin selling added downward pressure to BTC (which fell from ~$35K to ~$26K during the crisis), spreading contagion to the entire crypto market.

### 4.4 The Death Spiral -- Day by Day

**Pre-Collapse Context**:
- LUNA hit an all-time high of $119.18 on April 5, 2022.
- UST had a market cap of ~$18.7 billion.
- Anchor held ~$14 billion in UST deposits.
- The broader crypto market was already declining from November 2021 highs.

**May 7, 2022 (Saturday) -- The First Depeg**:
- Two large addresses withdrew **375 million UST** from Anchor.
- A whale watching bot revealed 85 million UST were swapped for USDC.
- A further ~$2 billion was withdrawn from Anchor.
- UST dropped from $1.00 to $0.985.
- LFG began deploying Bitcoin reserves to defend the peg.

**May 8, 2022 (Sunday) -- Temporary Recovery Attempt**:
- LFG deployed $750 million in BTC to market makers to buy UST.
- UST briefly recovered toward $0.99-$1.00.
- But selling pressure continued as more users fled Anchor.
- LUNA dropped below $60.

**May 9, 2022 (Monday) -- Anchor Bank Run**:
- **5 billion UST** (35% of Anchor's total deposits) withdrawn in a single day.
- UST fell to $0.68 at its low point.
- LUNA crashed below $30.
- LFG continued dumping Bitcoin reserves. Market-wide crypto selloff accelerated.
- Do Kwon tweeted "Deploying more capital - steady lads."

**May 10, 2022 (Tuesday) -- Failed Recovery**:
- UST temporarily recovered to $0.93 on LFG buying.
- But recovery was short-lived as selling resumed.
- UST closed the day around $0.67.
- LUNA fell to ~$15.
- LFG had now deployed most of its Bitcoin reserve.

**May 11, 2022 (Wednesday) -- Acceleration**:
- By this date, over **11 billion UST** had been withdrawn from Anchor.
- UST fell to $0.30.
- LUNA crashed below $1.00.
- LUNA supply began hyperinflating as UST holders mass-redeemed.
- Do Kwon proposed increasing LUNA minting capacity to accelerate UST burns. This was like throwing gasoline on the fire.

**May 12, 2022 (Thursday) -- Hyperinflation**:
- LUNA fell 96% in a single day to less than $0.10.
- UST hit $0.16.
- Many exchanges halted LUNA and UST trading.
- LUNA supply exploded from ~350 million to billions of tokens.
- The mint/burn mechanism was creating LUNA faster than anyone could sell it.

**May 13, 2022 (Friday) -- Blockchain Halted**:
- Terraform Labs halted the Terra blockchain twice.
- LUNA was trading at fractions of a cent -- effectively worthless.
- UST stabilized around $0.10-$0.15, then declined to ~$0.02.
- LUNA supply had reached **6.5 trillion tokens** (from 350 million a week earlier).

**May 25, 2022 -- Fork Approved**:
- Community voted to create "Terra 2.0" -- a new LUNA token without UST.
- Original LUNA renamed to LUNA Classic (LUNC).
- Original UST renamed to USTC.
- Both remain near-worthless.

### 4.5 The Aftermath -- By the Numbers

| Metric | Before Collapse | After Collapse |
|--------|----------------|----------------|
| **LUNA Price** | $119.18 (ATH, April 5) | < $0.0001 |
| **UST Price** | $1.00 | ~$0.02 |
| **UST Market Cap** | ~$18.7 billion | ~$300 million |
| **LUNA Market Cap** | ~$40 billion (ATH) | ~$0 |
| **Anchor TVL** | ~$17 billion | $0 |
| **LFG Bitcoin Reserve** | 80,394 BTC (~$3.5B) | 313 BTC |
| **LUNA Token Supply** | ~350 million | 6.5 trillion |
| **Estimated Total Losses** | | ~$45 billion direct; ~$400B+ broader market |
| **Estimated Victims** | | ~1 million (judge's estimate at sentencing) |

### 4.6 The Legal Aftermath

- **Do Kwon** fled to Montenegro after collapse, was arrested in March 2023 using a fake Costa Rican passport, extradited to the US in December 2024, pleaded guilty in August 2025, and was **sentenced to 15 years in prison** in December 2025.
- The DOJ revealed that Kwon had **secretly arranged for a trading firm to buy UST to restore the peg** during an earlier depeg in May 2021 -- not the algorithm. He then lied to investors claiming the algorithm had worked.
- **Terraform Labs** reached a $4.47 billion settlement with the SEC in June 2024.
- The collapse prompted regulatory action worldwide -- the EU's MiCA regulation explicitly restricts algorithmic stablecoins.

### 4.7 Why Algorithmic Stablecoins with Endogenous Collateral Are Fundamentally Fragile

The Terra collapse was not a black swan. It was a structural inevitability. Here is why:

**1. Circular Value Creation**:
LUNA's value came from UST demand. UST demand came from Anchor yields. Anchor yields came from subsidies funded by LUNA sales. There was no external value anchor -- just tokens referencing each other.

**2. Asymmetric Incentives in Crisis**:
The arbitrage mechanism (burn UST, receive LUNA, sell LUNA) works in calm markets. In a crisis, every arbitrageur selling LUNA pushes the price down further, requiring more LUNA to be minted per UST, accelerating the spiral. The mechanism that was supposed to restore the peg actively destroyed it.

**3. No Lender of Last Resort**:
Traditional currencies have central banks. Fiat-backed stablecoins have real reserves. Crypto-overcollateralized stablecoins have locked collateral. Algorithmic stablecoins have... an algorithm and confidence. When confidence breaks, there is nothing.

**4. Bank Run Dynamics**:
Everyone knows it is rational to exit before everyone else. This creates a "run" dynamic identical to a bank run, but without deposit insurance or circuit breakers. The first to sell lose the least.

**5. Scale Creates Fragility**:
Paradoxically, as UST grew, it became MORE fragile:
- More UST meant more potential redemptions.
- More redemptions meant more LUNA minting.
- LUNA's market cap could not scale proportionally because its value was reflexively tied to the system.

**The Lesson**: If someone shows you a stablecoin backed primarily by a token created within its own ecosystem, you are looking at a system that *will* eventually collapse. The only question is when. Exogenous collateral (assets whose value is independent of the stablecoin system) is the only proven foundation for a stable peg.

---

## 5. Other Major DeFi Failures & Exploits

### 5.1 Iron Finance / TITAN Token Collapse (June 2021)

**What It Was**:
Iron Finance was a protocol on Polygon (and BSC) that created a partially-collateralized stablecoin called IRON, backed by a mix of USDC (75%) and TITAN token (25%).

**The Mechanics**:
- IRON was redeemable for $0.75 USDC + $0.25 worth of TITAN.
- To mint IRON, users deposited $0.75 USDC + $0.25 worth of TITAN.
- TITAN was the protocol's own token -- endogenous collateral for the remaining 25%.

**The Collapse (June 16, 2021)**:
1. TITAN had risen 600% in the week prior (partly due to Mark Cuban promoting it on his blog).
2. Large holders ("whales") began taking profits, selling TITAN at ~$65.
3. TITAN selling caused IRON to drop slightly below $1.00.
4. Arbitrageurs redeemed IRON for $0.75 USDC + $0.25 TITAN, then sold the TITAN.
5. This created more TITAN selling pressure, pushing IRON further below peg.
6. More IRON redemptions -> more TITAN sold -> lower TITAN price -> more IRON depegs.
7. Classic death spiral activated.

**The Result**:
- TITAN went from $65 to **$0.000000035** (effectively zero) in less than 24 hours.
- IRON settled around $0.69 (backed only by its USDC component minus chaos).
- Approximately **$2 billion in value** was destroyed.
- Iron Finance called it "the first large-scale bank run" in DeFi history (Terra would dramatically claim that title a year later).

**Key Lesson**: Even *partial* reliance on endogenous collateral creates death spiral risk. The 25% TITAN component was enough to bring the entire system down. The Federal Reserve published a research note analyzing Iron Finance as evidence that algorithmic stablecoins are inherently fragile.

---

### 5.2 Olympus DAO and the (3,3) Meme

**What It Was**:
Olympus DAO launched in early 2021 as a "decentralized reserve currency" protocol. It operated on a novel "protocol-owned liquidity" model with a treasury-backed token, OHM.

**The (3,3) Game Theory**:
The name comes from a simple payoff matrix:
- **Stake** (+3): Lock OHM for rebasing rewards (the "cooperative" strategy).
- **Bond** (+1): Sell assets to the treasury at a discount for OHM (medium benefit).
- **Sell** (-1): Sell OHM on the open market (the "defecting" strategy).

The (3,3) meme meant: "If everyone stakes, everyone wins the most." Users added "(3,3)" to their Twitter names as a signal of commitment.

**How It Worked**:
1. Users bought OHM or acquired it through bonding (selling LP tokens or stablecoins to the treasury at a discount).
2. Staked OHM received rebasing rewards -- new OHM minted every 8 hours.
3. Displayed APYs were astronomical: **7,000% to 100,000%+** at peak.
4. These APYs were possible because they were paid in OHM tokens, which were continuously minted.

**Why It Collapsed**:
- The high APY was nominal, paid in a constantly diluting token. If OHM price dropped 90%, a 10,000% APY still resulted in losses.
- The system required *continuous new capital inflow* to sustain OHM's price against constant dilution from rebasing.
- In January 2022, an Olympus team member dumped $11 million worth of OHM. The price crashed 40% in two hours, wiping $600 million in market cap.
- OHM's price fell from an all-time high of **$1,415 to under $30** -- a decline of over 97%.
- The (3,3) game theory broke down because it only works if participants believe others will also stake. Once selling started, the rational move was to sell first (the classic prisoner's dilemma).

**The Fork Phenomenon**:
Olympus spawned dozens of forks (Wonderland/TIME, Klima DAO, Snowdog, etc.), each promising even higher APYs. Almost all collapsed even harder than OHM. The fork phenomenon demonstrated how easily the model's mechanics could be replicated without the (limited) treasury backing that OHM had.

**Key Lesson**: Extremely high APYs paid in the protocol's own token are a red flag. The yield is not "real" -- it is dilution disguised as returns. The game theory only works in an infinitely growing market, which does not exist.

---

### 5.3 Wonderland (TIME Token) / 0xSifu Scandal

**What It Was**:
Wonderland was an Olympus DAO fork on Avalanche, founded by Daniele Sestagalli (a prominent DeFi figure who also created Abracadabra/MIM). It used the same rebase mechanics with its TIME token.

**The Scandal (January 27, 2022)**:
- Crypto investigator @zachxbt revealed that Wonderland's CFO, known only as "0xSifu," was actually **Michael Patryn** -- co-founder of **QuadrigaCX**, the Canadian exchange that collapsed in 2019 with $190 million of customer funds. QuadrigaCX was officially labeled a "fraud" and "Ponzi scheme" by the Ontario Securities Commission.
- Patryn had also previously pled guilty to credit/bank fraud (2005) and admitted to burglary, theft, and computer fraud (2007).
- Sestagalli admitted he had known about Patryn's identity for a month but did not disclose it because he "believed in giving second chances."

**The Fallout**:
- TIME tokens fell over 60% following the revelation.
- Wonderland's TVL plunged from over $1 billion to ~$146 million.
- A community vote was held on whether to shut down Wonderland -- the majority voted to keep it going, but Sestagalli unilaterally announced closure anyway.
- The scandal caused cascading liquidations across DeFi, particularly in the Abracadabra/MIM ecosystem (also created by Sestagalli), since users had borrowed MIM against TIME tokens as collateral.

**Key Lesson**: Anonymous teams are a massive risk. When billions of dollars are managed by pseudonymous individuals, there is no accountability. The DeFi ethos of "code is law" breaks down when humans control treasuries. Always investigate who controls the money.

---

### 5.4 Celsius Network Collapse (June-July 2022)

**What It Was**:
Celsius was a centralized crypto lending platform (CeFi, not DeFi, but deeply intertwined with DeFi) that offered high yields to depositors (up to 17% APY on some assets) and lent funds to institutional borrowers and DeFi protocols.

**The Business Model**:
- Accept customer crypto deposits.
- Pay depositors high interest rates.
- Generate yield by: lending to institutions, deploying funds into DeFi protocols, running validator nodes, and making leveraged bets.
- The spread between what they earned and what they paid was supposed to be their profit.

**Why It Failed**:

1. **Chasing unsustainable yields**: To pay the promised high rates, Celsius invested customer funds in increasingly risky DeFi strategies. Former employees reported management pressured teams to find higher yields regardless of risk.

2. **Terra/LUNA exposure**: Celsius had exposure to the Terra ecosystem. When UST collapsed in May 2022, it directly impacted their portfolio.

3. **stETH illiquidity**: Celsius held large positions in Lido's stETH (staked ETH). Before Ethereum's Shanghai upgrade enabled withdrawals, stETH was only tradeable on secondary markets. When stETH traded at a discount to ETH, Celsius faced a liquidity crunch.

4. **Three Arrows Capital contagion**: Celsius had counterparty exposure to 3AC, which collapsed in June 2022.

**Timeline**:
- **May 2022**: Terra collapse hits Celsius portfolio.
- **June 12, 2022**: Celsius freezes all withdrawals, swaps, and transfers citing "extreme market conditions."
- **June-July 2022**: Users unable to access their funds.
- **July 13, 2022**: Celsius files Chapter 11 bankruptcy.
- Balance sheet revealed a **$1.2 billion hole** -- they owed users ~$4.7 billion but did not have the assets to cover it.
- Estimated recovery: 39.4% to 100% depending on account type, with most users receiving far less than their full deposits.

**Key Lesson**: "Not your keys, not your coins" is not just a meme -- it is a survival strategy. Centralized platforms that promise high yields are taking risks you cannot see or control. The opacity of CeFi combined with DeFi's volatility is a deadly combination.

---

### 5.5 FTX/Alameda Research and DeFi Contagion (November 2022)

**What It Was**:
FTX was the world's third-largest crypto exchange. Alameda Research was a crypto trading firm. Both were founded and controlled by Sam Bankman-Fried (SBF). Despite being presented as separate entities, they were deeply intertwined.

**The Core Fraud**:
- FTX secretly funneled approximately **$8 billion in customer deposits** to Alameda Research.
- Alameda used these funds for trading, venture investments, political donations, and real estate purchases.
- FTX's balance sheet was propped up by FTT (FTX's own exchange token) -- a classic case of circular, self-referential value.

**The Collapse (November 2-11, 2022)**:
1. **Nov 2**: CoinDesk published a leaked balance sheet showing Alameda's assets were dominated by FTT tokens.
2. **Nov 6**: Binance CEO CZ announced Binance would liquidate its ~$500M FTT holdings.
3. **Nov 7-8**: FTX experienced **$5 billion in withdrawal requests** within 72 hours. FTT crashed 80%.
4. **Nov 8**: Binance announced (then withdrew) a potential acquisition of FTX.
5. **Nov 11**: FTX, Alameda, and **130+ affiliated entities** filed for bankruptcy.

**DeFi Contagion**:
- **Solana ecosystem** was heavily impacted -- FTX/Alameda were major Solana investors and market makers. SOL dropped from ~$35 to ~$12.
- **BlockFi** filed bankruptcy on November 28, 2022, citing "significant exposure to FTX."
- **Genesis** (crypto lender) halted withdrawals and later filed bankruptcy.
- Numerous DeFi protocols that had counterparty exposure to Alameda were affected.
- The Solana DeFi ecosystem lost significant TVL and liquidity.

**Key Lesson**: Concentration risk is existential risk. The Solana ecosystem's heavy dependence on FTX/Alameda nearly killed it. Also: CeFi entities that interact with DeFi create opaque risk channels. The FTT token being used as collateral across DeFi created hidden exposure that was invisible until the collapse.

---

### 5.6 Mango Markets Exploit (October 2022) -- Solana-Specific

**What It Was**:
Mango Markets was a decentralized trading platform on Solana offering spot and perpetual futures trading with cross-margin capability.

**The Exploit (October 11, 2022)**:

Avraham Eisenberg executed a market manipulation attack in ~10 minutes:

1. **Set up**: Opened two accounts on Mango Markets with ~$5 million total.
2. **Position**: Used one account to take a massive long position in MNGO perpetual futures.
3. **Pump**: Simultaneously bought ~$4 million of MNGO tokens across three exchanges, pumping the oracle-reported price by **2,300%**.
4. **Extract**: The inflated MNGO perps position showed enormous unrealized profits. Eisenberg used this paper profit as collateral to **borrow $116 million** in various tokens from Mango Markets' lending pools.
5. **Exit**: Withdrew the borrowed funds. The inflated MNGO price eventually corrected, leaving the protocol with massive bad debt.

**Total Loss**: ~$116 million drained from the protocol.

**Aftermath**:
- Eisenberg publicly claimed it was a "highly profitable trading strategy" -- not an exploit.
- He returned $67 million to the Mango DAO after "negotiating" a settlement.
- He was arrested in December 2022 and convicted of commodities fraud, commodities manipulation, and wire fraud in April 2024.
- In a surprising turn, a federal judge vacated all criminal convictions in May 2025.
- Mango Markets wound down operations after an SEC settlement.

**Key Lesson for Solana Developers**:
- **Oracle manipulation** is a critical attack vector, especially on low-liquidity tokens.
- Cross-margin systems that allow borrowed funds based on unrealized PnL are vulnerable.
- Thin order books on Solana DEXs make price manipulation easier than on Ethereum.
- Any protocol that relies on the price of a low-cap token for collateral decisions is at risk.

---

### 5.7 Wormhole Bridge Hack (February 2022) -- Solana-Specific

**What It Was**:
Wormhole was (and remains, now rebuilt) a cross-chain bridge connecting Solana, Ethereum, and other blockchains.

**The Exploit (February 2, 2022)**:

1. The attacker found a vulnerability in Wormhole's Solana-side smart contract.
2. The core issue: a **deprecated, insecure function** (`verify_signatures`) was used for signature verification. The attacker could inject a **fake system account (sysvar)** that bypassed the verification check.
3. The attacker crafted a malicious message claiming 120,000 ETH had been deposited on Ethereum (it had not).
4. Using the forged verification, the attacker called the `complete_wrapped` function to mint **120,000 wETH** on Solana.
5. The attacker bridged 10,000 wETH back to Ethereum (receiving real ETH) and kept the rest on Solana.

**Total Loss**: **$326 million** (120,000 wETH at the time).

**Resolution**: Jump Trading (Wormhole's parent company and a major Solana ecosystem investor) covered the loss, depositing 120,000 ETH to make Wormhole whole. This was one of the largest single entity bailouts in crypto history.

**Key Lesson for Solana Developers**:
- Bridge contracts are among the highest-value targets in all of crypto.
- Using deprecated functions is a critical vulnerability. Always audit dependencies.
- Sysvar/account injection attacks are a Solana-specific concern -- the account model requires careful validation of every account passed to an instruction.
- Bridge security depends on the weakest link in any connected chain.

---

### 5.8 Ronin Bridge Hack (March 2022)

**What It Was**:
Ronin was an Ethereum sidechain built specifically for Axie Infinity, the play-to-earn NFT game. The Ronin Bridge connected it to Ethereum mainnet.

**The Exploit (March 23, 2022)**:

1. Ronin used a **validator-based bridge** with 9 validator nodes. A transaction required 5-of-9 signatures.
2. The attackers (identified as North Korea's **Lazarus Group** by the FBI) compromised 4 validator private keys belonging to Sky Mavis (Axie's developer) through **social engineering** -- reportedly via a fake job offer targeting an engineer.
3. They also gained access to a 5th key belonging to Axie DAO, which had temporarily granted Sky Mavis signing authority months earlier (and the permission was never revoked).
4. With 5-of-9 keys, the attackers forged withdrawal transactions for **173,600 ETH and 25.5 million USDC**.

**Total Loss**: **$625 million** -- the largest DeFi hack in history at the time.

**Detection Failure**: The hack went **unnoticed for 6 days** until a user reported being unable to withdraw 5,000 ETH from the bridge.

**Key Lessons**:
- **Multisig is only as secure as key management**. A 5-of-9 multisig is meaningless if one entity controls 5+ keys.
- **Social engineering** remains the #1 attack vector, even in cutting-edge crypto.
- **Nation-state attackers** (Lazarus Group) are active in DeFi and represent a threat level most protocols are not designed for.
- **Permission hygiene**: Temporary access grants must be revoked. The Axie DAO signing authority was never removed.
- **Monitoring**: A $625M theft going unnoticed for 6 days indicates catastrophic monitoring failures.

---

### 5.9 Beanstalk Governance Attack (April 2022)

**What It Was**:
Beanstalk was a decentralized algorithmic stablecoin protocol on Ethereum with a stablecoin called BEAN.

**The Exploit (April 17, 2022)**:

1. Beanstalk's governance had an `emergencyCommit` function that could execute proposals immediately if they received 2/3 of votes.
2. The attacker took out **$1 billion in flash loans** from Aave (in DAI, USDC, USDT).
3. Used these funds to acquire enough Stalk (Beanstalk's governance token) to control **67% of voting power**.
4. Submitted and immediately passed a malicious governance proposal via `emergencyCommit`.
5. The proposal used `delegatecall` to transfer **all protocol funds** to the attacker's wallet.
6. Repaid flash loans in the same transaction.

**Total Loss**: $182 million drained; attacker's profit was ~$80 million (after flash loan costs).

**Key Lesson**: Governance systems that allow instant execution are vulnerable to flash loan attacks. Any governance mechanism must include time locks (delay between proposal approval and execution) to prevent this class of attack. This is non-negotiable for any protocol managing significant value.

---

## 6. Common Patterns in DeFi Failures

After studying dozens of DeFi failures, clear patterns emerge. Understanding these patterns is more valuable than memorizing individual incidents.

### 6.1 Unsustainable Yield Promises

**The Pattern**: Protocols offer yields far above market rates to attract deposits, funded by:
- Printing governance tokens (dilution disguised as yield)
- Subsidies from a finite reserve (see: Anchor)
- New investor capital paying old investor returns (Ponzi dynamics)
- Risk-taking that is invisible to depositors (see: Celsius)

**The Test**: Ask "Where does the yield come from?" If the answer involves the protocol's own token, new deposits, or handwaving about "DeFi composability," the yield is likely unsustainable.

**Real yield** comes from:
- Lending interest (borrowers paying to use capital)
- Trading fees (providing liquidity that traders pay to use)
- Staking rewards (securing a blockchain)
- MEV capture (extracting value from transaction ordering)
- Real-world asset yields (US Treasury rates, etc.)

If the yield significantly exceeds these sources, the difference is being funded by unsustainable means.

### 6.2 Reflexive/Circular Value Creation

**The Pattern**: Token A's value depends on Protocol B, which depends on Token A.

**Examples**:
- UST value -> Anchor demand -> LUNA value -> UST backing (Terra)
- OHM price -> Treasury growth -> Rebasing rewards -> OHM demand (Olympus)
- FTT price -> FTX collateral -> Alameda solvency -> FTT market making (FTX)
- TITAN price -> IRON stability -> TITAN utility -> TITAN price (Iron Finance)

**The Test**: Draw the value dependency graph. If it forms a circle with no external input of real value, the system is reflexive and will eventually collapse.

### 6.3 Excessive Leverage and Rehypothecation

**The Pattern**: The same collateral is used multiple times across different protocols, creating hidden leverage:

```
User deposits 100 ETH into Aave
  -> Borrows 70 DAI
  -> Deposits DAI into another protocol
  -> Borrows against that position
  -> Repeats...
```

**The Danger**: A 10% ETH price drop might trigger liquidations that cascade across 5 protocols simultaneously. The effective leverage might be 10x or more, invisible to any individual protocol.

**Real-World Example**: Three Arrows Capital (3AC) was a $10 billion crypto hedge fund that borrowed from nearly every major lender (Celsius, Voyager, BlockFi, Genesis) simultaneously. When their leveraged LUNA position collapsed, they could not meet margin calls from any lender, creating a cascade that bankrupted multiple companies.

### 6.4 Oracle Manipulation

**The Pattern**: Protocols rely on price oracles (Pyth, Chainlink, on-chain TWAPs) to make liquidation, borrowing, and trading decisions. If an attacker can manipulate the oracle price, they can:
- Borrow more than collateral is worth.
- Trigger/prevent liquidations at will.
- Create artificial arbitrage opportunities.

**Attack Vectors**:
- **Flash loan attacks**: Borrow massive amounts, trade on a low-liquidity DEX to move the price, exploit the manipulated price, repay the loan -- all in one transaction.
- **Low-liquidity token manipulation**: Move the price of a thinly-traded token by trading a relatively small amount (see: Mango Markets).
- **TWAP manipulation**: With enough capital, sustained trading over the TWAP period can move time-weighted prices.

**Mitigation**: Use decentralized oracle networks (Pyth on Solana, Chainlink on EVM), multiple price sources, circuit breakers, and never rely on a single DEX pool as a price source.

### 6.5 Bridge Vulnerabilities

**The Pattern**: Cross-chain bridges are among the highest-value targets because they custody large amounts of assets from multiple chains.

**Why Bridges Are Uniquely Vulnerable**:
- **Massive honeypots**: Bridges hold the collateral backing all wrapped tokens. The Ronin bridge held $625M+.
- **Cross-chain complexity**: Verifying events on one chain from another is fundamentally hard.
- **Validator centralization**: Many bridges use small multisig committees (Ronin: 9 validators).
- **Smart contract surface area**: Bridge contracts on both chains must be correct, plus the relayer/validator layer.

**Historical Bridge Losses**: Over $2.8 billion stolen from bridges, representing ~40% of all value hacked in Web3.

**Major Bridge Hacks**:

| Bridge | Date | Loss | Attack Vector |
|--------|------|------|--------------|
| Ronin | Mar 2022 | $625M | Validator key compromise (social engineering) |
| Wormhole | Feb 2022 | $326M | Signature verification bypass |
| Nomad | Aug 2022 | $190M | Initialization bug (trusted root set to 0x00) |
| Harmony Horizon | Jun 2022 | $100M | 2-of-5 multisig key compromise |
| BNB Bridge | Oct 2022 | $586M | Proof verification exploit |

### 6.6 Governance Attacks

**The Pattern**: Exploiting on-chain governance mechanisms to pass malicious proposals.

**Vectors**:
- **Flash loan governance**: Borrow tokens to gain voting power temporarily (Beanstalk).
- **Low quorum attacks**: When participation is low, a small holder can pass proposals.
- **Malicious proposals disguised as upgrades**: Proposals that include hidden fund-draining logic.

**Mitigations**:
- Time locks between proposal and execution (minimum 24-48 hours).
- Snapshot voting power (based on holdings at a past block, not current -- prevents flash loan attacks).
- Guardians/multisigs with veto power for emergency security.
- Code audits of all governance proposals before execution.

### 6.7 Smart Contract Bugs

**The Pattern**: Code bugs that allow unintended behavior -- the most "pure" form of DeFi exploit.

**Common Bug Classes**:
- **Reentrancy**: A contract calls an external contract before updating its own state, allowing the external contract to call back in recursively. (Classic: The DAO hack, 2016, $60M).
- **Integer overflow/underflow**: Arithmetic errors (less common post-Solidity 0.8 / Rust's built-in checks, but still possible with unchecked math).
- **Access control failures**: Missing authorization checks allowing anyone to call privileged functions.
- **Logic errors**: Incorrect business logic (e.g., calculating rewards incorrectly, allowing withdrawal of more than deposited).
- **Initialization bugs**: Contracts deployed with incorrect initial state (Nomad: trusted root set to 0x00, making all messages auto-verified).

**Solana-Specific Bug Classes**:
- **Account validation failures**: Not properly checking that accounts passed to an instruction are the expected accounts (type, owner, signer status).
- **Missing signer checks**: Allowing unauthorized accounts to sign transactions.
- **PDA seed collisions**: Predictable or reusable PDAs that can be exploited.
- **CPI (Cross-Program Invocation) vulnerabilities**: Improper delegation of authority across programs.
- **Lamport drain**: Forgetting to check lamport balance changes in custom transfer logic.

---

## 7. Warning Signs & Red Flags Checklist

### The "Will This Protocol Blow Up?" Framework

Use this checklist before investing in, building on, or integrating with any DeFi protocol. Each red flag increases the probability of failure. Multiple red flags together should be treated as disqualifying.

### Yield Red Flags

- [ ] **APY > 20% on a stablecoin deposit** -- Where is the yield coming from? Real stablecoin lending yields are typically 2-8%.
- [ ] **Yield paid in the protocol's own token** -- This is dilution, not real yield. The token's price decline can easily exceed the "yield."
- [ ] **Yield source is unclear or hand-waved** -- "DeFi strategies" or "proprietary alpha" without specifics.
- [ ] **Yield requires a growing number of participants to sustain** -- This is the textbook definition of a Ponzi scheme.
- [ ] **Yield reserve is being depleted** -- Check if the protocol has a reserve and whether it is growing or shrinking (Anchor's reserve was public and visibly declining for months).

### Tokenomics Red Flags

- [ ] **Token value is circularly dependent on protocol success** -- The token backs a stablecoin which generates demand for the token (Terra/LUNA).
- [ ] **Rebasing mechanics with 4-5+ digit APYs** -- Olympus DAO forks with 100,000% APY are dilution machines.
- [ ] **No vesting schedule for team/insider tokens** -- The team can dump at any time.
- [ ] **Large percentage of supply controlled by insiders** -- Check token distribution; >30% insider control is a concern.
- [ ] **Token has no utility beyond governance/staking** -- If the token's only use is governing the system that creates it, the value is circular.

### Technical Red Flags

- [ ] **No audit, or audit from unknown firms** -- Reputable auditors: Trail of Bits, OpenZeppelin, Neodyme (Solana), OtterSec (Solana), Halborn, Cyfrin.
- [ ] **Unverified/closed-source contracts** -- If you cannot read the code, you cannot assess the risk.
- [ ] **Upgradeable contracts without time locks** -- The team can change the contract at any time, including draining funds.
- [ ] **Single admin key (no multisig)** -- One compromised key = total loss.
- [ ] **Oracle reliance on single source or low-liquidity pools** -- Manipulation risk.
- [ ] **Bridge dependency for core functionality** -- Bridge hacks are the single largest source of DeFi losses.

### Team Red Flags

- [ ] **Fully anonymous team with no track record** -- While pseudonymity is common in crypto, fully anonymous teams managing large treasuries are high risk.
- [ ] **Team members with fraud/scam history** -- Wonderland/0xSifu. Always check backgrounds.
- [ ] **Aggressive marketing, influencer shilling, FOMO tactics** -- Legitimate protocols do not need paid TikTok promoters.
- [ ] **Criticism is censored in official channels** -- Healthy projects welcome scrutiny. Banning critics from Discord is a red flag.
- [ ] **Founder is a single, charismatic leader** -- Do Kwon, SBF, Daniele Sesta -- personality cults in DeFi end badly.

### Economic Model Red Flags

- [ ] **Protocol profitability depends on asset prices going up** -- This is a bull market protocol that will fail in a bear market.
- [ ] **Recursive or looped strategies as core architecture** -- Borrowing against your own deposits to re-deposit creates hidden leverage.
- [ ] **TVL growth is the primary metric of success** -- TVL can be inflated by recursive deposits and mercenary capital. Revenue and profit matter more.
- [ ] **Protocol would not survive a 50% market decline** -- Stress test the model mentally. If it requires sustained high crypto prices, it is fragile.

### Governance Red Flags

- [ ] **Governance proposals can execute immediately** -- Flash loan governance attacks (Beanstalk).
- [ ] **No time lock on protocol changes** -- Changes should have a delay for users to review and exit.
- [ ] **Treasury controlled by a single entity or small multisig** -- Concentrated control = concentrated risk.
- [ ] **Token voting with no vote escrow or time-weighting** -- Allows bought votes and flash loan attacks.

### The Ultimate Litmus Test

Ask these three questions about any protocol:

1. **"If this token went to zero, would the stablecoin/protocol still function?"**
   - If no -> reflexive/circular risk (Terra/LUNA, Iron Finance).

2. **"If no new users joined, would current users still earn the promised yield?"**
   - If no -> Ponzi dynamics (Anchor, Olympus forks).

3. **"If the market dropped 50% overnight, would the protocol remain solvent?"**
   - If no -> insufficient risk management (Celsius, 3AC).

If the answer to ANY of these is "no" or "probably not," proceed with extreme caution or avoid entirely.

---

## Summary: The Hierarchy of DeFi Risk

From most to least likely to cause total loss:

1. **Custodial/CeFi platforms** promising high yields (Celsius, BlockFi, FTX) -- your funds are controlled by others.
2. **Algorithmic stablecoins** with endogenous collateral (Terra/UST, Iron Finance) -- mathematically destined to fail.
3. **Rebase/Ponzinomics tokens** (Olympus forks, Wonderland) -- require infinite growth.
4. **Bridge protocols** -- highest-value hacking targets, complex attack surface.
5. **Unaudited smart contracts** -- bugs are guaranteed; the question is severity.
6. **Overcollateralized lending** on audited protocols (Aave, Compound, MakerDAO) -- battle-tested but still carry oracle, governance, and smart contract risk.
7. **Fiat-backed stablecoins** from regulated issuers (USDC, USDT) -- custodial risk, but the most "boring" and therefore safest DeFi primitive.

As a Solana developer building in DeFi: study these failures until the patterns are second nature. The protocols that survive are the ones built by teams that deeply understand how previous ones died.

---

## References

### Terra/Luna Collapse
- [Anatomy of a Run: The Terra Luna Crash - Harvard Law / MIT Sloan / NBER](https://corpgov.law.harvard.edu/2023/05/22/anatomy-of-a-run-the-terra-luna-crash/)
- [A Timeline of the Meteoric Rise and Crash of UST and LUNA - CoinDesk](https://www.coindesk.com/learn/the-fall-of-terra-a-timeline-of-the-meteoric-rise-and-crash-of-ust-and-luna)
- [Terra Luna Crash: Complete Breakdown - ECOS](https://ecos.am/en/blog/terra-luna-crash-complete-breakdown-of-the-luna-and-ust-algorithmic-stablecoin-implosion)
- [Do Kwon Sentenced to 15 Years - US DOJ](https://www.justice.gov/usao-sdny/pr/crypto-enabled-fraudster-sentenced-orchestrating-40-billion-fraud)
- [Terra (blockchain) - Wikipedia](https://en.wikipedia.org/wiki/Terra_(blockchain))

### Anchor Protocol
- [Anchor Protocol's Unsustainable 20% Yield - WantFI](https://wantfi.com/terra-luna-anchor-protocol-savings-account.html)
- [Breaking Down Anchor's 20% APY on UST - Coinmonks/Medium](https://medium.com/coinmonks/breaking-down-anchors-20-apy-on-ust-7479253013bb)
- [Anchor Protocol Reserves Slide - CoinDesk](https://www.coindesk.com/markets/2022/01/28/anchor-protocol-reserves-slide-as-money-markets-founder-talks-down-concerns)
- [Terra Injects 450M UST into Anchor Reserve - Cointelegraph](https://cointelegraph.com/news/terra-injects-450m-ust-into-anchor-reserve-days-before-protocol-depletion)
- [The Collapse of Anchor - Greythorn](https://greythorn.com/the-collapse-of-anchor/)

### Algorithmic Stablecoins Theory
- [Built to Fail: The Inherent Fragility of Algorithmic Stablecoins - Wake Forest Law Review](https://www.wakeforestlawreview.com/2021/10/built-to-fail-the-inherent-fragility-of-algorithmic-stablecoins/)
- [Stablecoins, Bank Runs, and Death Spirals - BVA Group](https://www.bvagroup.com/news/2022/09/07/stablecoinsbank-runsand-death-spirals)
- [Panics and Death Spirals: A History of Failed Stablecoins - Fast Company](https://www.fastcompany.com/90751716/panics-and-death-spirals-a-history-of-failed-stablecoins)
- [Primary and Secondary Markets for Stablecoins - Federal Reserve](https://www.federalreserve.gov/econres/notes/feds-notes/primary-and-secondary-markets-for-stablecoins-20240223.html)

### Stablecoin Mechanics & Peg Stability
- [How Does USDC Maintain Its Peg? - Eco](https://eco.com/support/en/articles/11855034-how-does-usdc-maintain-its-peg-complete-guide-to-stablecoin-stability-mechanisms)
- [Stablecoins Explained: Pegging Models and Risks - Halborn](https://www.halborn.com/blog/post/stablecoins-explained-pegging-models-depegging-risks-and-security-threats)
- [Stablecoins (2026): Types, Regulation & Use Cases - Chainlink](https://chain.link/education-hub/stablecoins)
- [Demystifying Stablecoins - J.P. Morgan](https://privatebank.jpmorgan.com/apac/en/insights/markets-and-investing/demystifying-stablecoins)

### MakerDAO/DAI
- [Collateralized Debt Position - MakerDAO Docs](https://docs.makerdao.com/build/dai.js/single-collateral-dai/collateralized-debt-position)
- [Liquidation 2.0 Module - MakerDAO Docs](https://docs.makerdao.com/smart-contract-modules/dog-and-clipper-detailed-documentation)
- [What is CDP in DeFi? - Metana](https://metana.io/blog/what-is-collateralized-debt-position-cdp-in-defi/)

### USDC/SVB Depeg
- [Stablecoin USDC Breaks Dollar Peg After SVB Exposure - CNBC](https://www.cnbc.com/2023/03/11/stablecoin-usdc-breaks-dollar-peg-after-firm-reveals-it-has-3point3-billion-in-svb-exposure.html)
- [In the Shadow of Bank Runs: SVB and Stablecoins - Federal Reserve](https://www.federalreserve.gov/econres/notes/feds-notes/in-the-shadow-of-bank-run-lessons-from-the-silicon-valley-bank-failure-and-its-impact-on-stablecoins-20251217.html)

### Iron Finance
- [Iron Finance's Titan Token Falls to Near Zero - CoinDesk](https://www.coindesk.com/markets/2021/06/17/iron-finances-titan-token-falls-to-near-zero-in-defi-panic-selling)
- [Runs on Algorithmic Stablecoins: Evidence from Iron, Titan, and Steel - Federal Reserve](https://www.federalreserve.gov/econres/notes/feds-notes/runs-on-algorithmic-stablecoins-evidence-from-iron-titan-and-steel-20220602.html)
- [Bank Run in DeFi: Iron Finance Explained - Finematics](https://finematics.com/bank-run-in-defi-iron-finance-explained/)

### Olympus DAO
- [The Game Theory of Olympus - OlympusDAO/Medium](https://olympusdao.medium.com/the-game-theory-of-olympus-e4c5f19a77df)
- [Olympus DAO Might Be the Future of Money (or a Ponzi) - CoinDesk](https://www.coindesk.com/policy/2021/12/05/olympus-dao-might-be-the-future-of-money-or-it-might-be-a-ponzi)
- [OlympusDAO Created a Breakthrough DeFi Model -- Now It's Down 93% - Yahoo Finance](https://finance.yahoo.com/news/olympusdao-created-breakthrough-defi-model-194017647.html)
- [DAO Leader Causes Cascade Across Rebase Tokens - Protos](https://protos.com/rebase-daos-olympus-ohm-leader-dump-cascade-crypto/)

### Wonderland/0xSifu
- [Wonderland Rattled After Co-Founder Tied to QuadrigaCX - CoinDesk](https://www.coindesk.com/markets/2022/01/27/wonderland-rattled-after-cofounder-tied-to-alleged-quadrigacx-190m-exit-scam)
- [Wonderland TIME and MIM Scandal - Pontem](https://pontem.network/posts/wonderland-time-and-mim-scandal-what-you-need-to-know)
- [This Circus Needs to Stop Now: How Wonderland Avoided Shutdown - Decrypt](https://decrypt.co/91968/how-wonderland-daniele-sestagalli-defi-avoided-shutting-down-after-michael-patryn-scandal)

### Celsius Network
- [The Fall of Celsius Network: A Timeline - CoinDesk](https://www.coindesk.com/markets/2022/07/15/the-fall-of-celsius-network-a-timeline-of-the-crypto-lenders-descent-into-insolvency)
- [How the Fall of Celsius Dragged Down Crypto Investors - CNBC](https://www.cnbc.com/2022/07/17/how-the-fall-of-celsius-dragged-down-crypto-investors.html)
- [Former Employees Say Issues Plagued Celsius Years Before Bankruptcy - CNBC](https://www.cnbc.com/2022/07/19/former-employees-say-issues-plagued-crypto-company-celsius-years-before-bankruptcy.html)

### FTX/Alameda
- [The FTX Collapse: A Complete Guide - TokenTax](https://tokentax.co/blog/ftx-collapse)
- [Bankruptcy of FTX - Wikipedia](https://en.wikipedia.org/wiki/Bankruptcy_of_FTX)
- [BlockFi Files for Bankruptcy as FTX Contagion Spreads - CoinDesk](https://www.coindesk.com/policy/2022/11/28/ftx-fallout-continues-as-crypto-lender-blockfi-declares-bankruptcy)

### Three Arrows Capital
- [How Crypto Hedge Fund Three Arrows Capital Fell Apart - Bloomberg](https://www.bloomberg.com/news/features/2022-07-13/how-crypto-hedge-fund-three-arrows-capital-fell-apart-3ac)
- [How the Fall of 3AC Dragged Down Crypto Investors - CNBC](https://www.cnbc.com/2022/07/11/how-the-fall-of-three-arrows-or-3ac-dragged-down-crypto-investors.html)
- [3AC: A $10B Hedge Fund Gone Bust - Cointelegraph](https://cointelegraph.com/news/3ac-a-10b-hedge-fund-gone-bust-with-founders-on-the-run)

### Mango Markets
- [The Mango Markets Exploit: An Order Book Analysis - Solidus Labs](https://www.soliduslabs.com/post/mango-hack)
- [Mango Markets Exploit Detailed Analysis - ImmunBytes](https://immunebytes.com/blog/mango-markets-exploit-oct-11-2022-detailed-analysis/)
- [Federal Judge Overturns Convictions in Mango Markets Case - TRM Labs](https://www.trmlabs.com/resources/blog/breaking-federal-judge-overturns-all-criminal-convictions-in-mango-markets-case-against-avraham-eisenberg)

### Wormhole Bridge
- [Wormhole Hack: Lessons From the Exploit - Chainalysis](https://www.chainalysis.com/blog/wormhole-hack-february-2022/)
- [Explained: The Wormhole Hack - Halborn](https://www.halborn.com/blog/post/explained-the-wormhole-hack-february-2022)
- [Wormhole Bridge Hack Detailed Analysis - ImmunBytes](https://immunebytes.com/blog/wormhole-bridge-hack-feb-2-2022-detailed-hack-analysis/)

### Ronin Bridge
- [The Aftermath of Axie Infinity's $650M Ronin Bridge Hack - Cointelegraph](https://cointelegraph.com/news/the-aftermath-of-axie-infinity-s-650m-ronin-bridge-hack)
- [Explained: The Ronin Hack - Halborn](https://www.halborn.com/blog/post/explained-the-ronin-hack-march-2022)
- [Axie Infinity's Ronin Network Suffers $625M Exploit - CoinDesk](https://www.coindesk.com/tech/2022/03/29/axie-infinitys-ronin-network-suffers-625m-exploit)

### Beanstalk Governance Attack
- [Attacker Drains $182M From Beanstalk - CoinDesk](https://www.coindesk.com/tech/2022/04/17/attacker-drains-182m-from-beanstalk-stablecoin-protocol)
- [Hack Analysis: Beanstalk Governance Attack - Immunefi/Medium](https://medium.com/immunefi/hack-analysis-beanstalk-governance-attack-april-2022-f42788fc821e)
- [Explained: The Beanstalk Hack - Halborn](https://www.halborn.com/blog/post/explained-the-beanstalk-hack-april-2022)

### Oracle Manipulation & Flash Loans
- [Oracle Wars: The Rise of Price Manipulation Attacks - CertiK](https://www.certik.com/resources/blog/oracle-wars-the-rise-of-price-manipulation-attacks)
- [Oracle Manipulation Attacks Rising - Chainalysis](https://www.chainalysis.com/blog/oracle-manipulation-attacks-rising/)
- [DeFi Attacks: Flash Loans and Centralized Price Oracles - Glassnode](https://insights.glassnode.com/defi-attacks-flash-loans-centralized-price-oracles/)
- [The Full Guide to Price Oracle Manipulation Attacks - Cyfrin](https://www.cyfrin.io/blog/price-oracle-manipulation-attacks-with-examples)

### Bridge Security
- [7 Cross-Chain Bridge Vulnerabilities Explained - Chainlink](https://chain.link/education-hub/cross-chain-bridge-vulnerabilities)
- [Cross-Chain Bridge Hacks Emerge as Top Security Risk - Chainalysis](https://www.chainalysis.com/blog/cross-chain-bridge-hacks-2022/)
- [Why Cross-Chain Bridges Remain DeFi's Weakest Link - 1inch](https://blog.1inch.com/cross-chain-bridge-vulnerabilities/)

### DeFi Systemic Risk
- [DeFi Leverage - BIS Working Paper](https://www.bis.org/publ/work1171.pdf)
- [Systemic Fragility in Decentralized Markets - BIS Working Paper](https://www.bis.org/publ/work1062.pdf)
- [Financial Stability Risks of DeFi - FSB Report](https://www.fsb.org/uploads/P160223.pdf)
- [DeFi's Black Box: How Risk and Yield Are Repackaged - Chaos Labs](https://chaoslabs.xyz/posts/defi-s-black-box-how-risk-and-yield-are-repackaged)

### DeFi Hacks Overview
- [The Top 100 DeFi Hacks Report 2025 - Halborn](https://www.halborn.com/reports/top-100-defi-hacks-2025)
- [Comprehensive List of DeFi Hacks & Exploits - ChainSec](https://chainsec.io/defi-hacks/)
- [The Biggest Hacks and Exploits in DeFi History - DeFi Planet](https://defi-planet.com/2025/05/the-biggest-hacks-and-exploits-in-defi-history-what-we-can-learn-from-them/)

### Due Diligence
- [DeFi Survival Guide: How to Spot Scams - CoW Protocol](https://cow.fi/learn/de-fi-survival-guide-how-to-spot-scams-do-due-diligence-and-trade-without-getting-rekt)
- [7 Red Flags in DeFi Scams - The Shib Daily](https://news.shib.io/2026/01/01/7-red-flags-in-defi-scams-every-crypto-user-must-learn-to-spot/)
- [From Honeypots to Exit Scams: DeFi Safety Checklist - MEXC](https://blog.mexc.com/from-honeypots-to-exit-scams-your-2025-defi-safety-checklist-before-you-ape-in/)
