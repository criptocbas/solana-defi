# Token Distribution and Launch Strategies

> Written for experienced Solana developers entering token design.
> Last updated: February 2026

---

## Table of Contents

1. [Why Distribution Matters More Than Supply](#1-why-distribution-matters-more-than-supply)
2. [Token Allocation Models](#2-token-allocation-models)
3. [Launch Mechanisms](#3-launch-mechanisms)
   - [3.1 ICOs (Initial Coin Offerings)](#31-icos-initial-coin-offerings)
   - [3.2 IEOs and IDOs](#32-ieos-and-idos)
   - [3.3 Liquidity Bootstrapping Pools (LBPs)](#33-liquidity-bootstrapping-pools-lbps)
   - [3.4 Fair Launches](#34-fair-launches)
   - [3.5 Airdrops](#35-airdrops)
   - [3.6 Points Systems and Progressive Distribution](#36-points-systems-and-progressive-distribution)
4. [Vesting and Lockup Design](#4-vesting-and-lockup-design)
5. [Treasury Management](#5-treasury-management)
6. [The Insider Problem](#6-the-insider-problem)
7. [Distribution Case Studies](#7-distribution-case-studies)
8. [Anti-Sybil and Fairness](#8-anti-sybil-and-fairness)
9. [Program-Level Implementation](#9-program-level-implementation)
10. [References](#10-references)

---

## 1. Why Distribution Matters More Than Supply

A token can have perfect supply mechanics — fixed cap, elegant burns, sustainable emission — but if the distribution is wrong, it will fail. Distribution determines:

| What Distribution Determines | Why It Matters |
|---|---|
| **Who controls governance** | Concentrated distribution = plutocracy |
| **Who benefits from appreciation** | Unfair distribution = community resentment |
| **Initial sell pressure** | Too much to insiders = heavy selling at TGE |
| **Regulatory classification** | Distribution method affects security vs. utility classification |
| **Community alignment** | Fair distribution = loyal, invested community |
| **Decentralization** | Wide distribution = credible decentralization |

### The Gini Coefficient of Tokens

The Gini coefficient measures inequality in a distribution (0 = perfect equality, 1 = one entity holds everything). Most tokens are extremely concentrated:

| Token | Top 10 Holders % | Top 100 Holders % | Gini |
|---|---|---|---|
| BTC | ~5% | ~15% | ~0.88 |
| ETH | ~25% | ~40% | ~0.91 |
| UNI | ~50% | ~75% | ~0.97 |
| BONK | ~15% | ~35% | ~0.85 |

Even "decentralized" tokens have extreme concentration. The key metric is not perfect equality but whether the concentration is productive (staking, governance participation) vs. extractive (team dumping on retail).

---

## 2. Token Allocation Models

### The Standard Allocation Template

Most VC-backed protocols follow a similar allocation pattern:

```
Typical 2024-2026 Token Allocation:

  Community & Ecosystem:  35-50%
    ├── Airdrops:          10-20%
    ├── Liquidity mining:   5-15%
    ├── Grants/Ecosystem:   5-10%
    └── Future community:  10-20%

  Team & Advisors:         15-25%
    ├── Core team:          12-18%
    └── Advisors:            3-7%

  Investors:               15-25%
    ├── Seed round:          5-10%
    ├── Series A/B:          8-15%
    └── Strategic:           2-5%

  Treasury/Foundation:     10-20%
    ├── Development:         5-10%
    └── Strategic reserve:   5-10%
```

### Evolution of Allocations Over Time

| Era | Team/Insider % | Community % | Key Change |
|---|---|---|---|
| ICO era (2017) | 30-60% | 40-70% (sale) | Tokens sold, not earned |
| DeFi Summer (2020) | 20-40% | 60-80% | Liquidity mining explosion |
| VC era (2021-2022) | 30-50% | 50-70% | Large VC rounds at low FDV |
| Current (2024-2026) | 20-35% | 50-65% | Community-first, points/airdrops |

### The Community-First Movement

Jupiter's JUP launch in January 2024 set a new standard:

```
JUP Allocation:
  Team:       50%  (with meaningful vesting)
  Community:  50%  (airdrops + ecosystem)

First airdrop: ~10% of supply to 955,000 wallets
Subsequent: Active Supply Management by community vote
```

The 50/50 split was controversial (team gets half!) but transparent. Jupiter argued that a well-funded team builds better products, which benefits token holders long-term. The community accepted this because:
1. Team tokens have real vesting
2. The product (Jupiter DEX aggregator) was already the #1 aggregator on Solana
3. Community governs supply management (voted to burn 3B tokens)

---

## 3. Launch Mechanisms

### 3.1 ICOs (Initial Coin Offerings)

**How it works**: Protocol sells tokens directly to investors before or at launch.

```
Typical ICO Structure (2017):
1. Announce project + whitepaper
2. Private sale to VCs/angels at $0.01 per token
3. Public sale at $0.05 per token
4. List on exchange at market price
5. Team/investors already 5-50x in profit at listing
```

**Problems**:
- No product required (just a whitepaper)
- Insiders buy at deep discounts and dump on retail
- No vesting (immediate liquidity)
- 90%+ of ICOs went to zero
- Regulatory crackdown (SEC, MiCA)

**Legacy**: ICOs are effectively dead for serious projects. The model was exploitative but pioneered the concept of community fundraising for protocols.

### 3.2 IEOs and IDOs

**IEO (Initial Exchange Offering)**: Exchange vets the project and conducts the sale on their platform. The exchange acts as a gatekeeper. Examples: Binance Launchpad, FTX (defunct).

**IDO (Initial DEX Offering)**: Token launches directly on a decentralized exchange.

```
IDO Structure:
1. Create token
2. Provide initial liquidity on a DEX (e.g., Raydium)
3. Token becomes immediately tradeable
4. Price discovery happens in real-time on the DEX

Advantages:
  - Permissionless (no exchange gatekeeper)
  - Immediate liquidity
  - Transparent price discovery

Disadvantages:
  - Sniper bots front-run the launch
  - No price control (can pump 100x then crash 99%)
  - Easy to create fake liquidity then rug-pull
```

### 3.3 Liquidity Bootstrapping Pools (LBPs)

**Innovation**: Balancer's LBP mechanism creates a fairer price discovery process.

```
LBP Mechanics:
1. Create a weighted pool: 90% TOKEN / 10% USDC
2. Over 24-72 hours, weights shift: 90/10 → 10/90
3. The shifting weight creates natural downward price pressure
4. Buyers can enter at any point during the shift
5. Early buyers pay more, late buyers pay less (opposite of normal launches)
```

**Price behavior**:
```
Time 0:   Token weight 90%, price starts HIGH
Time 12h: Token weight 60%, price declining
Time 24h: Token weight 30%, price lower
Time 36h: Token weight 10%, price at minimum

The optimal strategy is to WAIT, which prevents FOMO-driven buying
```

**Why this is better**:
- Discourages sniping (snipers buy at the highest price)
- Rewards patience
- Creates a smooth price discovery curve
- No pre-sale discount for insiders

**Solana implementation**: Meteora's Alpha Vault and dynamic AMM pools provide similar functionality on Solana.

### 3.4 Fair Launches

**Definition**: No pre-mine, no VC allocation, no team allocation. All tokens are earned through participation.

**The YFI Standard** (July 2020):
```
Total supply: 30,000 YFI (later increased to 36,666)
Pre-mine: 0
VC allocation: 0
Team allocation: 0
Distribution: 100% to liquidity providers over 1 week
Andre Cronje (founder) owned 0 YFI at launch
```

YFI's fair launch created an intensely loyal community and launched at ~$30, eventually reaching $90,000 — a ~3000x appreciation. The fair launch narrative was so powerful that it became a template.

**The fair launch paradox**: Fair launches are great for community alignment but terrible for sustainable development. If the team gets nothing, how do they pay for development? YFI eventually approved additional minting for a treasury. Most "fair launch" tokens either:
1. Add team allocation later (governance vote)
2. Rely on the team buying tokens on the open market
3. Fail because the team has no funding

### 3.5 Airdrops

Airdrops distribute tokens for free to wallets meeting certain criteria. This has become the dominant distribution mechanism in 2024-2026.

#### Types of Airdrops

**Retroactive Airdrop**: Reward past users for organic behavior before any token was announced.

```
Uniswap (September 2020):
  Criteria: Any address that used Uniswap before September 1, 2020
  Amount: 400 UNI per address (~$1,200 at launch)
  Total: ~150,000 addresses

  The brilliance: Users were rewarded for behavior they would have done anyway.
  No farming, no gaming, just genuine usage.
```

**Tiered Airdrop**: Different amounts based on activity level.

```
Jupiter (January 2024):
  Tier 1 (heaviest users): 10,000+ JUP
  Tier 2 (regular users):  5,000-10,000 JUP
  Tier 3 (occasional):     1,000-5,000 JUP
  Tier 4 (minimal):        200-1,000 JUP

  Factors: Volume traded, number of swaps, time as user, products used
```

**Conditional Airdrop**: Must perform an action to claim.

```
Optimism (OP):
  Requirement: Delegate OP to a governance delegate before claiming
  Purpose: Forces engagement with governance, not just farming
```

**Ongoing/Multi-Round Airdrop**: Multiple distributions over time.

```
Jupiter:
  Round 1: January 2024 (retroactive)
  Round 2: Announced for active users
  Round 3+: Planned (community decides allocation)
```

#### The Airdrop Seller Problem

Most airdrop recipients sell immediately. Data from multiple airdrops shows:

```
Typical Airdrop Selling Pressure:
  Day 1:    30-50% of airdropped tokens sold
  Week 1:   50-70% sold
  Month 1:  60-80% sold
  Month 6:  75-90% sold
```

**Mitigation strategies**:
- Vesting/lockup on airdropped tokens (reduces immediate selling)
- Staking bonuses (higher APY for airdrop tokens that are staked)
- Governance requirements (must delegate before claiming)
- Multiple rounds (later rounds reward holders of round 1)

### 3.6 Points Systems and Progressive Distribution

The dominant 2024-2026 launch pattern:

```
1. Launch protocol without token
2. Award "points" for usage (deposits, trades, referrals)
3. Points are off-chain, non-transferable, no market
4. After months of usage data, announce token
5. Convert points to tokens at TGE
6. Users who accumulated points receive proportional allocation
```

**Examples**:
- **EigenLayer**: Points for restaking, converted to EIGEN
- **Blast**: Points for bridging + using DeFi on Blast L2
- **Kamino (Solana)**: Points for lending/liquidity provision

**Advantages**:
- Long evaluation period (months of real usage data)
- Harder to sybil (must provide real value over time)
- Creates genuine user engagement before TGE
- Allows team to calibrate distribution based on real data

**Disadvantages**:
- Opaque (users don't know the conversion rate)
- Creates its own meta-game (point farming strategies)
- Community trust issues if conversion is perceived as unfair
- Not truly decentralized (team controls the point system)

---

## 4. Vesting and Lockup Design

### Why Vesting Exists

Without vesting, insiders sell at TGE and crash the price. Vesting aligns long-term incentives by forcing insiders to hold through the protocol's growth period.

### Standard Vesting Components

| Component | Definition | Typical Range |
|---|---|---|
| **Cliff** | Period before any tokens unlock | 6-12 months |
| **Linear vest** | Even distribution after cliff | 12-36 months |
| **Total duration** | Cliff + linear vest | 18-48 months |
| **TGE unlock** | Percentage available immediately at launch | 0-25% |

### Common Vesting Schedules

**Team/Investors (Conservative)**:
```
Month 0-12:  CLIFF (0% unlocked)
Month 12:    25% unlocked at cliff
Month 13-48: Linear unlock (~2.08%/month)
Month 48:    100% unlocked

Total duration: 4 years
```

**Community/Airdrop (Moderate)**:
```
TGE:         50% immediately available
Month 1-6:   Remaining 50% linear unlock (~8.3%/month)
Month 6:     100% unlocked

Total duration: 6 months
```

**Aggressive (VC-friendly, community-hostile)**:
```
TGE:         25% immediately
Month 1-12:  75% linear unlock
Month 12:    100% unlocked

Problem: VCs can start selling almost immediately
```

### The Vesting Unlock Calendar

Token prices often drop near major vesting unlocks. The market anticipates sell pressure:

```
Price behavior around large unlocks:

  2 weeks before:  Price starts declining (anticipation)
  Unlock day:      Mixed (sometimes sell pressure already priced in)
  1 week after:    Continued selling as insiders execute
  1 month after:   Stabilization
```

**Design implication**: Spread unlocks evenly rather than creating large cliff events. A smooth daily linear vest creates less market impact than a monthly or quarterly cliff.

### Vesting Design Recommendations

| Stakeholder | Recommended Cliff | Recommended Vest | TGE Unlock |
|---|---|---|---|
| Core team | 12 months | 36 months linear | 0% |
| Investors (seed) | 12 months | 24-36 months linear | 0% |
| Investors (later rounds) | 6 months | 18-24 months linear | 0-10% |
| Advisors | 6 months | 12-24 months linear | 0% |
| Community airdrop | 0 months | 0-6 months | 50-100% |
| Ecosystem grants | 3 months | 12 months | 0-25% |

---

## 5. Treasury Management

### What a Protocol Treasury Does

The treasury is the protocol's war chest — funds for development, grants, liquidity incentives, partnerships, and emergencies.

### Treasury Composition

**Healthy treasury**:
```
  40% stablecoins (USDC, USDT) — operations, payroll, grants
  30% native token — governance power, ecosystem incentives
  20% blue-chip crypto (ETH, SOL, BTC) — diversified reserves
  10% strategic investments — protocol-owned liquidity, LP positions
```

**Risky treasury**:
```
  90%+ native token — if token price drops 80%, treasury loses 80% of value
  This is the "circular treasury" problem
```

### Treasury Governance

| Model | Speed | Trust | Example |
|---|---|---|---|
| **Team multisig** (3/5 or 4/7) | Fast (minutes) | Requires trust in team | Early-stage protocols |
| **Governance vote** (token-weighted) | Slow (days-weeks) | Trustless but attackable | Uniswap, Compound |
| **Hybrid** (multisig for small, governance for large) | Medium | Balanced | MakerDAO |
| **Streaming** (Sablier, Superfluid) | Continuous | Automated, predictable | Grant distributions |

### Protocol-Owned Liquidity (POL)

Instead of paying LPs with emissions (mercenary capital), some protocols use treasury funds to provide their own liquidity:

```
Traditional:
  Protocol emits 10,000 TOKENS/day to LPs → LPs sell tokens → price drops

POL:
  Protocol uses treasury to seed TOKEN/USDC pool
  No emissions needed → no sell pressure
  Protocol earns trading fees → sustainable
```

**Olympus (OHM)** pioneered protocol-owned liquidity via bonding:
```
User sells LP tokens to protocol at a discount
Protocol receives LP ownership permanently
Result: Protocol owns its own liquidity, not renting it
```

The concept was sound but OHM's implementation had problems (unsustainable APY, death spiral). The POL concept survived and is widely adopted.

---

## 6. The Insider Problem

### Token Distribution Conflicts of Interest

| Stakeholder | Wants | Conflicts With |
|---|---|---|
| **Team** | Large allocation, short vesting | Community (dilution, dump risk) |
| **VCs** | Low FDV entry, fast vesting | Community (buy low, sell high on them) |
| **Community** | Large community %, fair distribution | Team (need funding), VCs (need investment) |
| **Market makers** | Token loan + options | Everyone (can manipulate price) |

### The VC Discount Problem

```
Seed round:     $10M valuation → VCs buy at $0.01/token
Series A:       $100M valuation → VCs buy at $0.10/token
Public TGE:     $500M valuation → Retail buys at $0.50/token

VC is 50x in profit at TGE. Even with 1-year cliff + 2-year vest:
  - Month 12: VCs unlock 33% → sell at $0.50 → 50x return
  - Month 24: More unlocks → continued selling
  - Month 36: Fully unlocked → VCs have exited

Retail bought at $0.50 and price may never return there
```

### Mitigation Strategies

1. **Longer vesting**: 4-year vest with 1-year cliff minimum for investors
2. **Token-weighted cost basis**: Later investors get shorter vesting (they paid more)
3. **Performance-based unlock**: Team tokens unlock based on TVL/revenue milestones, not just time
4. **Community-majority allocation**: Ensure >50% goes to community
5. **Transparent unlock schedules**: Published, on-chain vesting contracts
6. **Market maker transparency**: Disclose market-making agreements

---

## 7. Distribution Case Studies

### Case Study 1: Uniswap (UNI) — The Retroactive Airdrop

```
Total supply: 1,000,000,000 UNI
Distribution:
  Community (governance treasury): 43.0%
  Team & future employees:         21.27%
  Investors:                        18.04%
  Advisors:                          0.69%
  Initial liquidity mining:         17.0% (over 4 years, but only ran 2 months)

Retroactive airdrop:
  - 400 UNI to every address that interacted with Uniswap
  - ~150,000 eligible addresses
  - ~60M UNI distributed (~6% of total supply)
  - Value at launch: ~$1,200 per address
```

**Why it worked**:
- Genuine surprise — no one was farming for it
- Rewarded real usage, not speculation
- Created immediate, passionate community
- Set the standard for all future airdrops

**What could be improved**:
- Binary distribution (everyone got 400 UNI regardless of volume)
- Sybil addresses received multiple drops
- LP mining lasted only 2 months before governance ended it

### Case Study 2: Jupiter (JUP) — Solana-Native Distribution

```
Total supply: 10,000,000,000 JUP
Distribution:
  Team (Jupuary):                50%
  Community:                     50%
    ├── Airdrop Round 1:         ~10% (955,000 wallets)
    ├── Active Staking Rewards:  ongoing
    ├── Ecosystem/Grants:        allocated by DAO
    └── Future rounds:           community-governed

Active Supply Management:
  - Community voted to burn 3B tokens (30% of supply)
  - Ongoing governance over remaining treasury
```

**Why it worked**:
- Product was already dominant (#1 aggregator on Solana)
- 50% community allocation is generous for a VC-backed project
- Tiered distribution rewarded heavy users proportionally
- ASM gave community ongoing control over supply
- Multiple rounds incentivize continued usage

### Case Study 3: YFI — The Pure Fair Launch

```
Total supply: 36,666 YFI
Pre-mine: 0
Team allocation: 0
Investor allocation: 0
Distribution: 100% to LPs over ~10 days across 3 pools

Andre Cronje (founder): 0 YFI at launch
```

**Why it worked**:
- Created the most passionate DeFi community
- Extreme scarcity (36,666 total) created high per-unit price
- Fair launch eliminated insider dump risk
- The community later voted to mint additional YFI for a treasury

**Why it's hard to replicate**:
- Andre was already a well-known, trusted developer
- Yearn had a working product before the token
- Team had no funding — relied on community treasury
- Very few teams can afford to launch with 0% allocation

### Case Study 4: Optimism (OP) — Governance-First

```
Total supply: 4,294,967,296 OP
Distribution:
  Ecosystem fund:     25%
  Retroactive public goods: 20%
  Core team:          19%
  Investors:          17%
  Airdrop #1:         5%
  Future airdrops:    14%

Airdrop requirements:
  - Must delegate to a governance delegate BEFORE claiming
  - Multiple criteria (Optimism users, DAO voters, multi-sig signers, etc.)
```

**Innovation**: Forcing delegation before claiming ensures that airdrop recipients participate in governance, not just sell. This created one of the most active governance communities in crypto.

### Case Study 5: Jito (JTO) — Staking-Aligned Distribution

```
Total supply: 1,000,000,000 JTO
Distribution:
  Community growth:    34.3%
  Core contributors:   24.5%
  Investors:           16.2%
  Airdrop:             10%
  Ecosystem:           15%

Airdrop criteria:
  - Based on jitoSOL holdings, duration, and staking activity
  - Rewarded genuine Solana stakers using Jito's MEV-enhanced staking
```

**Why it worked**:
- Aligned distribution with the protocol's purpose (MEV staking)
- JTO staking provides governance over Jito's MEV strategy
- Solana staking ecosystem is massive → large eligible user base

---

## 8. Anti-Sybil and Fairness

### The Sybil Problem

Sybil attacks exploit per-wallet distributions by creating many wallets:

```
Honest user: 1 wallet, gets 1,000 tokens
Sybil attacker: 1,000 wallets, gets 1,000,000 tokens

The attacker gets 1,000x more for the same economic contribution
```

### Anti-Sybil Techniques

| Technique | Effectiveness | Downsides |
|---|---|---|
| **Minimum transaction threshold** | Low | Easy to reach with small transactions |
| **Activity over time** | Medium | Bots can maintain persistent activity |
| **Transaction volume weighting** | Medium | Wash trading can inflate volume |
| **Cross-protocol analysis** | High | Complex, may exclude legitimate users |
| **Social graph analysis** | High | Cluster analysis can detect sybil networks |
| **Identity verification** | Very high | Kills permissionless ethos |
| **Gitcoin Passport / World ID** | High | Requires opt-in, privacy concerns |
| **Quadratic distribution** | Medium-High | Sybils still benefit, just less |

### Quadratic Distribution

Instead of linear distribution (more volume = more tokens), quadratic distribution rewards breadth over depth:

```
Linear:     tokens = f(total_volume)
Quadratic:  tokens = f(sqrt(total_volume))

User A: $1M volume → Linear: 1000 tokens, Quadratic: 31.6 tokens
User B: $1K volume → Linear: 1 token, Quadratic: 1 token

Ratio:
  Linear:     1000:1 (1000x more for 1000x volume)
  Quadratic:  31.6:1 (31.6x more for 1000x volume)
```

Quadratic distribution compresses the range, benefiting smaller users relative to whales.

### Cluster Analysis

Modern anti-sybil tools analyze on-chain transaction graphs to identify wallet clusters:

```
Indicators of sybil clusters:
  - Many wallets funded from the same source
  - Near-simultaneous transactions
  - Sequential transaction patterns
  - Identical transaction amounts
  - All wallets interact with the same contracts in the same order
  - Funds consolidate back to the same destination
```

Tools like Chainalysis, Nansen, and custom Dune queries can identify these patterns. Jupiter's airdrop team, for example, manually reviewed flagged clusters.

---

## 9. Program-Level Implementation

### Vesting Contract on Solana (Anchor)

```rust
use anchor_lang::prelude::*;
use anchor_spl::token::{self, Token, TokenAccount, Transfer};

#[account]
pub struct VestingSchedule {
    pub beneficiary: Pubkey,
    pub token_mint: Pubkey,
    pub total_amount: u64,
    pub released_amount: u64,
    pub start_timestamp: i64,
    pub cliff_duration: i64,    // seconds
    pub vesting_duration: i64,  // seconds (total, including cliff)
    pub revocable: bool,
    pub admin: Pubkey,
    pub bump: u8,
}

impl VestingSchedule {
    pub const SPACE: usize = 8 + 32 + 32 + 8 + 8 + 8 + 8 + 8 + 1 + 32 + 1;

    /// Calculate vested amount at a given timestamp
    pub fn vested_amount(&self, current_time: i64) -> u64 {
        let elapsed = current_time - self.start_timestamp;

        // Before cliff: nothing vested
        if elapsed < self.cliff_duration {
            return 0;
        }

        // After full vesting: everything vested
        if elapsed >= self.vesting_duration {
            return self.total_amount;
        }

        // During linear vest (post-cliff):
        // vested = total * elapsed / vesting_duration
        (self.total_amount as u128)
            .checked_mul(elapsed as u128)
            .unwrap()
            .checked_div(self.vesting_duration as u128)
            .unwrap() as u64
    }

    /// Calculate releasable (vested minus already released)
    pub fn releasable_amount(&self, current_time: i64) -> u64 {
        self.vested_amount(current_time)
            .checked_sub(self.released_amount)
            .unwrap_or(0)
    }
}
```

### Claiming Vested Tokens

```rust
pub fn claim_vested(ctx: Context<ClaimVested>) -> Result<()> {
    let schedule = &mut ctx.accounts.vesting_schedule;
    let clock = Clock::get()?;

    let releasable = schedule.releasable_amount(clock.unix_timestamp);
    require!(releasable > 0, VestingError::NothingToRelease);

    // Update released amount
    schedule.released_amount = schedule
        .released_amount
        .checked_add(releasable)
        .ok_or(VestingError::MathOverflow)?;

    // Transfer tokens from vesting vault to beneficiary
    let seeds = &[
        b"vesting_vault",
        schedule.beneficiary.as_ref(),
        &[schedule.bump],
    ];
    let signer = &[&seeds[..]];

    token::transfer(
        CpiContext::new_with_signer(
            ctx.accounts.token_program.to_account_info(),
            Transfer {
                from: ctx.accounts.vesting_vault.to_account_info(),
                to: ctx.accounts.beneficiary_token_account.to_account_info(),
                authority: ctx.accounts.vesting_authority.to_account_info(),
            },
            signer,
        ),
        releasable,
    )?;

    Ok(())
}
```

### Merkle Airdrop on Solana

For large airdrops (100K+ addresses), using a Merkle tree is gas-efficient:

```rust
use anchor_lang::prelude::*;
use anchor_lang::solana_program::keccak;

#[account]
pub struct MerkleDistributor {
    pub merkle_root: [u8; 32],
    pub token_mint: Pubkey,
    pub max_total_claim: u64,
    pub total_claimed: u64,
    pub admin: Pubkey,
    pub bump: u8,
}

/// Each user has a claim receipt PDA to prevent double-claiming
#[account]
pub struct ClaimReceipt {
    pub claimed: bool,
    pub amount: u64,
    pub claimant: Pubkey,
}

pub fn claim_airdrop(
    ctx: Context<ClaimAirdrop>,
    amount: u64,
    proof: Vec<[u8; 32]>,
) -> Result<()> {
    let distributor = &mut ctx.accounts.distributor;
    let receipt = &mut ctx.accounts.claim_receipt;

    // Prevent double claim
    require!(!receipt.claimed, AirdropError::AlreadyClaimed);

    // Verify Merkle proof
    let leaf = keccak::hashv(&[
        &ctx.accounts.claimant.key().to_bytes(),
        &amount.to_le_bytes(),
    ]);

    let mut computed_hash = leaf.0;
    for proof_element in proof.iter() {
        if computed_hash <= *proof_element {
            computed_hash = keccak::hashv(&[&computed_hash, proof_element]).0;
        } else {
            computed_hash = keccak::hashv(&[proof_element, &computed_hash]).0;
        }
    }

    require!(
        computed_hash == distributor.merkle_root,
        AirdropError::InvalidProof
    );

    // Mark as claimed
    receipt.claimed = true;
    receipt.amount = amount;
    receipt.claimant = ctx.accounts.claimant.key();

    // Transfer tokens
    distributor.total_claimed = distributor
        .total_claimed
        .checked_add(amount)
        .ok_or(AirdropError::MathOverflow)?;

    // ... token transfer CPI ...

    Ok(())
}
```

### Emission Controller

```rust
#[account]
pub struct EmissionConfig {
    pub token_mint: Pubkey,
    pub initial_rate: u64,       // tokens per second
    pub decay_rate_bps: u64,     // decay per epoch (basis points)
    pub epoch_duration: i64,     // seconds per epoch
    pub start_timestamp: i64,
    pub last_emission_time: i64,
    pub total_emitted: u64,
    pub max_supply: u64,         // cap (0 = no cap)
    pub authority_bump: u8,
}

impl EmissionConfig {
    /// Calculate current emission rate
    pub fn current_rate(&self, current_time: i64) -> u64 {
        let elapsed = current_time - self.start_timestamp;
        let epochs = elapsed / self.epoch_duration;

        let mut rate = self.initial_rate as u128;
        for _ in 0..epochs {
            rate = rate * (10000 - self.decay_rate_bps as u128) / 10000;
        }

        rate as u64
    }

    /// Calculate tokens to emit since last emission
    pub fn pending_emission(&self, current_time: i64) -> u64 {
        let elapsed = (current_time - self.last_emission_time) as u128;
        let rate = self.current_rate(current_time) as u128;
        let pending = rate * elapsed;

        // Cap at max supply
        if self.max_supply > 0 {
            let remaining = (self.max_supply - self.total_emitted) as u128;
            pending.min(remaining) as u64
        } else {
            pending as u64
        }
    }
}
```

---

## 10. References

1. **Uniswap UNI announcement (2020)**: The retroactive airdrop post
2. **Jupiter JUP launch documentation (2024)**: Community-first distribution model
3. **"Fair Launch Capital" — various (2020)**: The fair launch movement
4. **Token Unlocks (tokenunlocks.app)**: Vesting schedule data for major tokens
5. **"Sybil Resistance in DeFi Airdrops" — Flashbots research**: Anti-sybil techniques
6. **SPL Token Vesting by Bonfida**: Open-source vesting contract on Solana
7. **Merkle Distributor by Jito**: Efficient airdrop distribution on Solana
8. **"The State of Airdrops" — Dune Analytics dashboards**: Empirical selling data

---

*Next: [04 - Token Utility and Value Accrual](./04-token-utility-and-value-accrual.md) — How tokens capture value, fee switches, buyback-and-burn, staking rewards, and the critical question of why anyone should hold your token.*
