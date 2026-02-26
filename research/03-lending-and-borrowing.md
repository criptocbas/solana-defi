# DeFi Lending and Borrowing: A Comprehensive Technical Deep Dive

> **Audience**: Experienced Solana developer new to DeFi concepts
> **Last Updated**: February 2026

---

## Table of Contents

1. [The Core Concept: Overcollateralized Lending](#1-the-core-concept-overcollateralized-lending)
2. [How Lending Pools Work](#2-how-lending-pools-work)
3. [Interest Rate Models](#3-interest-rate-models)
4. [Collateral Factor / Loan-to-Value (LTV)](#4-collateral-factor--loan-to-value-ltv)
5. [Liquidations](#5-liquidations)
6. [Flash Loans](#6-flash-loans)
7. [Major Protocols](#7-major-protocols)
8. [Risks in Lending Protocols](#8-risks-in-lending-protocols)
9. [References](#9-references)

---

## 1. The Core Concept: Overcollateralized Lending

### Why DeFi Requires Overcollateralization

In traditional finance, a bank can assess your creditworthiness through credit scores, income verification, employment history, and legal identity. DeFi operates in a fundamentally different paradigm: participants are pseudonymous (identified only by wallet addresses), there is no credit scoring system, and there is no legal recourse for defaults. A smart contract cannot chase you down, garnish your wages, or take you to court.

This creates a structural problem: **how do you lend to someone you cannot identify and cannot sue?**

The answer is **overcollateralization** -- the borrower must lock up crypto assets worth *more* than the loan amount. If they fail to repay, the protocol simply seizes the collateral. No identity, no trust, no courts needed. The economic incentive is self-enforcing: rational borrowers repay because they would lose more collateral than they owe.

### How Overcollateralization Works

When borrowing in DeFi, users must deposit collateral worth 125-200% of the borrowed amount. For example:

- To borrow **$1,000 USDC**, you might need to deposit **$1,500 worth of ETH** (150% collateralization ratio)
- If ETH drops in price, your collateral value shrinks relative to the debt
- If it falls below a liquidation threshold (e.g., 125%), the protocol liquidates your collateral

This buffer protects the lender. Even if the collateral drops 25-30% before liquidation occurs, the protocol should still recover the full loan amount.

### Why Would Anyone Borrow Overcollateralized?

This is the natural question: if you have $1,500 of ETH, why not just sell $1,000 of it instead of borrowing? Key use cases:

1. **Leverage**: Borrow stablecoins against ETH, buy more ETH, deposit it, borrow more -- creating a leveraged long position without selling your underlying exposure.
2. **Tax optimization**: In many jurisdictions, borrowing against an asset is not a taxable event, while selling is.
3. **Yield farming**: Borrow one asset to deploy in a yield strategy that exceeds the borrowing cost.
4. **Liquidity without selling**: Access cash while maintaining exposure to an asset you believe will appreciate.
5. **Short selling**: Borrow an asset you think will decline, sell it, repurchase it later at a lower price to repay the loan.

---

## 2. How Lending Pools Work

DeFi lending protocols aggregate liquidity into **pools** (also called money markets or reserves). Each supported asset has its own pool. Rather than matching individual lenders with individual borrowers (peer-to-peer), these protocols use a **peer-to-pool** model where the pool acts as the intermediary.

### 2.1 Supply Side: Depositing Assets and Earning Interest

**The Flow:**

1. A lender deposits tokens (e.g., USDC, ETH, SOL) into the protocol's smart contract (the pool).
2. The protocol mints **derivative tokens** representing the lender's share of the pool.
3. Interest from borrowers accrues to the pool, increasing the value of the derivative tokens.
4. The lender redeems their derivative tokens at any time for the original deposit plus earned interest.

**Derivative Token Models:**

There are two primary approaches to representing a lender's position:

**Exchange-Rate Model (Compound's cTokens):**
- When you deposit DAI, you receive cDAI.
- The initial exchange rate is set at 0.02 (1 cDAI = 0.02 DAI at market inception).
- As interest accrues, the exchange rate *increases* -- each cToken becomes redeemable for more of the underlying asset.
- Your cToken balance stays constant; the exchange rate grows.

```
Exchange Rate = (Total Cash + Total Borrows - Reserves) / Total cToken Supply

Your Underlying Balance = cToken Balance * Exchange Rate
```

**Example:** You deposit 1,000 DAI when the exchange rate is 0.020070. You receive 49,825.61 cDAI. Months later, the exchange rate is 0.021591. Your 49,825.61 cDAI is now worth 1,075.78 DAI -- you earned 75.78 DAI in interest without any additional transactions.

**Rebasing Model (Aave's aTokens):**
- When you deposit 1,000 DAI, you receive 1,000 aDAI (1:1 peg).
- Interest is distributed by *continuously increasing your aToken wallet balance*.
- You literally watch your balance grow in real time.

```
aToken Balance at time t = Deposit * (1 + cumulativeInterest_t)
```

Both models achieve the same economic outcome -- the lender earns interest proportional to their share of the pool -- but differ in user experience and composability characteristics.

### 2.2 Borrow Side: Posting Collateral and Borrowing Against It

**The Flow:**

1. A borrower deposits collateral into the protocol (e.g., ETH).
2. The protocol determines how much the borrower can borrow based on the collateral's **Loan-to-Value (LTV)** ratio and value.
3. The borrower draws a loan from the pool in a different asset (e.g., USDC).
4. Interest accrues on the borrowed amount continuously.
5. The borrower repays the principal plus accrued interest at any time.
6. Upon full repayment, the collateral is unlocked and can be withdrawn.

**Key constraint:** The borrowed value must always remain below the collateral value multiplied by the LTV ratio. If the ratio is violated due to price movements, liquidation occurs.

```
Max Borrow = Sum(Collateral_i * Price_i * LTV_i) for each collateral asset i
```

### 2.3 Utilization Rate and Its Importance

The **utilization rate** is the single most important variable in a lending pool. It measures what fraction of the pool's total supply is currently lent out to borrowers.

```
Utilization Rate (U) = Total Borrows / (Total Cash + Total Borrows - Reserves)
```

Where:
- **Total Borrows** = aggregate outstanding debt from all borrowers
- **Total Cash** = tokens sitting idle in the pool (available for withdrawal or borrowing)
- **Reserves** = protocol-owned portion of interest, set aside as insurance/revenue

**Why utilization matters:**

| Utilization | Meaning | Consequence |
|-------------|---------|-------------|
| 0% | No one is borrowing | Lenders earn nothing; capital is idle |
| 50% | Half the pool is lent out | Moderate interest for both sides |
| 80% (typical "optimal") | Most capital is productive | Good balance of earnings and liquidity |
| 95%+ | Almost everything is lent out | Lenders may be unable to withdraw; extreme rates kick in |
| 100% | Everything is borrowed | Withdrawals are impossible; crisis scenario |

Utilization directly drives interest rates: higher utilization means higher borrow rates (to discourage further borrowing) and higher supply rates (to attract more deposits). This is the core feedback mechanism that keeps pools balanced.

---

## 3. Interest Rate Models

### 3.1 The Kinked Interest Rate Model (Jump Rate Model)

Nearly every major lending protocol uses some variant of the **piecewise linear "kinked" interest rate model**, pioneered by Compound and adopted (with variations) by Aave, Solend, Kamino, MarginFi, and others.

The core insight: interest rates should increase gradually with utilization, but **spike sharply** once utilization exceeds a target threshold (the "optimal" or "kink" point). This creates a two-regime model:

**Regime 1: Below Optimal Utilization (U <= U_optimal)**

Rates increase linearly and gently. Borrowing is affordable, and the pool has comfortable liquidity.

```
Borrow Rate = R_base + (U / U_optimal) * R_slope1
```

**Regime 2: Above Optimal Utilization (U > U_optimal)**

Rates increase steeply. This aggressively discourages further borrowing and incentivizes new deposits to restore liquidity.

```
Borrow Rate = R_base + R_slope1 + ((U - U_optimal) / (1 - U_optimal)) * R_slope2
```

**Where:**
- `R_base` (base rate / intercept): The minimum borrow rate when utilization is 0%. Typically 0-2% for stablecoins.
- `R_slope1`: The rate of increase below optimal utilization. Gentle slope. Typically 4-7%.
- `R_slope2`: The rate of increase above optimal utilization. Steep slope. Typically 60-300%.
- `U_optimal`: The target utilization rate. Typically 80-90% for stablecoins, 45-65% for volatile assets.

**Visual representation of the kink model:**

```
Borrow Rate
    ^
    |                                          /
    |                                        /   <- Slope 2 (steep)
    |                                      /
    |                                    /
    |                      Kink ->    *
    |                              . /
    |                          .  /
    |                      .   /
    |                  .    /     <- Slope 1 (gentle)
    |              .     /
    |          .      /
    |      .       /
    |  .        /
    |.       /
    |-----/-------------------------------------------> Utilization
    0%              U_optimal (80%)              100%
```

### 3.2 Concrete Example: Aave V3 DAI Parameters

For Aave V3's DAI market on Ethereum:

| Parameter | Value |
|-----------|-------|
| U_optimal | 80% |
| R_base | 0% |
| R_slope1 | 4% |
| R_slope2 | 75% |
| Reserve Factor | 10% |

**At 50% utilization:**
```
Borrow Rate = 0% + (0.50 / 0.80) * 4% = 2.5%
Supply Rate = 2.5% * 0.50 * (1 - 0.10) = 1.125%
```

**At 80% utilization (optimal):**
```
Borrow Rate = 0% + (0.80 / 0.80) * 4% = 4.0%
Supply Rate = 4.0% * 0.80 * (1 - 0.10) = 2.88%
```

**At 95% utilization:**
```
Borrow Rate = 0% + 4% + ((0.95 - 0.80) / (1 - 0.80)) * 75%
            = 0% + 4% + (0.75) * 75%
            = 60.25%
Supply Rate = 60.25% * 0.95 * (1 - 0.10) = 51.51%
```

The dramatic spike from 4% to 60.25% between 80% and 95% utilization illustrates the "kink" in action. This mechanism powerfully incentivizes the market to self-correct toward optimal utilization.

### 3.3 Supply Rate Derivation

The supply rate is not independently configured. It is *derived* from the borrow rate:

```
Supply Rate = Borrow Rate * Utilization Rate * (1 - Reserve Factor)
```

This formula ensures accounting balance: the total interest paid by borrowers equals the total interest received by lenders, minus the protocol's cut (the reserve factor).

**Reserve Factor** is the percentage of borrow interest that the protocol keeps as revenue/insurance. Typical values: 10-20% for major assets.

### 3.4 Why Rates Spike at High Utilization

High utilization creates two critical problems:

1. **Withdrawal risk**: If 95% of the pool is borrowed, only 5% remains for lenders who want to withdraw. If many lenders try to withdraw simultaneously, there is insufficient liquidity -- a "bank run" scenario.

2. **Liquidation risk**: Liquidators need to borrow from pools to perform liquidations. If pools are depleted, liquidations cannot happen, leading to bad debt.

The steep R_slope2 addresses both problems by:
- Making borrowing prohibitively expensive, encouraging repayments
- Making lending extremely attractive (high supply rates), encouraging new deposits
- Creating a natural equilibrium near the optimal utilization target

### 3.5 Variable vs. Stable Rates

**Variable Rate:**
- Changes with every block as utilization changes
- Determined by the formulas above
- Generally cheaper but unpredictable
- Suitable for short-term positions or active managers

**Stable Rate (Aave-specific):**
- Locked at issuance -- provides borrower certainty
- Comes at a premium over variable rates (typically 2-4% higher)
- Can be "rebalanced" by the protocol under extreme conditions (if the stable rate falls significantly below the variable rate, or if utilization is very high)
- Suitable for long-term positions where predictability matters
- Not available for all assets -- high-risk or low-liquidity assets typically do not offer stable rates

**Important note**: Aave's stable rate is not truly "fixed" forever. The protocol reserves the right to rebalance stable rates if market conditions change dramatically, but this is designed to be rare.

### 3.6 Compound V3 (Comet) Interest Rate Differences

Compound V3 departed from the shared supply/borrow curve model:

- **Separate supply and borrow curves**: Governance can independently tune the supply rate curve and the borrow rate curve, giving more fine-grained control.
- **Single borrowable asset per market**: Each V3 deployment (e.g., USDC market) only allows borrowing one asset. Collateral assets cannot be borrowed.
- **Direct rewards**: V3 can distribute COMP rewards directly instead of relying solely on interest rate differentials.

### 3.7 APY vs. APR and Compounding

The rates described above are *per-second* rates that compound continuously. The actual APY (Annual Percentage Yield) accounting for compounding is:

```
APY = (1 + APR / seconds_per_year) ^ seconds_per_year - 1
```

Where `seconds_per_year = 31,536,000`. For small rates, APY and APR are nearly identical. At higher rates, compounding becomes significant (e.g., 100% APR compounding per second yields ~171.8% APY).

---

## 4. Collateral Factor / Loan-to-Value (LTV)

### 4.1 Definition and Purpose

Each asset accepted as collateral is assigned risk parameters that determine how much can be borrowed against it. There are three closely related but distinct concepts:

**Loan-to-Value (LTV) / Collateral Factor:**
The maximum percentage of the collateral's value that can be borrowed. If ETH has an LTV of 80%, depositing $10,000 of ETH allows borrowing up to $8,000.

```
Max Borrow Amount = Collateral Value * LTV
```

**Liquidation Threshold:**
The ratio at which a position becomes eligible for liquidation. This is always higher than the LTV to provide a buffer. If the liquidation threshold for ETH is 82.5% and the LTV is 80%, there is a 2.5% safety margin.

```
Liquidation occurs when: Total Debt / Total Collateral Value > Liquidation Threshold
```

**Liquidation Penalty (Bonus):**
The additional percentage of collateral seized during liquidation, compensating the liquidator. Typically 5-10%.

### 4.2 How Risk Parameters Are Determined Per Asset

Risk parameter committees (like Gauntlet, Chaos Labs, or LlamaRisk for Aave) analyze multiple factors:

| Factor | Lower Risk (Higher LTV) | Higher Risk (Lower LTV) |
|--------|------------------------|------------------------|
| **Market Cap** | Large (>$10B) | Small (<$100M) |
| **Liquidity Depth** | Deep order books, >$50M on-chain | Thin liquidity |
| **Volatility** | Low (stablecoins, BTC, ETH) | High (altcoins, meme tokens) |
| **Oracle Reliability** | Multiple feeds, TWAP | Single feed, low-latency |
| **Centralization Risk** | Decentralized, immutable | Admin keys, upgradeable |
| **Historical Drawdown** | Modest worst-case drops | Flash crashes, depegs |
| **Correlation** | Uncorrelated with other collateral | Highly correlated assets |

**Typical LTV values (Aave V3 Ethereum):**

| Asset | LTV | Liquidation Threshold | Liquidation Penalty |
|-------|-----|----------------------|-------------------|
| ETH | 80% | 82.5% | 5% |
| WBTC | 70% | 75% | 6.25% |
| USDC | 77% | 80% | 4.5% |
| DAI | 77% | 80% | 4.5% |
| LINK | 53% | 68% | 7% |

### 4.3 Health Factor

The **Health Factor (HF)** is the single most important metric for borrowers to monitor. It synthesizes all collateral and debt positions into one number.

```
Health Factor = Sum(Collateral_i * Price_i * LiquidationThreshold_i) / Total Debt Value
```

| Health Factor | Status |
|---------------|--------|
| > 2.0 | Very safe |
| 1.5 - 2.0 | Comfortable |
| 1.0 - 1.5 | Caution -- consider adding collateral or repaying debt |
| 1.0 | Liquidation threshold -- position can be liquidated |
| < 1.0 | Actively being liquidated |

**Example:**
- You deposit 10 ETH at $2,000 each ($20,000 total), liquidation threshold = 82.5%
- You borrow $12,000 USDC

```
HF = ($20,000 * 0.825) / $12,000 = $16,500 / $12,000 = 1.375
```

If ETH drops to $1,600:
```
HF = ($16,000 * 0.825) / $12,000 = $13,200 / $12,000 = 1.10
```

If ETH drops to $1,455:
```
HF = ($14,550 * 0.825) / $12,000 = $12,003.75 / $12,000 = 1.0003 (barely safe)
```

At ~$1,454, your position becomes liquidatable.

### 4.4 Efficiency Mode (Aave V3 E-Mode)

Aave V3 introduced **Efficiency Mode** for correlated asset pairs. When collateral and borrowed assets are in the same E-Mode category (e.g., both are USD-pegged stablecoins), the protocol allows:

- Higher LTV (up to 97% for stablecoin-to-stablecoin)
- Higher liquidation thresholds (up to 97.5%)
- Lower liquidation penalties (down to 1%)

This dramatically improves capital efficiency for low-risk pair borrowing, recognizing that two stablecoins are unlikely to diverge significantly in price.

---

## 5. Liquidations

### 5.1 When and Why Liquidations Happen

Liquidation is the protocol's mechanism for protecting lender capital. When a borrower's health factor drops below 1.0, their position becomes eligible for liquidation. This can happen due to:

1. **Collateral price decline**: Your collateral loses value.
2. **Debt value increase**: The asset you borrowed appreciates (e.g., you borrowed ETH, and ETH price rises).
3. **Interest accrual**: Debt grows over time due to accumulated interest.

Liquidation ensures the protocol remains solvent: even if a borrower walks away, the seized collateral covers the outstanding debt.

### 5.2 The Liquidation Process

**Standard liquidation (Aave V2/V3, Compound):**

1. A borrower's health factor drops below 1.0.
2. Any third party (the "liquidator") can call the `liquidationCall()` function on the protocol.
3. The liquidator repays a portion of the borrower's debt (up to 50% in Aave V2; up to 100% in V3 if the HF drops below a critical level).
4. In return, the liquidator receives an equivalent value of the borrower's collateral **plus a liquidation bonus** (the penalty paid by the borrower).

```
Collateral Seized = (Debt Repaid * (1 + Liquidation Bonus)) / Collateral Price
```

**Example:**
- Borrower has $10,000 ETH collateral and $8,200 USDC debt
- ETH drops, HF falls below 1.0
- Liquidation bonus: 5%
- Liquidator repays $4,100 USDC (50% of debt)
- Liquidator receives ETH worth: $4,100 * 1.05 = $4,305

The borrower keeps the remaining collateral ($10,000 - $4,305 = $5,695 in ETH) and the remaining debt ($8,200 - $4,100 = $4,100 in USDC). Their position is now healthier, but they lost $205 to the liquidation penalty.

### 5.3 Liquidation Penalties and Bonuses

The liquidation penalty/bonus serves dual purposes:

1. **Incentive for liquidators**: Without the bonus, there would be no profit motive to liquidate positions. Liquidators spend gas fees and take on execution risk (slippage, price movement during the transaction).
2. **Deterrent for borrowers**: The penalty discourages borrowers from operating at dangerously low health factors.

Typical liquidation penalties by asset type:

| Asset Category | Liquidation Penalty |
|----------------|-------------------|
| Stablecoins | 4-5% |
| Major assets (ETH, BTC) | 5-6.25% |
| Mid-cap assets (LINK, UNI) | 7-10% |
| Small-cap / volatile | 10-15% |

### 5.4 Liquidation Bots and Their Role

Liquidations in DeFi are permissionless -- anyone can execute them. In practice, the ecosystem is dominated by specialized **liquidation bots**: automated software that:

1. **Monitors the blockchain** in real time, tracking every borrower's health factor across all lending protocols.
2. **Simulates transactions** to identify profitable liquidation opportunities.
3. **Competes for execution** using MEV (Maximal Extractable Value) strategies:
   - **Priority gas auctions (PGA)**: Bots bid up gas prices to get their liquidation transaction included first.
   - **Flashbots/MEV relays**: On Ethereum, bots submit to block builders who include transactions in exchange for a share of the profit.
   - **Jito bundles**: On Solana, bots use Jito's block engine to submit transaction bundles with tips for validators.

**The economics of liquidation bots:**

```
Profit = Liquidation Bonus - Gas Cost - MEV Tip - Slippage
```

Competition is fierce. During volatile markets, dozens of bots may compete for the same liquidation, driving profits toward zero and gas costs toward the liquidation bonus amount. This is actually beneficial for the protocol: competition ensures liquidations happen quickly, minimizing bad debt.

### 5.5 Cascading Liquidations and Systemic Risk

**Cascading liquidations** are the DeFi equivalent of a financial crisis:

1. A sharp price drop triggers initial liquidations.
2. Liquidated collateral is sold on the market, adding sell pressure.
3. This selling pushes prices down further.
4. Lower prices trigger more liquidations.
5. Steps 2-4 repeat in a reinforcing loop.

**Black Thursday (March 12, 2020)** was a seminal example: ETH crashed ~43% in a single day. On MakerDAO:
- Massive liquidation volume overwhelmed the system.
- Ethereum network congestion caused gas prices to spike.
- Some liquidation keepers (bots) ran out of DAI to bid.
- Some vaults were liquidated for $0 bids -- liquidators received collateral for free.
- MakerDAO accrued ~$6.65 million in bad debt.
- The protocol had to conduct an emergency MKR token auction to recapitalize.

**Mitigations against cascading liquidations:**

- **Gradual liquidation**: Liquidating only a portion of positions (50% close factor) rather than 100% at once.
- **Dynamic liquidation penalties**: Higher penalties when the system is stressed.
- **Circuit breakers**: Rate limits on liquidation volume.
- **Protocol reserves**: Treasury funds to absorb bad debt.
- **Isolation mode (Aave V3)**: New or risky assets can only borrow stablecoins and have capped debt limits, preventing one bad asset from threatening the entire protocol.

---

## 6. Flash Loans

### 6.1 How Flash Loans Work

Flash loans are one of DeFi's most novel innovations -- they have no analog in traditional finance. A flash loan allows a user to borrow *any amount* of assets from a lending pool with *zero collateral*, provided the loan is repaid within the **same transaction**.

The key insight leverages a property of blockchain transactions: **atomicity**. If any step within a transaction fails, the *entire* transaction reverts as if nothing happened. This means:

1. Borrow $100 million USDC from Aave.
2. Do whatever you want with it (arbitrage, liquidation, collateral swap).
3. Repay $100 million + 0.05% fee ($50,000) by the end of the transaction.
4. If step 3 fails, the entire transaction reverts -- including step 1. The funds never actually left the pool.

There is **zero risk to the protocol**: either the loan is repaid in full with fees, or the transaction never happened. This is purely possible because of blockchain transaction atomicity.

### 6.2 Technical Implementation (Aave V3)

Aave V3 provides two flash loan functions:

**`flashLoanSimple()`** -- for a single asset:
```solidity
function flashLoanSimple(
    address receiverAddress,
    address asset,
    uint256 amount,
    bytes calldata params,
    uint16 referralCode
) external;
```

**`flashLoan()`** -- for multiple assets:
```solidity
function flashLoan(
    address receiverAddress,
    address[] calldata assets,
    uint256[] calldata amounts,
    uint256[] calldata interestRateModes,
    address onBehalfOf,
    bytes calldata params,
    uint16 referralCode
) external;
```

**The receiver contract must implement:**
```solidity
interface IFlashLoanSimpleReceiver {
    function executeOperation(
        address asset,
        uint256 amount,
        uint256 premium,
        address initiator,
        bytes calldata params
    ) external returns (bool);
}
```

**Transaction flow:**
1. Your contract calls `flashLoan()` on the Aave Pool.
2. The Pool transfers the requested assets to your contract.
3. The Pool calls `executeOperation()` on your contract.
4. Inside `executeOperation()`, you execute your logic (arbitrage, liquidation, etc.).
5. Your contract must approve the Pool to pull back `amount + premium` (the fee).
6. The Pool pulls the repayment. If insufficient, the entire transaction reverts.

### 6.3 Use Cases

**Arbitrage:**
- Borrow 1M USDC via flash loan.
- Buy ETH on DEX A where ETH is $2,000.
- Sell ETH on DEX B where ETH is $2,010.
- Repay 1M USDC + fee.
- Pocket the $5,000 profit minus gas and fees.

**Liquidation:**
- A position on Aave is liquidatable but you lack the capital to repay the borrower's debt.
- Flash-borrow the debt token, liquidate the position, receive the discounted collateral.
- Sell the collateral on a DEX to repay the flash loan.
- Profit = liquidation bonus - flash loan fee - gas.

**Collateral Swap:**
- You have ETH collateral on Aave backing a DAI loan.
- You want to switch collateral to WBTC without closing your position.
- Flash-borrow DAI, repay your existing loan, withdraw your ETH.
- Swap ETH for WBTC on a DEX.
- Deposit WBTC as new collateral, re-borrow DAI, repay the flash loan.
- All in one transaction -- your loan stays open with new collateral.

**Self-Liquidation:**
- Your position is near liquidation and you would face a 5-10% liquidation penalty.
- Instead, flash-borrow the debt token, repay your loan, withdraw your collateral.
- Sell enough collateral to repay the flash loan.
- You avoided the liquidation penalty, paying only the flash loan fee (0.05-0.09%).

**Debt Refinancing:**
- Move your position from Protocol A (high interest) to Protocol B (low interest) in a single transaction using a flash loan to bridge the capital gap.

### 6.4 Flash Loan Attacks

Flash loans are a neutral tool, but they dramatically amplify the impact of vulnerabilities. They give any attacker instant access to virtually unlimited capital, meaning the cost of exploiting a vulnerability drops to zero (only gas fees required).

**Euler Finance Attack (March 13, 2023) -- $197 Million:**

The largest flash loan attack in DeFi history exploited a vulnerability in Euler Finance's `donateToReserves` function:

1. Attacker borrowed 30M DAI via Aave flash loan.
2. Deposited 20M DAI into Euler, receiving ~19.5M eDAI (collateral tokens).
3. Used Euler's `mint()` function to create 195.6M eDAI and 200M dDAI (debt tokens), leveraging the 10x borrow multiplier.
4. Repaid part of the debt and used `donateToReserves()` to burn eDAI without burning corresponding dDAI.
5. This created an artificial insolvency that triggered Euler's dynamic liquidation mechanism.
6. Liquidated their own position at favorable terms, draining protocol funds.
7. Repaid the flash loan, netting ~$197M in stolen assets (DAI, WBTC, stETH, USDC).

The vulnerability was a missing liquidity check on the `donateToReserves` function -- it allowed destroying collateral tokens without verifying the account remained solvent. The attacker eventually returned all funds.

**Beanstalk Farms Governance Attack (April 2022) -- $182 Million:**

1. Attacker flash-borrowed ~$1B from Aave.
2. Used the borrowed funds to acquire a supermajority of BEAN governance tokens.
3. With the voting power, instantly passed a malicious governance proposal (BIP-18).
4. The proposal drained all protocol funds into the attacker's wallet.
5. Repaid the flash loan.

This attack demonstrated that flash loans can compromise governance systems that allow instant voting or have insufficient time-lock mechanisms.

**PancakeBunny Price Manipulation (May 2021) -- $45 Million:**

1. Flash-borrowed a massive amount of BNB.
2. Used the BNB to manipulate the price of BUNNY token on PancakeSwap.
3. Exploited the protocol's flawed price oracle (which read directly from the DEX spot price).
4. Minted an enormous amount of BUNNY at the manipulated price.
5. Dumped the BUNNY tokens, crashing the price from $146 to $6.17.
6. Repaid the flash loan.

---

## 7. Major Protocols

### 7.1 Aave

**Overview:** Aave (from the Finnish word for "ghost") is the largest and most widely adopted lending protocol in DeFi. As of mid-2025, Aave's TVL exceeded $40 billion, up from $8 billion in early 2024. It is deployed across 10+ chains including Ethereum, Polygon, Arbitrum, Optimism, Avalanche, Base, and more.

**Key Technical Features:**

- **Multi-chain deployment**: Same protocol logic deployed across multiple L1s and L2s with unified governance.
- **aToken model**: Interest-bearing tokens that maintain a 1:1 peg with the underlying, distributing interest via continuous balance increases.
- **Flash Loans**: Aave pioneered flash loans in DeFi. V3 charges a 0.05% fee on flash loans (0.09% in V2).
- **Efficiency Mode (E-Mode)**: Up to 97% LTV for correlated asset pairs (e.g., stablecoin-to-stablecoin).
- **Isolation Mode**: New assets can be listed with strict caps -- they can only be used as collateral for borrowing approved stablecoins, with a maximum debt ceiling.
- **Portal**: Cross-chain liquidity transfer by burning aTokens on one chain and minting on another.
- **GHO Stablecoin**: Aave's native overcollateralized stablecoin, minted by Aave borrowers.
- **Governance**: AAVE token holders govern the protocol through on-chain voting with a time-lock mechanism. Changes go through Aave Improvement Proposals (AIPs).

**Interest Rate Model (V3):**
Aave V3 uses the kinked rate model described in Section 3, with per-asset parameters determined by risk managers (Gauntlet, Chaos Labs). The protocol also introduced an adaptive interest rate mechanism that adjusts the optimal utilization point based on market conditions.

### 7.2 Compound

**Overview:** Compound Finance was the protocol that essentially invented the autonomous lending pool model in 2018. While Aave has surpassed it in TVL, Compound's design patterns (cTokens, utilization-based rates, governance minimalism) have been forked hundreds of times and form the intellectual foundation of DeFi lending.

**V2 Architecture (the foundational model):**

- **cToken Model**: Each market is a separate cToken contract (cETH, cUSDC, cDAI, etc.). Supplying assets mints cTokens; redeeming cTokens returns underlying + interest.
- **Exchange Rate**: Starts at 0.02 and monotonically increases as interest accrues.
- **Comptroller**: Central contract that manages risk parameters (collateral factors, close factors) and validates borrow/liquidation actions.
- **Interest Rate Models**: Pluggable contracts; each market can have a different model. Most use the Jump Rate Model (kinked model).
- **COMP Governance**: COMP token holders vote on parameter changes, new asset listings, and protocol upgrades.

```
cToken Exchange Rate = (Underlying Balance + Total Borrows - Reserves) / Total cToken Supply

Supply Rate = Borrow Rate * Utilization * (1 - Reserve Factor)

Utilization = Total Borrows / (Cash + Total Borrows - Reserves)
```

**V3 (Comet) Architecture:**

Compound V3 represents a radical simplification:

- **Single borrowable asset per market**: Each Comet deployment (e.g., cUSDCv3) only allows borrowing one asset (e.g., USDC). Multiple assets can be supplied as collateral, but they cannot be borrowed.
- **No rehypothecation**: Supplied collateral is not lent out -- it stays in the contract until withdrawal or liquidation. This eliminates the risk of one bad collateral draining the entire pool.
- **Separate rate curves**: Supply and borrow rates have independent curves, giving governance more control.
- **Single contract**: Each market is one monolithic contract (vs. V2's multi-contract architecture), simplifying security audits and reducing attack surface.
- **Chainlink-only oracles**: V3 exclusively uses Chainlink price feeds, eliminating the governance surface for oracle management.

### 7.3 MakerDAO / Sky (CDP Model)

**Overview:** MakerDAO (now rebranded as **Sky**) is fundamentally different from Aave and Compound. Instead of lending existing assets, MakerDAO *creates* new assets. Borrowers lock collateral and **mint DAI** (now USDS), a decentralized stablecoin. This is the **Collateralized Debt Position (CDP)** model, now called "Vaults."

**How It Works:**

1. User deposits collateral (ETH, WBTC, stablecoins, real-world assets) into a Maker Vault.
2. The user mints DAI up to the maximum allowed by the collateralization ratio (minimum 150% for ETH).
3. DAI is a new token created by the protocol -- it does not come from a pool of existing DAI.
4. The user pays a **Stability Fee** (continuously compounding interest on the minted DAI).
5. To close the vault, the user repays the DAI (which is burned) plus the stability fee.
6. The collateral is unlocked and returned.

**Key Mechanism Differences from Pool-Based Lending:**

| Feature | Maker (CDP) | Aave/Compound (Pool) |
|---------|-------------|---------------------|
| What you borrow | Newly minted DAI | Existing assets from lender pool |
| Interest paid to | Protocol treasury (burned MKR/SKY) | Lenders in the pool |
| Supply side | No "lenders" -- DAI is created | Lenders deposit and earn interest |
| Collateral usage | Locked, not lent out | Can be lent to other borrowers |

**Stability Fee:** Continuously compounding at a per-second rate set by governance:

```
Accumulated Debt = Principal * (Stability Fee Rate) ^ (seconds elapsed)
```

For example, at a 2% annual stability fee:
```
Per-second rate = 1.0000000006279371924910298109948
After 1 year: Principal * 1.02 (exactly 2% growth)
```

**Dai Savings Rate (DSR):** MakerDAO allows DAI holders to deposit DAI into the DSR contract and earn interest. This is funded by the stability fees collected from vault owners. The DSR is a governance-controlled rate that serves as a monetary policy tool:
- Raising the DSR increases DAI demand (people deposit DAI to earn yield), supporting the peg.
- Lowering the DSR decreases DAI demand, useful if DAI is trading above $1.

**Liquidation System (Liquidations 2.0):**
MakerDAO uses a Dutch auction system for liquidations. When a vault falls below its minimum collateralization ratio:
1. The collateral is put up for auction starting at a high price.
2. The price decreases over time until a buyer steps in.
3. The buyer pays DAI (which is burned) and receives the collateral.
4. A 13% liquidation penalty is applied (for ETH vaults).

**Sky Rebrand:** In 2024, MakerDAO began transitioning to the "Sky" brand, introducing USDS (replacing DAI) and SKY tokens (replacing MKR). The core CDP mechanism remains the same.

### 7.4 Solana Lending Protocols

Solana's lending ecosystem reached ~$3.6B TVL by December 2025, representing 33% year-over-year growth. The architecture differs from Ethereum-based protocols in several important ways due to Solana's account model and high throughput.

#### Kamino Lend

**Overview:** Kamino Finance is the dominant lending protocol on Solana, controlling ~75% of Solana's lending market with ~$3.6B TVL as of late 2025. Originally a concentrated liquidity management platform, Kamino expanded into lending with K-Lend and subsequently K-Lend V2.

**Key Features:**

- **Modular Market System**: V2 allows permissionless creation of lending markets with custom parameters (any asset pair, tailored interest curves, individual risk oracles). This enables isolated risk markets without requiring governance approval for each new asset.
- **Automated Lending Vaults**: Single-asset vaults that automatically allocate deposits across multiple lending markets to optimize yield.
- **Multiply (Leverage)**: A product that creates leveraged positions by repeatedly depositing collateral and borrowing against it. Supply SOL as collateral, borrow USDC, buy more SOL, deposit it, borrow more USDC -- automated into a single transaction.
- **Scam Wick Protection**: Mechanism to protect against short-lived price manipulation (oracle "scam wicks") that would trigger unwarranted liquidations.
- **Auction-Based Liquidation**: V2 introduced a Dutch auction system for liquidations, creating competitive dynamics that benefit borrowers (lower penalties) while ensuring timely execution.
- **Collateral Rehypothecation**: Deposited collateral earns yield even while being used as collateral.

**Architecture:**
Kamino is built using Solana's account model. Each lending market is a collection of Solana accounts (market state, reserve accounts per asset, obligation accounts per user). The protocol uses on-chain programs (smart contracts on Solana) written in Rust.

#### MarginFi

**Overview:** MarginFi is a cross-margin lending protocol on Solana that differentiates through sophisticated risk management. It features interconnected asset markets, isolated pools for higher-risk tokens, and dedicated liquid staking collateral options.

**Key Features:**

- **Multi-Market Structure**:
  - **Global Market**: Main lending pool with interconnected assets; collateral in one asset can back borrows in another.
  - **Isolated Markets**: Separate pools with independent risk parameters for riskier tokens.
  - **Native Stake Market**: Specialized market for liquid staking collateral (mSOL, jitoSOL, bSOL).

- **Risk Engine**: MarginFi's proprietary risk engine evaluates three primary factors:
  1. **Liquidator Execution Capacity**: Can liquidators actually execute in time?
  2. **Market Depth**: Is there sufficient on-chain liquidity to absorb liquidation sales?
  3. **Market Depth Recovery Time**: How quickly does liquidity return after a large sale?

  The engine uses these to set dynamic LTV ratios per asset:
  - Conservative for stables: ~90% LTV for USDC
  - Aggressive for volatiles: ~50% LTV for SOL

- **Oracle Redundancy**: Aggregates prices from eight independent oracle providers, eliminating single points of failure. Uses Pyth Network as the primary oracle with Switchboard as a fallback.

- **Single Health Factor**: All positions across markets are aggregated into one health factor for real-time monitoring.

**2024 Controversy:** In April 2024, MarginFi faced internal turmoil when concerns about points distribution and team management caused significant outflows. Users withdrew over $200M in deposits, shaking confidence. Kamino capitalized on this to solidify its lead in Solana lending.

#### Save (formerly Solend)

**Overview:** Solend was the first major lending protocol on Solana, launching in August 2021. It was rebranded to "Save" in 2024. Despite being the OG of Solana lending, Save has struggled to capture recent growth, maintaining ~$300M TVL while competitors like Kamino and Jupiter Lend rapidly gained share.

**Key Features:**
- Straightforward algorithmic pool model similar to Aave's approach.
- Uses Pyth Network and Switchboard oracle feeds for price data and liquidation triggers.
- Multiple pool types: main pool with major assets, isolated pools for experimental assets.

**Notable Incidents:**

- **June 2022 Whale Crisis**: A single whale had deposited $108M SOL as collateral with a large borrow position. As SOL price declined, the position neared liquidation. Liquidating 95% of the SOL in the main pool on-chain would have overwhelmed Solana's DEX liquidity, potentially crashing SOL's price and cascading into further liquidations. Solend's team controversially proposed (and passed via governance) a proposal to take over the whale's account to enable an OTC liquidation -- a move that was widely criticized as antithetical to DeFi's permissionless ethos. The proposal was later reversed.

- **November 2022 Oracle Exploit**: Solend suffered an oracle exploit resulting in $1.26M of bad debt.

#### Other Notable Solana Lending Protocols

- **Jupiter Lend**: Built by Jupiter (Solana's leading DEX aggregator), leveraging its existing user base and liquidity routing. Growing rapidly.
- **Drift Protocol**: Primarily a perpetuals DEX, but also offers lending/borrowing as part of its cross-margin trading system.

---

## 8. Risks in Lending Protocols

### 8.1 Oracle Manipulation

Lending protocols depend entirely on price oracles to determine collateral values, health factors, and liquidation eligibility. Oracle manipulation is one of the most dangerous attack vectors.

**How Oracle Manipulation Works:**

1. **Spot price manipulation**: Attacker uses a large trade (often flash-loaned) to temporarily move the price on a DEX. If the protocol's oracle reads this manipulated spot price, the attacker can borrow more than they should or trigger unjustified liquidations.

2. **Oracle feed compromise**: The oracle itself is compromised or returns incorrect data. This happened with Moonwell in February 2026, where a misconfigured Coinbase oracle briefly valued cbETH at $1 (instead of ~$2,200), triggering $1.78M in bad debt from erroneous liquidations.

3. **Stale oracle data**: During network congestion, oracle updates may be delayed. Prices could move significantly between updates, creating windows for exploitation.

**Mango Markets Oracle Manipulation (October 2022) -- $117 Million:**

The most prominent oracle manipulation attack on Solana:

1. Avraham Eisenberg funded two accounts on Mango Markets.
2. Account A shorted 488M MNGO perpetual futures; Account B went long.
3. Eisenberg then spent $4M buying MNGO on three exchanges, pumping the oracle price by 2,300%.
4. Account B now showed ~$400M in paper profits (unrealized PnL backed by the inflated oracle price).
5. Eisenberg borrowed against this inflated position value, draining virtually all assets from Mango Markets.
6. The manipulation was not a code exploit -- the protocol functioned as designed. The vulnerability was reliance on easily-manipulated low-liquidity price feeds.
7. Eisenberg was later arrested, charged, and convicted of commodities fraud, manipulation, and wire fraud.

**Mitigations:**

- **Time-Weighted Average Prices (TWAP)**: Average prices over a window (e.g., 30 minutes) rather than reading spot prices. This makes manipulation far more expensive since the attacker must sustain the price distortion.
- **Multi-oracle aggregation**: Use multiple independent oracles (Chainlink, Pyth, Switchboard, Band, Uniswap TWAP) and take the median or detect outliers.
- **Circuit breakers**: Reject price updates that deviate more than X% from the previous known price within a short window.
- **Delayed price impact**: Wait N blocks before reflecting new prices in protocol logic.

### 8.2 Bad Debt Scenarios

**Bad debt** occurs when a borrower's collateral is insufficient to cover their outstanding debt, and no liquidator can profitably resolve the position. The protocol (and by extension, its lenders) absorbs the loss.

**How bad debt accumulates:**

1. **Rapid price crash**: Collateral drops so fast that liquidators cannot act before the position becomes underwater (collateral < debt).
2. **Network congestion**: On Ethereum, gas spikes during crashes can make liquidation transactions prohibitively expensive. On Solana, compute unit limits and transaction failures can delay liquidations.
3. **Thin liquidity**: For small-cap collateral, even moderate liquidation volumes can overwhelm on-chain liquidity, resulting in massive slippage. The liquidator receives collateral they cannot sell at a reasonable price.
4. **Oracle failure**: Bad oracle data leads to either missed liquidations (protocol thinks position is healthy) or incorrect liquidations.

**Protocol defenses against bad debt:**

- **Reserve Factor**: A percentage of all interest income is set aside in protocol reserves to cover potential bad debt.
- **Safety Module / Insurance Fund**: Aave has a Safety Module where stakers deposit AAVE tokens; in the event of a shortfall, up to 30% of staked AAVE can be slashed to cover bad debt.
- **Treasury Backstop**: Protocol treasuries (from token sales, fee revenue) serve as a last line of defense.
- **Debt ceilings**: Limits on total borrowing against any single collateral type, preventing concentrated risk.

### 8.3 Smart Contract Risk

All DeFi protocols are ultimately code running on a blockchain. Bugs, vulnerabilities, or unforeseen interactions between contracts can lead to catastrophic fund loss.

**Common vulnerability patterns:**

**Reentrancy Attacks:**
A function makes an external call before updating its own state. The called contract calls back into the original function, executing it again with stale state. The 2016 DAO hack ($50M) was a reentrancy attack. More recently, in 2020, Uniswap V1 lost 1,278 ETH to reentrancy.

```
// VULNERABLE: State updated after external call
function withdraw(uint amount) {
    require(balances[msg.sender] >= amount);
    msg.sender.call{value: amount}("");  // External call
    balances[msg.sender] -= amount;       // State update (too late!)
}

// SAFE: State updated before external call (Checks-Effects-Interactions)
function withdraw(uint amount) {
    require(balances[msg.sender] >= amount);  // Check
    balances[msg.sender] -= amount;           // Effect
    msg.sender.call{value: amount}("");       // Interaction
}
```

**Note on Solana:** Reentrancy attacks work differently on Solana because the Solana runtime prevents a program from being invoked again while it is already on the call stack (re-entrancy guard is built into the runtime). However, cross-program invocation (CPI) patterns can still create analogous vulnerabilities if state is not properly managed.

**Logic Errors:**
The Euler Finance hack ($197M) was a logic error: the `donateToReserves` function did not check whether the account remained solvent after the donation, allowing an attacker to artificially create bad debt and exploit the liquidation mechanism.

**Upgrade Risks:**
Upgradeable proxy contracts introduce the risk that a malicious or compromised admin could deploy a new implementation that drains funds. This is why many protocols use timelocks, multisigs, and eventually transition to immutable contracts.

**Mitigation:**

- Multiple independent security audits (Trail of Bits, OpenZeppelin, Halborn, OtterSec, etc.)
- Formal verification of critical invariants
- Bug bounty programs (Aave offers up to $250K; some protocols offer millions)
- Gradual parameter changes with timelocks
- Open-source code with community review

### 8.4 Governance Attacks

Governance tokens grant voting power over protocol parameters. If an attacker acquires sufficient voting power, they can pass malicious proposals.

**Beanstalk Farms ($182M, April 2022):**
As detailed in Section 6.4, an attacker used a flash loan to borrow governance tokens, pass a malicious proposal, and drain the protocol -- all in a single transaction. The vulnerability: Beanstalk allowed instant governance execution without a time-lock.

**Attack vectors:**

1. **Flash loan governance**: Borrow governance tokens, vote, repay in one transaction. Mitigated by requiring token snapshots at proposal creation time rather than at voting time.
2. **Gradual accumulation**: Quietly accumulate governance tokens over time, then pass a harmful proposal. Mitigated by active community monitoring and time-lock delays.
3. **Social engineering**: Convince token holders to vote for proposals with hidden malicious effects.
4. **Low quorum exploitation**: In protocols with low voter turnout, a small number of tokens can pass proposals. Mitigated by setting adequate quorum requirements.

**Mitigations:**

- **Time-locks**: All governance actions have a mandatory delay (24-72 hours) before execution, allowing the community to review and potentially cancel malicious proposals.
- **Snapshot voting**: Voting power is determined by token holdings at a past block (snapshot), not at the time of voting, preventing flash loan governance attacks.
- **Guardian / Emergency multisig**: A multisig of trusted community members can cancel malicious proposals during the time-lock period.
- **Delegation**: Token holders delegate to informed delegates who actively participate in governance.
- **Minimum proposal thresholds**: Requiring a significant token balance to create proposals, preventing spam.

---

## 9. References

### Protocol Documentation
- [Aave V3 Documentation](https://aave.com/docs/aave-v3/overview)
- [Aave Interest Rate Strategy](https://aave.com/docs/developers/smart-contracts/interest-rate-strategy)
- [Aave Borrow Interest Rate Risk Docs](https://docs.aave.com/risk/liquidity-risk/borrow-interest-rate)
- [Aave V3 Features FAQ](https://docs.aave.com/faq/aave-v3-features)
- [Aave Liquidations FAQ](https://docs.aave.com/faq/liquidations)
- [Aave Flash Loans Documentation](https://aave.com/docs/aave-v3/guides/flash-loans)
- [Compound V2 Documentation](https://docs.compound.finance/v2/)
- [Compound V2 cTokens Docs](https://docs.compound.finance/v2/ctokens/)
- [Compound V3 (Comet) Documentation](https://docs.compound.finance/)
- [Compound V3 Interest Rates](https://docs.compound.finance/interest-rates/)
- [Compound Whitepaper](https://compound.finance/documents/Compound.Whitepaper.pdf)
- [MakerDAO Whitepaper](https://makerdao.com/whitepaper/White%20Paper%20-The%20Maker%20Protocol_%20MakerDAO%E2%80%99s%20Multi-Collateral%20Dai%20(MCD)%20System-FINAL-%20021720.pdf)
- [MakerDAO Rates Module](https://docs.makerdao.com/smart-contract-modules/rates-module)
- [MarginFi Documentation](https://docs.marginfi.com/)
- [Kamino Lend V2 Introduction](https://blog.kamino.finance/introducing-kamino-lend-v2-08ad8f52855c)

### Technical Analysis and Explainers
- [RareSkills: Aave V3 and Compound V2 Interest Rate Models](https://rareskills.io/post/aave-interest-rate-model)
- [RareSkills: DeFi Liquidations and Collateral](https://rareskills.io/post/defi-liquidations-collateral)
- [RareSkills: Compound V3 Architecture](https://rareskills.io/post/compound-v3-contracts-tutorial)
- [Krayon Digital: Aave Interest Rate Model Explained](https://www.krayondigital.com/blog/aave-interest-rate-model-explained)
- [Ian Macalinao: Understanding Compound's Interest Rates](https://ianm.com/posts/2020-12-20-understanding-compound-protocols-interest-rates)
- [MixBytes: How Liquidations Work in DeFi](https://mixbytes.io/blog/how-liquidations-work-in-defi-a-deep-dive)
- [Cyfrin: Aave V3 Improved Lending, Liquidity, and Risk Management](https://www.cyfrin.io/blog/aave-v3-improved-lending-liquidity-and-risk-management)
- [Cyfrin: DeFi Liquidation Risks and Vulnerabilities](https://www.cyfrin.io/blog/defi-liquidation-vulnerabilities-and-mitigation-strategies)
- [Finematics: Lending and Borrowing in DeFi Explained](https://finematics.com/lending-and-borrowing-in-defi-explained/)
- [LlamaRisk: Aave IRM and TradFi Symbiosis](https://www.llamarisk.com/research/aave-irm)

### Solana Ecosystem
- [RedStone Blog: Solana Lending Markets Report 2025](https://blog.redstone.finance/2025/12/11/solana-lending-markets/)
- [Jito Foundation: Lending Markets on Solana](https://www.jito.network/blog/lending-markets-on-solana/)
- [Backpack: Best Solana Lending Protocols 2025](https://learn.backpack.exchange/articles/best-solana-lending-protocols)
- [Nansen: What is MarginFi?](https://www.nansen.ai/post/what-is-marginfi)
- [CoinDesk: Chaos at MarginFi](https://www.coindesk.com/business/2024/04/11/chaos-at-marginfi-shakes-up-solana-defis-borrow-and-lend-landscape)
- [Kamino Solana DeFi Deep Dive (Q3 2025)](https://medium.com/@Scoper/solana-defi-deep-dives-kamino-late-2025-080f6f52fa29)
- [OnchainTimes: Maximizing Yield with Kamino Multiply](https://www.onchaintimes.com/maximizing-yield-on-solana-with-kamino-multiply/)

### Security and Attacks
- [Chainalysis: Euler Finance Flash Loan Attack Explained](https://www.chainalysis.com/blog/euler-finance-flash-loan-attack/)
- [Cyfrin: Euler Finance Hack Analysis](https://www.cyfrin.io/blog/how-did-the-euler-finance-hack-happen-hack-analysis)
- [Chainalysis: Oracle Manipulation Attacks Rising](https://www.chainalysis.com/blog/oracle-manipulation-attacks-rising/)
- [Solidus Labs: Mango Markets Exploit Order Book Analysis](https://www.soliduslabs.com/post/mango-hack)
- [OWASP: SC07:2025 Flash Loan Attacks](https://owasp.org/www-project-smart-contract-top-10/2025/en/src/SC07-flash-loan-attacks.html)
- [Hacken: Flash Loan Attacks Risks and Prevention](https://hacken.io/discover/flash-loan-attacks/)
- [CertIK: Oracle Wars - The Rise of Price Manipulation Attacks](https://www.certik.com/resources/blog/oracle-wars-the-rise-of-price-manipulation-attacks)
- [Koinly: 5 Biggest Flash Loan Attacks](https://koinly.io/blog/biggest-flash-loan-attacks-stats/)
- [ImmuneBytes: List of Flash Loan Attacks in Crypto](https://immunebytes.com/blog/list-of-flash-loan-attacks-in-crypto/)
- [Decrypt: Moonwell Oracle Bad Debt ($1.8M)](https://decrypt.co/358374/oracle-error-leaves-defi-lender-moonwell-1-8-million-bad-debt)

### Risk and Research
- [Amberdata: Overcollateralization for Institutional DeFi Lenders](https://blog.amberdata.io/overcollateralization-what-institutional-defi-lenders-need-to-know)
- [GARP: Risk Management in DeFi Lending and Borrowing](https://www.garp.org/hubfs/Whitepapers/a2r5d0000065tecAAA_RiskIntel.Whitepaper.DeFi.2.9.23.pdf)
- [BIS: DeFi Lending - Intermediation Without Information](https://www.bis.org/publ/bisbull57.pdf)
- [ChainRisk: DeFi Lending and Borrowing Risk Framework](https://www.chainrisk.xyz/blog-posts/defi-lending-borrowing-risk-framework)
- [An Empirical Study of DeFi Liquidations (arXiv)](https://arxiv.org/pdf/2106.06389)
