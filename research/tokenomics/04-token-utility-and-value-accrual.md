# Token Utility and Value Accrual

> Written for experienced Solana developers entering token design.
> Last updated: February 2026

---

## Table of Contents

1. [The Value Accrual Problem](#1-the-value-accrual-problem)
2. [Taxonomy of Token Utility](#2-taxonomy-of-token-utility)
3. [Fee Switches and Revenue Distribution](#3-fee-switches-and-revenue-distribution)
4. [Buyback Mechanisms](#4-buyback-mechanisms)
5. [Staking as Value Accrual](#5-staking-as-value-accrual)
6. [Access and Membership Tokens](#6-access-and-membership-tokens)
7. [Collateral and Work Tokens](#7-collateral-and-work-tokens)
8. [The Demand Stack](#8-the-demand-stack)
9. [Value Accrual Math](#9-value-accrual-math)
10. [Anti-Patterns: Fake Utility](#10-anti-patterns-fake-utility)
11. [Program-Level Implementation](#11-program-level-implementation)
12. [References](#12-references)

---

## 1. The Value Accrual Problem

The central question of tokenomics: **Why should anyone hold this token instead of selling it?**

Every token exists on a spectrum from "pure speculation" to "essential infrastructure asset." The closer a token is to the "essential" end, the more robust its value proposition:

```
Pure Speculation ◄──────────────────────────────────────► Essential Infrastructure
    MEME coins         Governance-only tokens         ETH/SOL (gas fees)
    No intrinsic        Value from voting              Required for every
    demand floor        rights alone                   transaction
```

### The Fundamental Equation

```
Token Value = Σ(Utility Value) + Σ(Cash Flow Value) + Speculative Premium

Where:
  Utility Value:    What you can DO with the token (access, fees, governance)
  Cash Flow Value:  What the token PAYS you (staking, revenue share, burns)
  Speculative Premium: What people HOPE it will be worth (narrative, momentum)
```

For long-term sustainability, Utility Value + Cash Flow Value must be positive. Speculative premium alone is not sustainable.

### The Token Premium Question

A protocol generates revenue. A token exists. The question: **How much of the protocol's revenue should flow to token holders?**

| Revenue Destination | Token Impact | Example |
|---|---|---|
| Treasury (no distribution) | No direct value accrual | Early Uniswap (no fee switch) |
| Token buyback + burn | Indirect (scarcity) | MakerDAO surplus auctions |
| Direct distribution to stakers | Direct cash flow | GMX 30% fee share |
| Subsidized fees for holders | Indirect (usage discount) | BNB exchange fee discount |
| Protocol-owned liquidity | Indirect (stability) | Olympus bonding |

---

## 2. Taxonomy of Token Utility

### Category 1: Governance Rights

The token grants voting power over protocol decisions.

**Minimal governance** (weak utility):
- Vote on minor parameters (fee tiers, asset listings)
- No treasury control
- No revenue distribution
- Example: Early UNI — governance over a protocol with no fee switch

**Meaningful governance** (moderate utility):
- Control over treasury allocation
- Fee switch activation
- Protocol upgrade authority
- Emergency powers
- Example: MKR — governs stability fees, collateral types, risk parameters, and surplus distribution

**Full sovereignty** (strong utility):
- All of the above, plus:
- Controls the token's own supply mechanics
- Can modify the tokenomics itself
- Example: Jupiter DAO — voted to burn 3B JUP, governing emission and supply

### Category 2: Economic Rights

The token entitles the holder to a share of protocol economics.

| Right | Mechanism | Example |
|---|---|---|
| **Fee sharing** | Protocol fees distributed pro-rata to stakers | GMX: 30% of trading fees to stakers |
| **Fee discount** | Holding/staking reduces protocol fees | BNB: lower Binance trading fees |
| **Boosted yield** | Staking increases LP rewards | CRV: up to 2.5x boost for veCRV holders |
| **Priority access** | Token holders get early access to features | Raydium AcceleRaytor: RAY stakers get IDO allocation |
| **Insurance** | Token used as backstop for protocol insolvency | AAVE Safety Module: staked AAVE covers bad debt |

### Category 3: Functional Requirement

The token must be spent or staked to use the protocol.

| Function | Mechanism | Example |
|---|---|---|
| **Gas/execution** | Required for every transaction | ETH, SOL |
| **Oracle payment** | Data consumers pay in token | LINK (Chainlink) |
| **Storage payment** | Users pay for decentralized storage | FIL (Filecoin), AR (Arweave) |
| **Compute payment** | Users pay for decentralized compute | RENDER |
| **Network registration** | Must stake to be a service provider | GRT (Graph), HNT (Helium) |

### Category 4: Collateral Utility

The token is accepted as collateral in DeFi protocols.

```
ETH as collateral:
  - Aave: borrow USDC against ETH
  - MakerDAO: mint DAI against ETH
  - Compound: borrow against ETH

SOL as collateral:
  - Kamino: borrow USDC against SOL
  - Drift: margin trading with SOL collateral
```

Collateral demand creates a price floor — as long as borrowing demand exists, there is demand to hold the token as collateral. This is one of the strongest demand drivers in DeFi.

---

## 3. Fee Switches and Revenue Distribution

### What Is a Fee Switch?

A fee switch is a governance mechanism that activates or modifies how protocol revenue flows to token holders. "Turning on the fee switch" typically means directing a portion of protocol fees to token stakers or the treasury.

### The Uniswap Fee Switch Saga

Uniswap is the most famous fee switch case:

```
Uniswap V2/V3 Fee Structure:
  - LPs earn 100% of trading fees (0.30% per swap)
  - UNI holders earn 0% of trading fees
  - The protocol has a "fee switch" that could redirect 1/6 of LP fees
  - For V2: LP fee goes from 0.30% to 0.25%, with 0.05% to protocol
  - For V3: Each pool can be set independently

Status: The fee switch has been discussed many times but as of 2026 has not
been fully activated at protocol level, partly due to regulatory concerns
(turning it on might make UNI look like a security).

Uniswap Labs charges a 0.15% interface fee on certain pairs through the
frontend, but this goes to Uniswap Labs (the company), not UNI holders.
```

**The fee switch dilemma**:
- Turning it on: Generates revenue for token holders, creates cash flow valuation, but reduces LP incentives and might attract regulatory scrutiny
- Keeping it off: LPs get 100% of fees (more attractive), but UNI has no direct value accrual beyond governance

### Revenue Distribution Models

**Model 1: Direct Distribution (GMX-style)**

```
Protocol earns fees → Convert to ETH/USDC → Distribute to stakers pro-rata

GMX mechanics:
  - GMX stakers earn 30% of platform trading fees (in ETH/AVAX)
  - esGMX (escrowed GMX) stakers earn the same
  - Multiplier Points boost yield for long-term stakers

Weekly distribution example:
  Protocol weekly fees: $5M
  GMX staker share (30%): $1.5M
  Total staked GMX: 7M tokens
  Revenue per staked GMX: $0.214/week = $11.14/year

  If GMX price = $50:
  Yield = $11.14 / $50 = 22.3% APR (real yield, not emissions)
```

**Model 2: Buyback and Burn (MKR-style)**

```
Protocol earns fees → Surplus above buffer → Buy MKR on DEX → Burn

MakerDAO mechanics:
  - DAI stability fees accumulate in surplus buffer
  - When surplus > threshold: excess buys MKR via Flap Auction
  - Bought MKR is burned → supply decreases
  - MKR holders benefit indirectly through increased scarcity

Annualized (example):
  Annual stability fees: $100M
  Less expenses/bad debt: $30M
  Net surplus for buybacks: $70M
  MKR market cap: $2B
  Annual burn yield: 3.5% (equivalent to a stock buyback yield)
```

**Model 3: Buyback and Redistribute (SUSHI-style)**

```
Protocol earns fees → Buy SUSHI on market → Distribute to xSUSHI stakers

SushiSwap mechanics:
  - 0.05% of every swap goes to xSUSHI stakers (1/6 of total 0.30% fee)
  - Users stake SUSHI → receive xSUSHI
  - xSUSHI/SUSHI exchange rate increases over time
  - Stakers earn yield in SUSHI (compounding)
```

**Model 4: ve-Token Revenue Lock (Curve-style)**

```
Protocol earns fees → Distribute to veCRV holders proportionally

Curve mechanics:
  - 50% of trading fees go to veCRV holders
  - Must lock CRV for 1-4 years to receive veCRV
  - Longer lock = more veCRV = larger fee share
  - Fees paid in 3CRV (LP token) — real yield

Key: Revenue only flows to LOCKED tokens.
This ensures only committed holders receive fees.
```

---

## 4. Buyback Mechanisms

### Simple Buyback and Burn

```
Revenue collection → DEX swap (TOKEN → burn address)

Implementation considerations:
  1. How often? (daily, weekly, on-demand)
  2. Through which DEX? (protocol's own pool? Jupiter aggregation?)
  3. Slippage protection? (max slippage, TWAP orders)
  4. Transparent? (on-chain, auditable)
```

### TWAP (Time-Weighted Average Price) Buybacks

Instead of a single large market buy (which would move the price), spread the buyback over time:

```
Total buyback: $1M of TOKEN per week
TWAP execution: Buy $6,000 worth every hour for 168 hours
Result: Average execution price, minimal market impact
```

**Solana implementation options**:
- Jupiter DCA (Dollar Cost Averaging) orders
- Custom program with time-based execution
- Keeper/crank system that executes periodic swaps

### Buyback Efficiency

```
Buyback Efficiency = Price Impact / Dollar Spent

Good: $1M buyback moves price 0.5% → ratio = 0.5%/$1M
Bad:  $1M buyback moves price 5% → ratio = 5%/$1M (10x worse)

Factors affecting efficiency:
  - Token liquidity (deeper pools = less impact)
  - Execution strategy (TWAP vs. single swap)
  - Market conditions (low-volume periods are worse)
  - Transparency (announced buybacks get front-run)
```

---

## 5. Staking as Value Accrual

### Types of Staking

| Type | What You Lock | What You Earn | Risk |
|---|---|---|---|
| **Network staking** | Native token (ETH, SOL) | Inflation rewards + tips | Slashing |
| **Protocol staking** | Governance token | Revenue share + emissions | Opportunity cost |
| **Safety module staking** | Governance token | Yield + emissions | Slashing (covers bad debt) |
| **Vote-escrow staking** | Governance token (locked) | Fees + boosted rewards | Illiquidity for lock period |
| **Liquid staking** | Native token | LST (liquid staking token) | Smart contract risk |

### The Staking Yield Decomposition

```
Total Staking APY = Real Yield + Emission Yield

Real Yield = (Protocol Revenue to Stakers) / (Total Value Staked)
Emission Yield = (New Tokens Emitted to Stakers × Token Price) / (Total Value Staked)

Example (GMX):
  Real Yield: 30% of $5M/week in fees → $1.5M/week to stakers
  Staked: 7M GMX at $50 = $350M
  Real yield APR: ($1.5M × 52) / $350M = 22.3%
  Emission yield: esGMX emissions (let's say $5M/year) / $350M = 1.4%
  Total: ~23.7% APR

Example (generic PoS):
  Real Yield: transaction tips = minimal (~0.5%)
  Emission yield: 7% inflation directed to stakers
  Total: ~7.5% APR (but 7% is just token printing)
```

### Safety Module Staking (Aave-style)

AAVE token holders can stake in the Safety Module, providing a backstop for the protocol:

```
Safety Module Mechanics:
  1. User stakes AAVE in Safety Module
  2. Earns staking yield (~8-12% APR from emissions + fees)
  3. If protocol has a shortfall event (bad debt):
     - Up to 30% of staked AAVE can be slashed
     - Slashed AAVE is sold to cover the deficit
  4. This is REAL risk — stakers can lose up to 30%

Value proposition:
  - Protocol gets a decentralized insurance fund
  - Stakers earn yield for providing insurance
  - AAVE becomes a "productive" asset (not just governance)
  - Creates demand sink (AAVE must be bought and staked)
```

---

## 6. Access and Membership Tokens

### Tiered Access Models

Some protocols require holding or staking tokens for access to features:

```
Example: Tiered Access Protocol

  Free tier:         No tokens required, basic features
  Silver tier:       Hold 100 TOKENS → enhanced features
  Gold tier:         Stake 1,000 TOKENS → premium features + fee discount
  Platinum tier:     Stake 10,000 TOKENS → all features + revenue share

Demand creation:
  - Each tier requires MORE tokens
  - Popular protocol → more users want premium → more demand
  - Tokens are locked (staked) → reduced circulating supply
```

### Token-Gated Communities

Holding a token grants access to exclusive communities, information, or services:
- Discord roles based on token holdings
- Alpha trading groups
- Early access to new features
- Priority support

**Demand impact**: Creates a minimum holding requirement. If 10,000 users each need 100 tokens for access, that's 1M tokens in permanent demand.

---

## 7. Collateral and Work Tokens

### Collateral Demand

When a token is accepted as collateral in lending/borrowing protocols, it creates structural demand:

```
Collateral demand loop:
  1. AAVE accepts TOKEN as collateral at 75% LTV
  2. User buys 100 TOKEN, deposits in AAVE
  3. Borrows 75 TOKEN worth of USDC
  4. Uses USDC for other purposes
  5. TOKEN is locked as collateral (cannot sell)

Result: Each borrower locks TOKEN out of circulation
More borrowing demand → more TOKEN locked → less supply on market
```

**Metrics to track**:
```
Collateral utilization = TOKEN deposited as collateral / Total circulating supply

ETH: ~25-30% used as collateral across DeFi
SOL: ~15-20% used as collateral
Most governance tokens: <5%
```

### Work Tokens (Stake to Serve)

```
Operator economics:
  1. Stake 10,000 TOKEN to become an operator
  2. Serve requests from the network
  3. Earn fees for each request served
  4. If you misbehave: slashed (lose staked tokens)

Demand equation:
  Required stake = (Network demand × Security factor) / Token price
  As network demand grows, required stake grows, token demand grows
```

---

## 8. The Demand Stack

A well-designed token has multiple, independent sources of demand. Each source creates a "floor" that supports the others:

```
DEMAND STACK (strongest at bottom):

  ┌─────────────────────────┐
  │  Speculation / Momentum │  ← Least reliable, most volatile
  ├─────────────────────────┤
  │  Governance Voting      │  ← Moderate reliability
  ├─────────────────────────┤
  │  Collateral Demand      │  ← Reliable (tied to borrowing)
  ├─────────────────────────┤
  │  Fee Sharing / Yield    │  ← Reliable (tied to revenue)
  ├─────────────────────────┤
  │  Staking Requirement    │  ← Very reliable (structural lock)
  ├─────────────────────────┤
  │  Gas / Fee Payment      │  ← Most reliable (required for usage)
  └─────────────────────────┘
```

**Examples**:

**ETH demand stack**:
```
Gas payment:         Required for every Ethereum transaction ✓
Staking:             32 ETH to validate, ~25% of supply staked ✓
Collateral:          Most used collateral in DeFi ✓
Fee sharing:         Stakers earn tips + MEV ✓
Fee burn:            EIP-1559 burns base fee ✓
Governance:          (minimal, via social consensus) ✗
Speculation:         "Ultrasound money" narrative ✓

Result: 5/6 demand layers → extremely robust demand profile
```

**UNI demand stack** (pre fee switch):
```
Gas payment:         ✗ (uses ETH)
Staking:             ✗ (no staking mechanism)
Collateral:          Limited ✓
Fee sharing:         ✗ (fee switch off)
Fee burn:            ✗
Governance:          ✓ (controls $3B+ treasury)
Speculation:         ✓ (leading DEX narrative)

Result: 2-3/6 demand layers → weaker demand profile
```

---

## 9. Value Accrual Math

### DCF Valuation for Fee-Generating Tokens

```
Token Value = Σ (Annual Protocol Revenue × Share to Token Holders) / (1 + r)^t

Example: GMX
  Annual trading volume: ~$100B
  Fee rate: 0.1% average
  Annual fees: $100M
  Token holder share: 30%
  Annual cash flow to stakers: $30M
  Discount rate: 25% (high risk, crypto)
  Terminal growth: 3%

  PV of cash flows (10-year DCF + terminal):
  ≈ $30M / (0.25 - 0.03) = $136M (simplified Gordon Growth Model)

  GMX total supply: ~13.5M
  Fair value per GMX: $136M / 13.5M ≈ $10

  Note: This is JUST the cash flow value. Add governance value,
  speculative premium, and growth optionality for full valuation.
```

### P/E and P/S Ratios for Tokens

Like equities, tokens can be valued on multiples:

```
P/E = Token Market Cap / Annual Earnings to Token Holders
P/S = Token FDV / Annual Protocol Revenue

Token          P/E Ratio    P/S Ratio    Revenue (annual)
MKR            ~15x         ~12x         ~$200M
UNI (no fee)   ∞ (no E)    ~8x          ~$500M (LP fees, not to UNI)
AAVE           ~25x         ~20x         ~$100M
CRV            ~30x         ~15x         ~$50M
```

**Interpretation**: Lower P/E = cheaper relative to earnings. But growth rate matters — a high P/E with rapid growth may be a better investment than a low P/E with no growth.

### Burn-Adjusted Supply

For tokens with burn mechanics:

```
Effective Annual Deflation = Annual Burn Amount / Current Supply

MKR example:
  Supply: ~900,000 MKR
  Annual burn: ~10,000 MKR
  Annual deflation: ~1.1%

  At this rate, supply in 10 years: 900,000 × (1 - 0.011)^10 ≈ 800,000 MKR
  Value per token increases by ~12.5% from supply reduction alone
```

---

## 10. Anti-Patterns: Fake Utility

### The "Payment Token" Fallacy

```
Claim: "Users pay for our service in TOKEN"
Reality: Users would prefer to pay in USDC/SOL directly
Problem: Forcing users to buy TOKEN → swap → pay creates friction
         Users buy TOKEN, immediately spend it → high velocity → low value
```

Unless the token provides a genuine discount or exclusive access, requiring it for payment is just adding unnecessary friction. The protocol would grow faster accepting stablecoins.

### The "Governance Only" Trap

```
Claim: "TOKEN gives you governance over the protocol"
Reality: 95% of token holders never vote
Problem: Governance alone doesn't justify paying $50 per token
         No cash flow, no burn, no staking → no price floor
```

Governance is necessary but not sufficient for value accrual. It must be paired with economic rights.

### The "Burn Everything" Illusion

```
Claim: "We burn 5% of every transaction! Deflationary!"
Reality: If the protocol also mints 10% annually in emissions,
         net supply change = +10% - 5% = +5% (still inflationary)
Problem: Burn rate must exceed emission rate to be truly deflationary
```

Always look at NET supply change, not gross burns.

### The "Staking for Staking's Sake" Pattern

```
Claim: "Stake TOKEN to earn 50% APY!"
Reality: The APY comes from inflating TOKEN supply
Problem: Staking APY = inflation → stakers maintain share, non-stakers diluted
         Net real yield to stakers = 0%
         The protocol is printing money and calling it yield
```

Real staking yield comes from protocol revenue, not emissions.

---

## 11. Program-Level Implementation

### Fee Distribution Contract (Solana/Anchor)

```rust
use anchor_lang::prelude::*;
use anchor_spl::token::{self, Token, TokenAccount, Transfer};

/// Stake TOKEN, earn revenue share in USDC
#[account]
pub struct StakePool {
    pub stake_token_mint: Pubkey,    // The governance token
    pub reward_token_mint: Pubkey,   // USDC (or whatever revenue is in)
    pub total_staked: u64,
    pub reward_per_token_stored: u128, // Scaled by 1e18
    pub last_update_time: i64,
    pub reward_rate: u64,            // Rewards per second
    pub reward_duration_end: i64,
    pub authority_bump: u8,
}

#[account]
pub struct UserStake {
    pub owner: Pubkey,
    pub amount: u64,
    pub reward_per_token_paid: u128,
    pub rewards_earned: u64,
}

impl StakePool {
    /// Calculate reward per token up to current time
    pub fn reward_per_token(&self, current_time: i64) -> u128 {
        if self.total_staked == 0 {
            return self.reward_per_token_stored;
        }

        let time_end = current_time.min(self.reward_duration_end);
        let elapsed = (time_end - self.last_update_time).max(0) as u128;
        let reward_accrued = elapsed * self.reward_rate as u128;

        // Scale by 1e18 for precision
        self.reward_per_token_stored
            + (reward_accrued * 1_000_000_000_000_000_000u128)
                / self.total_staked as u128
    }

    /// Calculate earned rewards for a user
    pub fn earned(&self, user: &UserStake, current_time: i64) -> u64 {
        let rpt = self.reward_per_token(current_time);
        let delta = rpt - user.reward_per_token_paid;

        let earned = (user.amount as u128 * delta)
            / 1_000_000_000_000_000_000u128;

        user.rewards_earned + earned as u64
    }
}
```

### Buyback and Burn Program

```rust
/// Admin triggers a buyback using protocol revenue
pub fn execute_buyback(ctx: Context<ExecuteBuyback>, usdc_amount: u64) -> Result<()> {
    // 1. Swap USDC for TOKEN via Jupiter/DEX CPI
    //    (Simplified — real implementation uses Jupiter CPI)
    let tokens_received = swap_usdc_for_token(
        &ctx.accounts.usdc_vault,
        &ctx.accounts.token_vault,
        usdc_amount,
        ctx.accounts.minimum_tokens_out,
    )?;

    // 2. Burn the received tokens
    token::burn(
        CpiContext::new_with_signer(
            ctx.accounts.token_program.to_account_info(),
            token::Burn {
                mint: ctx.accounts.token_mint.to_account_info(),
                from: ctx.accounts.token_vault.to_account_info(),
                authority: ctx.accounts.buyback_authority.to_account_info(),
            },
            &signer_seeds,
        ),
        tokens_received,
    )?;

    emit!(BuybackEvent {
        usdc_spent: usdc_amount,
        tokens_burned: tokens_received,
        timestamp: Clock::get()?.unix_timestamp,
    });

    Ok(())
}
```

### Token-2022 Transfer Fee as Revenue

```rust
// Create a Token-2022 mint with transfer fee
// Every transfer automatically collects a fee

// At mint creation:
let transfer_fee_config = TransferFeeConfig {
    transfer_fee_config_authority: Some(admin.key()),
    withdraw_withheld_authority: Some(protocol_treasury.key()),
    // 50 basis points = 0.5% fee on every transfer
    newer_transfer_fee: TransferFee {
        epoch: 0,
        maximum_fee: u64::MAX,
        transfer_fee_basis_points: 50, // 0.5%
    },
    older_transfer_fee: TransferFee {
        epoch: 0,
        maximum_fee: u64::MAX,
        transfer_fee_basis_points: 50,
    },
};

// The protocol can periodically harvest withheld fees:
// spl_token_2022::instruction::harvest_withheld_tokens_to_mint
// spl_token_2022::instruction::withdraw_withheld_tokens_from_mint
```

---

## 12. References

1. **"Fat Protocols" — Joel Monegro (USV, 2016)**: Where value accrues in protocol stacks
2. **GMX documentation**: Real yield revenue sharing model
3. **MakerDAO governance docs**: Surplus auctions and MKR burn mechanics
4. **Uniswap fee switch proposals**: Governance forum discussions
5. **"The Ownership Economy" — Jesse Walden (Variant, 2020)**: Token holder economics
6. **"Token Value Flows" — Delphi Digital**: Framework for analyzing value accrual
7. **AAVE Safety Module documentation**: Insurance staking mechanics
8. **Token Terminal**: Protocol revenue and earnings data

---

*Next: [05 - Staking, Vote-Escrow, and Incentive Design](./05-staking-and-vote-escrow.md) — Deep dive into PoS staking economics, ve-tokenomics, gauge systems, bribing markets, and the Curve Wars.*
