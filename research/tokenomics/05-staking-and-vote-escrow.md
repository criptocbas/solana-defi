# Staking, Vote-Escrow, and Incentive Design

> Written for experienced Solana developers entering token design.
> Last updated: February 2026

---

## Table of Contents

1. [Staking Economics Fundamentals](#1-staking-economics-fundamentals)
2. [Proof-of-Stake Network Staking](#2-proof-of-stake-network-staking)
3. [Vote-Escrow (ve) Tokenomics](#3-vote-escrow-ve-tokenomics)
   - [3.1 The veCRV Model](#31-the-vecrv-model)
   - [3.2 Gauge Systems and Emission Voting](#32-gauge-systems-and-emission-voting)
   - [3.3 Boost Mechanics](#33-boost-mechanics)
4. [The Curve Wars](#4-the-curve-wars)
5. [Bribing Markets](#5-bribing-markets)
6. [Liquidity Mining and Mercenary Capital](#6-liquidity-mining-and-mercenary-capital)
7. [Incentive Design Principles](#7-incentive-design-principles)
8. [Emission Schedule Design](#8-emission-schedule-design)
9. [Slashing and Penalties](#9-slashing-and-penalties)
10. [Program-Level Implementation](#10-program-level-implementation)
11. [References](#11-references)

---

## 1. Staking Economics Fundamentals

### What Staking Actually Does

Staking is the act of locking tokens in exchange for rewards and/or governance power. At its core, staking solves a simple problem: **how do you incentivize people to not sell?**

```
Without staking:
  Token holder choice: hold (opportunity cost) vs. sell (realize value)
  Default behavior: sell, because holding generates no return

With staking:
  Token holder choice: stake (earn yield, lose liquidity) vs. sell
  Changed behavior: stake, because the yield compensates for illiquidity
```

### The Staking Trilemma

Every staking system must balance three competing goals:

```
                    Security
                   /        \
                  /          \
                 /            \
           Liquidity ──── Decentralization
```

| Goal | How Staking Helps | Trade-off |
|---|---|---|
| **Security** | More staked = more expensive to attack | Requires high rewards (inflation) |
| **Liquidity** | Less staked = more liquid markets | Less security |
| **Decentralization** | Many small stakers = decentralized | Economies of scale favor large validators |

### Optimal Staking Ratio

For PoS networks, there is an optimal staking ratio that balances security and liquidity:

```
If staking ratio is too LOW (<30%):
  - Network security is weak (cheap to acquire 33% stake)
  - But plenty of liquidity for trading

If staking ratio is too HIGH (>80%):
  - Network is extremely secure
  - But most tokens are locked → thin liquidity → high volatility
  - DeFi ecosystem suffers (no tokens available for lending, LPs)

Sweet spot: ~40-65% staked
  SOL: ~65% staked (slightly high but liquid staking helps)
  ETH: ~27% staked (slightly low, increasing)
  ATOM: ~62% staked
```

---

## 2. Proof-of-Stake Network Staking

### Solana Staking Economics

```
Solana staking parameters (Feb 2026):
  Current inflation: ~5.2%
  Staking ratio: ~65%
  Validator count: ~1,800 active
  Minimum stake: None (delegated staking)
  Epoch length: ~2 days
  Unstaking period: 1 epoch (~2 days)

Staker yield calculation:
  Total inflation: 5.2%
  Staking ratio: 65%
  Gross staker yield: 5.2% / 0.65 = 8.0%
  Less validator commission: ~7% average → 8.0% × 0.93 = 7.44%
  Less inflation dilution: 5.2%
  Net real yield: 7.44% - 5.2% = 2.24%
```

**Key insight**: Solana staking rewards are primarily inflationary. The real yield (after accounting for dilution) is only ~2%. Non-stakers lose ~5.2% annually to dilution. This creates a strong incentive to stake, which locks supply.

### Ethereum Staking Economics

```
Ethereum staking parameters (Feb 2026):
  Issuance: ~1,700 ETH/day
  Fee burns: variable (often exceeds issuance)
  Staking ratio: ~27%
  Validator count: ~900,000
  Minimum stake: 32 ETH
  Unstaking period: exit queue (variable, days to weeks)

Staker yield:
  Base APR (issuance): ~3.3%
  Priority fees/MEV: ~0.5-1.5%
  Total gross: ~4-5%
  Net inflation: ~-0.2% to +0.5% (often deflationary)
  Real yield: 3.5-5.0% (most of the yield is real because inflation is near-zero)
```

**ETH vs SOL staking comparison**:
```
                    ETH Staking       SOL Staking
Gross APY:          ~4-5%             ~7.5%
Inflation:          ~0%               ~5.2%
Real yield:         ~4-5%             ~2.2%
Lock period:        Variable queue     ~2 days
Minimum:            32 ETH            None
Liquid staking:     stETH, rETH       mSOL, jitoSOL
```

ETH staking has lower nominal APY but higher real yield because inflation is near-zero.

### Liquid Staking: Solving the Liquidity Problem

The fundamental problem with staking: locked tokens can't participate in DeFi. Liquid staking solves this by issuing a receipt token:

```
Traditional staking:
  User stakes SOL → locked for epoch → earns ~7.5% APY → can't use SOL in DeFi

Liquid staking:
  User stakes SOL → receives mSOL → earns ~7.5% APY → uses mSOL in DeFi
  mSOL/SOL exchange rate grows over time as staking rewards accrue

mSOL value:
  Day 0:   1 mSOL = 1.00 SOL
  Year 1:  1 mSOL ≈ 1.075 SOL (7.5% yield accrued)
  Year 2:  1 mSOL ≈ 1.156 SOL (compounded)
```

**Liquid staking protocols on Solana**:
| Protocol | Token | TVL | Yield Enhancement |
|---|---|---|---|
| Marinade | mSOL | Large | Stake across many validators (delegation strategy) |
| Jito | jitoSOL | Large | MEV rewards from Jito validator client |
| Blaze | bSOL | Medium | Stake pool with delegation strategy |
| Sanctum | Various LSTs | Growing | Unified LST liquidity layer |

---

## 3. Vote-Escrow (ve) Tokenomics

### 3.1 The veCRV Model

Curve Finance invented the vote-escrow model in 2020, and it has become the most influential tokenomics innovation since Bitcoin's halving schedule.

**Core mechanics**:

```
Lock CRV tokens → Receive veCRV (vote-escrowed CRV)

Lock duration → veCRV amount:
  1 year lock:  1 CRV → 0.25 veCRV
  2 year lock:  1 CRV → 0.50 veCRV
  3 year lock:  1 CRV → 0.75 veCRV
  4 year lock:  1 CRV → 1.00 veCRV

veCRV decays linearly:
  At lock:        1.00 veCRV
  1 year later:   0.75 veCRV
  2 years later:  0.50 veCRV
  3 years later:  0.25 veCRV
  4 years later:  0.00 veCRV (lock expires)
```

**veCRV provides three benefits**:

1. **Governance voting**: Vote on which pools receive CRV emissions
2. **Fee sharing**: Earn 50% of Curve's trading fees (paid in 3CRV)
3. **Boost**: Up to 2.5x multiplier on CRV farming rewards

**Why it works**:
- Long locks align incentives (can't dump and run)
- Decaying weight requires re-locking to maintain power
- Three simultaneous benefits create strong demand for veCRV
- CRV emissions have a destination (gauge votes determine allocation)

### 3.2 Gauge Systems and Emission Voting

**What is a gauge?**

A gauge is a smart contract that distributes token emissions to a specific pool or activity. The emission rate for each gauge is determined by vote:

```
Curve Gauge System:

  Total CRV emission: 1,000,000 CRV/week

  veCRV holders vote on gauge weights:
    3pool (DAI/USDC/USDT):     30% → 300,000 CRV/week
    ETH/stETH:                  25% → 250,000 CRV/week
    FRAX/USDC:                  15% → 150,000 CRV/week
    Other pools:                30% → 300,000 CRV/week

  Votes are updated every epoch (weekly for Curve)
  Holders can change their votes each epoch
```

**The gauge creates a market for emissions**:
```
If your protocol has a Curve pool:
  More CRV emissions to your pool → Higher APY → More TVL → More trading volume

  To get more emissions, you need veCRV votes
  To get veCRV votes, you either:
    1. Buy CRV and lock it yourself
    2. Bribe veCRV holders to vote for your pool
```

This is the foundation of the "Curve Wars."

### 3.3 Boost Mechanics

veCRV holders who provide liquidity to Curve pools receive a boost on their CRV farming rewards:

```
Base CRV farming rate: 100 CRV/day (without boost)
Maximum boost: 2.5x
Boosted rate: up to 250 CRV/day

Boost depends on:
  1. Your veCRV balance (more veCRV = more boost)
  2. The pool's total liquidity
  3. Your share of the pool

Boost formula (simplified):
  boost_factor = min(2.5, your_veCRV / required_veCRV)
  required_veCRV = (pool_liquidity × your_LP_share) / total_veCRV × 0.4
```

**This creates three demand pressures for CRV**:
1. Lock for governance (vote on gauges)
2. Lock for fee sharing (earn 50% of trading fees)
3. Lock for boost (earn up to 2.5x more CRV)

---

## 4. The Curve Wars

### What Are the Curve Wars?

The "Curve Wars" is the competition between DeFi protocols to accumulate veCRV voting power in order to direct CRV emissions to their liquidity pools. It was the first major instance of **meta-governance** — protocols governing other protocols.

### The Key Players

**Convex Finance (CVX)**:
```
Convex's strategy:
  1. Accept CRV deposits from users
  2. Lock ALL deposited CRV as veCRV (permanent lock)
  3. Pool the veCRV voting power
  4. CVX holders control how Convex's veCRV votes

Result:
  - Convex controls ~50% of all veCRV
  - If you control CVX, you control Curve's emission votes
  - CVX becomes a "meta-governance" token
```

**Why protocols fight for Curve emissions**:
```
Protocol X has a stablecoin (xUSD) and wants it to be widely used.
Wide usage requires deep liquidity.
Deep liquidity requires attractive yields for LPs.
Attractive yields come from CRV emissions.
CRV emissions are directed by veCRV votes.
veCRV votes are controlled by CVX holders.

Therefore: Protocol X must either:
  a) Buy CRV and lock it (expensive, capital-intensive)
  b) Buy CVX and vote (more capital-efficient, since CVX controls veCRV)
  c) Bribe CVX/veCRV holders to vote for their pool (cheapest)
```

### Economic Analysis

```
Cost of liquidity via Curve Wars:

Option A: Traditional liquidity mining
  Emit 10,000 TOKENS/day to LPs
  At $10/TOKEN → $100,000/day → $36.5M/year
  LPs sell tokens → price drops → need more emissions

Option B: Curve Wars approach
  Buy $5M of CRV, lock as veCRV for 4 years
  Direct 500,000 CRV/week to your pool
  At $0.50/CRV → $250,000/week → $13M/year in incentives
  Cost to protocol: $5M one-time vs. $36.5M/year
  Plus: CRV emissions come from Curve, not your treasury

Option C: Bribe
  Pay $0.10 per veCRV vote
  Need 10M veCRV votes → $1M/week → $52M/year
  But the resulting emissions are worth $250K/week to LPs

ROI analysis: $1 in bribes generates $X in emissions
  If $1 bribe → $2 in CRV emissions → 2x return for bribe recipients
  veCRV holders earn MORE from bribes than from Curve trading fees
```

### Curve Wars on Solana

While the exact Curve Wars dynamic is Ethereum-specific, similar patterns exist on Solana:

```
Solana equivalents:
  - Raydium emission voting (RAY stakers influence LP rewards)
  - Marinade MNDE gauge voting (determines validator delegation)
  - Jupiter JUP governance (votes on ecosystem incentives)
  - Meteora DLMM incentive allocation
```

The ve-model has been adopted by several Solana protocols, though the ecosystem is less mature than Ethereum's Curve Wars.

---

## 5. Bribing Markets

### How Bribing Works

```
Protocol: "If you vote for my pool's gauge, I'll pay you $X"
veCRV holder: "Deal. Your pool gets my votes, I get your bribe."

Bribe platforms:
  - Votium (Ethereum): Bribes for Convex/Curve votes
  - Hidden Hand (Ethereum): Multi-protocol bribe marketplace
  - Tribeca (Solana): Gauge voting framework
```

### Bribe Economics

```
Efficient bribe market equilibrium:
  Bribe value ≈ Expected CRV emission value × discount

Example:
  Your pool receives 100,000 CRV/week from gauge votes
  CRV price: $0.50
  Emission value: $50,000/week

  Rational bribe: up to $50,000/week to secure those votes
  Actual bribe: ~$30,000/week (discount because bribers are price-takers)

  veCRV holder return:
    From bribes: $30,000/week
    From Curve fees: $10,000/week
    Total: $40,000/week

    If this exceeds what they'd earn from Curve fees alone → rational to accept bribes
```

### Why Bribing Is Efficient

```
Without bribing:
  Protocol pays LPs directly: $100K/week in token emissions
  LPs dump tokens → price drops → protocol loses value

With bribing:
  Protocol pays veCRV bribes: $30K/week
  Curve emits CRV to the pool: $50K/week in CRV rewards
  Net: $30K in cost generates $50K in LP incentives
  AND the protocol doesn't inflate its own token supply
```

Bribing is more capital-efficient than direct liquidity mining because it leverages Curve's existing emission infrastructure.

---

## 6. Liquidity Mining and Mercenary Capital

### The Liquidity Mining Problem

```
DeFi Summer pattern:
  1. Launch token with high emissions → 1000% APY
  2. TVL floods in (mercenary capital chasing yield)
  3. Farmers dump emitted tokens → price crashes
  4. APY drops (same emissions, more TVL, lower token price)
  5. Mercenary capital leaves for next high-APY farm
  6. TVL drops, token price drops further → death spiral
```

### Quantifying Mercenary Capital

```
Loyalty metric = TVL retained after emission reduction

Protocol A: Reduces emissions by 50%
  TVL before: $100M
  TVL after:  $80M
  Loyalty: 80% (good — 80% of capital is "sticky")

Protocol B: Reduces emissions by 50%
  TVL before: $100M
  TVL after:  $20M
  Loyalty: 20% (bad — 80% of capital was mercenary)
```

### Solving Mercenary Capital

| Solution | Mechanism | Example |
|---|---|---|
| **Time-locked rewards** | Emissions unlock over weeks/months | Pendle vePENDLE |
| **Boosted long-term staking** | More rewards for longer lock | Curve veCRV boost |
| **Protocol-owned liquidity** | Protocol owns its own LP positions | Olympus POL |
| **Real yield** | Revenue-based rewards, not emissions | GMX fee sharing |
| **Loyalty multipliers** | Longer staking → higher multiplier | Marinade mSOL |
| **Exit penalties** | Fee for early unstaking | Some yield farms charge 0.5-2% early withdrawal |

### Best Practices for Emission Programs

```
DO:
  ✓ Set emission schedules in advance (predictable)
  ✓ Decrease emissions over time (disinflationary)
  ✓ Combine emissions with real yield (reduce dependence)
  ✓ Lock emitted tokens (reduce immediate sell pressure)
  ✓ Target emissions to strategic pools (not spray everywhere)

DON'T:
  ✗ Promise APYs you can't sustain
  ✗ Emit without a lock or vesting schedule
  ✗ Compete solely on emission APY (race to the bottom)
  ✗ Ignore the sell pressure that emissions create
  ✗ Treat TVL from mercenary capital as real growth
```

---

## 7. Incentive Design Principles

### Mechanism Design Basics

Tokenomics is applied **mechanism design** — the branch of economics/game theory that designs rules so self-interested agents produce desired outcomes.

**Key principles**:

**1. Incentive Compatibility**: The optimal strategy for each participant should align with the protocol's goals.

```
Good: Stakers earn more by locking longer → protocol gets stable capital
Bad:  Highest rewards go to short-term farmers → protocol gets mercenary capital
```

**2. Individual Rationality**: Each participant must be better off participating than not.

```
Good: Staking yield > opportunity cost → rational to stake
Bad:  Staking yield < opportunity cost → rational to sell and deploy capital elsewhere
```

**3. Budget Balance**: The protocol's incentive spending should be sustainable.

```
Good: Revenue-funded rewards (spending < income)
Bad:  Emission-funded rewards with no revenue (spending from treasury, eventually exhausts)
```

### The Incentive Stack

Design incentives in layers, from most to least sustainable:

```
Layer 1: Revenue sharing (sustainable indefinitely)
  → "Stake TOKEN, earn 5% APR from protocol fees"

Layer 2: Governance + boost (moderate sustainability)
  → "Lock TOKEN for governance power and boosted yields"

Layer 3: Targeted emissions (time-limited)
  → "Earn TOKEN rewards for providing liquidity to specific pools"

Layer 4: One-time distribution (bootstrap only)
  → "Airdrop tokens to early users"
```

### Game Theory in Token Design

**Prisoner's Dilemma of Staking**:
```
                    Other holders stake    Other holders sell
You stake:          Both earn yield,       You earn yield but
                    price stable           price drops
You sell:           You realize value,     Both lose (crash)
                    others diluted
```

Good tokenomics makes "stake" the dominant strategy by ensuring staking yield exceeds the opportunity cost of selling.

**Coordination Games**:
```
veCRV lock creates a coordination game:
  - If everyone locks for 4 years → maximum governance power for all
  - If no one locks → no one has governance power
  - If some lock and some don't → lockers get outsized rewards

The boost mechanic rewards early/long lockers → creates incentive to coordinate on "everyone locks long"
```

---

## 8. Emission Schedule Design

### Emission Budget

```
Total emission budget = Max supply - Initial circulating supply

Allocation:
  Liquidity mining:      40% of budget (4 years)
  Staking rewards:       30% of budget (ongoing, decreasing)
  Community grants:      20% of budget (DAO-managed)
  Strategic reserves:    10% of budget (emergency/opportunity)
```

### Optimal Emission Curve

```
Year 1: High emissions (bootstrapping)
  Purpose: Attract initial liquidity and users
  Rate: 40% of liquidity mining budget

Year 2: Moderate emissions (growth)
  Purpose: Sustain growth while building revenue
  Rate: 30% of liquidity mining budget

Year 3: Low emissions (maturation)
  Purpose: Supplement revenue-based rewards
  Rate: 20% of liquidity mining budget

Year 4: Minimal emissions (self-sustaining)
  Purpose: Protocol should be revenue-sufficient
  Rate: 10% of liquidity mining budget

Year 5+: Revenue-only rewards
  Purpose: Fully sustainable
  Rate: 0% emissions, 100% revenue-based
```

### Emission vs. Revenue Transition

```
Revenue share of total rewards:

Year 1: |████░░░░░░░░░░░░░░░░| 20% revenue / 80% emissions
Year 2: |████████░░░░░░░░░░░░| 40% / 60%
Year 3: |████████████░░░░░░░░| 60% / 40%
Year 4: |████████████████░░░░| 80% / 20%
Year 5: |████████████████████| 100% / 0%

Goal: By year 5, protocol revenue fully replaces emission rewards
```

---

## 9. Slashing and Penalties

### What Is Slashing?

Slashing is the penalty mechanism that enforces correct behavior in staking systems. A portion of a staker's tokens is destroyed (burned) if they misbehave.

### Types of Slashable Offenses

| Offense | Network | Penalty |
|---|---|---|
| **Double signing** | Ethereum, Solana | Severe (up to 100% of stake) |
| **Inactivity/downtime** | Ethereum | Mild (leak, not slash) |
| **Incorrect data** | Chainlink, Graph | Variable (depends on severity) |
| **Protocol insolvency** | Aave (Safety Module) | Up to 30% of staked AAVE |

### Slashing Design Considerations

```
Too harsh slashing:
  - Discourages staking (risk too high)
  - Only large operators can afford the risk
  - Reduces decentralization

Too mild slashing:
  - Doesn't deter misbehavior
  - Free-riding problem (stake, collect rewards, don't validate properly)
  - Security assumptions break down

Sweet spot:
  - Proportional to offense severity
  - Minimum threshold (don't slash for minor issues)
  - Grace period for recovery (e.g., inactivity window before slashing)
  - Insurance options (delegators can buy slash protection)
```

### Solana's Approach

Solana doesn't have protocol-level slashing for validators in the same way Ethereum does. Instead:

```
Solana validator incentives:
  - Good validators earn rewards (stake weight × commission)
  - Bad validators lose delegators (stake moves elsewhere)
  - The "slashing" is economic: lost delegation = lost revenue
  - This is softer than Ethereum's approach but still effective
```

---

## 10. Program-Level Implementation

### Vote-Escrow Lock on Solana

```rust
use anchor_lang::prelude::*;

#[account]
pub struct VeLock {
    pub owner: Pubkey,
    pub token_mint: Pubkey,
    pub amount: u64,            // Tokens locked
    pub lock_start: i64,        // Timestamp
    pub lock_end: i64,          // Timestamp
    pub initial_ve_power: u64,  // vePower at lock time
    pub bump: u8,
}

impl VeLock {
    pub const MAX_LOCK_DURATION: i64 = 4 * 365 * 24 * 3600; // 4 years

    /// Calculate current vote-escrow power (decays linearly)
    pub fn current_ve_power(&self, current_time: i64) -> u64 {
        if current_time >= self.lock_end {
            return 0; // Lock expired
        }

        let remaining = (self.lock_end - current_time) as u128;
        let total_duration = (self.lock_end - self.lock_start) as u128;

        // Linear decay: power × remaining / total
        ((self.initial_ve_power as u128) * remaining / total_duration) as u64
    }

    /// Calculate initial ve power from lock duration
    pub fn calculate_ve_power(amount: u64, lock_duration: i64) -> u64 {
        // 4-year lock → 1:1 ratio
        // 1-year lock → 1:0.25 ratio
        let max_duration = Self::MAX_LOCK_DURATION as u128;
        let duration = (lock_duration as u128).min(max_duration);

        ((amount as u128) * duration / max_duration) as u64
    }
}

/// Create a new ve-lock
pub fn create_lock(
    ctx: Context<CreateLock>,
    amount: u64,
    lock_duration: i64,
) -> Result<()> {
    require!(amount > 0, VeError::ZeroAmount);
    require!(
        lock_duration >= 7 * 24 * 3600, // Minimum 1 week
        VeError::LockTooShort
    );
    require!(
        lock_duration <= VeLock::MAX_LOCK_DURATION,
        VeError::LockTooLong
    );

    let clock = Clock::get()?;
    let ve_power = VeLock::calculate_ve_power(amount, lock_duration);

    let lock = &mut ctx.accounts.ve_lock;
    lock.owner = ctx.accounts.owner.key();
    lock.token_mint = ctx.accounts.token_mint.key();
    lock.amount = amount;
    lock.lock_start = clock.unix_timestamp;
    lock.lock_end = clock.unix_timestamp + lock_duration;
    lock.initial_ve_power = ve_power;

    // Transfer tokens to lock vault
    // ... token transfer CPI ...

    Ok(())
}
```

### Gauge Voting System

```rust
#[account]
pub struct Gauge {
    pub pool: Pubkey,        // The LP pool this gauge rewards
    pub total_votes: u64,    // Total vePower allocated to this gauge
    pub emission_share: u64, // Basis points of total emissions (calculated)
    pub epoch: u64,          // Current voting epoch
}

#[account]
pub struct GaugeVote {
    pub voter: Pubkey,       // The ve-lock owner
    pub gauge: Pubkey,       // Which gauge they voted for
    pub ve_power_used: u64,  // How much vePower allocated
    pub epoch: u64,          // Which epoch this vote is for
}

/// Vote for a gauge (allocate your vePower to a pool)
pub fn vote_for_gauge(
    ctx: Context<VoteForGauge>,
    ve_power_amount: u64,
) -> Result<()> {
    let ve_lock = &ctx.accounts.ve_lock;
    let clock = Clock::get()?;

    // Check voter has sufficient vePower
    let available_power = ve_lock.current_ve_power(clock.unix_timestamp);
    let already_allocated = get_total_allocated_power(&ctx.accounts.voter)?;
    let remaining_power = available_power
        .checked_sub(already_allocated)
        .ok_or(VeError::InsufficientVePower)?;

    require!(
        ve_power_amount <= remaining_power,
        VeError::InsufficientVePower
    );

    // Record vote
    let vote = &mut ctx.accounts.gauge_vote;
    vote.voter = ctx.accounts.owner.key();
    vote.gauge = ctx.accounts.gauge.key();
    vote.ve_power_used = ve_power_amount;
    vote.epoch = ctx.accounts.epoch_config.current_epoch;

    // Update gauge totals
    let gauge = &mut ctx.accounts.gauge;
    gauge.total_votes = gauge
        .total_votes
        .checked_add(ve_power_amount)
        .ok_or(VeError::MathOverflow)?;

    Ok(())
}

/// Calculate emission allocation per gauge (called at epoch end)
pub fn tally_votes(ctx: Context<TallyVotes>) -> Result<()> {
    let total_votes_all_gauges = get_total_votes_all_gauges()?;

    for gauge in ctx.remaining_accounts.iter() {
        let mut gauge_data: Account<Gauge> = Account::try_from(gauge)?;

        // Each gauge gets emissions proportional to its vote share
        gauge_data.emission_share = if total_votes_all_gauges > 0 {
            ((gauge_data.total_votes as u128) * 10000u128
                / total_votes_all_gauges as u128) as u64
        } else {
            0
        };

        gauge_data.epoch += 1;
        gauge_data.total_votes = 0; // Reset for next epoch
    }

    Ok(())
}
```

### Boost Calculator

```rust
/// Calculate boost multiplier for an LP position
/// Based on veCRV-style mechanics
pub fn calculate_boost(
    user_ve_power: u64,
    total_ve_power: u64,
    user_lp_amount: u64,
    total_lp_amount: u64,
) -> u64 {
    // Min boost: 1.0x (10000 bps)
    // Max boost: 2.5x (25000 bps)

    if total_ve_power == 0 || total_lp_amount == 0 {
        return 10000; // 1.0x
    }

    // Boost formula:
    // boosted_balance = min(user_lp, 0.4 * user_lp + 0.6 * total_lp * user_ve / total_ve)
    // boost = boosted_balance / (0.4 * user_lp)

    let base = (user_lp_amount as u128) * 4000 / 10000; // 0.4 × user_lp
    let bonus = (total_lp_amount as u128) * 6000 / 10000
        * (user_ve_power as u128) / (total_ve_power as u128); // 0.6 × total_lp × ve_share

    let boosted = base + bonus;
    let user_lp = user_lp_amount as u128;
    let capped = boosted.min(user_lp); // Cap at user's LP amount

    // Boost = capped / base, scaled to bps
    let boost_bps = if base > 0 {
        (capped * 10000 / base).min(25000) as u64
    } else {
        10000
    };

    boost_bps.max(10000) // Minimum 1.0x
}
```

---

## 11. References

1. **Curve Finance whitepaper**: The original ve-tokenomics specification
2. **Convex Finance documentation**: Meta-governance and CVX mechanics
3. **"Evaluating the ve-Model" — Delphi Digital (2022)**: Comprehensive analysis of ve-tokenomics
4. **Votium / Hidden Hand**: Bribe marketplace documentation
5. **"The Curve Wars" — various blog posts (2021-2022)**: Historical narrative of the CRV competition
6. **Solana staking documentation**: Inflation schedule, validator economics
7. **Ethereum Beacon Chain spec**: PoS staking mechanics, slashing conditions
8. **"Mechanism Design and Blockchains" — Tim Roughgarden**: Academic foundation for token incentive design
9. **Marinade Finance documentation**: Solana liquid staking and MNDE governance
10. **Pendle Finance**: ve-tokenomics on a yield trading protocol

---

*Next: [06 - Governance Tokenomics](./06-governance-tokenomics.md) — DAOs, voting mechanisms, delegation, on-chain vs. off-chain governance, governance attacks, and building governance programs on Solana.*
