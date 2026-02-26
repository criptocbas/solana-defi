# Token Supply Mechanics: Fixed, Inflationary, and Deflationary Models

> Written for experienced Solana developers entering token design.
> Last updated: February 2026

---

## Table of Contents

1. [Supply Models Overview](#1-supply-models-overview)
2. [Fixed Supply Tokens](#2-fixed-supply-tokens)
3. [Inflationary Supply Tokens](#3-inflationary-supply-tokens)
   - [3.1 Linear Emission](#31-linear-emission)
   - [3.2 Exponential Decay (Halving)](#32-exponential-decay-halving)
   - [3.3 Tail Emission](#33-tail-emission)
   - [3.4 Demand-Responsive Emission](#34-demand-responsive-emission)
4. [Deflationary Mechanisms](#4-deflationary-mechanisms)
   - [4.1 Burn on Transaction](#41-burn-on-transaction)
   - [4.2 Buyback and Burn](#42-buyback-and-burn)
   - [4.3 Buyback and Distribute](#43-buyback-and-distribute)
   - [4.4 Fee Burns](#44-fee-burns)
5. [Hybrid Models](#5-hybrid-models)
6. [Emission Schedules and Math](#6-emission-schedules-and-math)
7. [Rebasing Tokens](#7-rebasing-tokens)
8. [Mint Authority and Supply Control on Solana](#8-mint-authority-and-supply-control-on-solana)
9. [Supply Mechanics Case Studies](#9-supply-mechanics-case-studies)
10. [References](#10-references)

---

## 1. Supply Models Overview

Every token has a supply model that defines how many tokens exist, how that number changes over time, and who controls the change. The supply model is arguably the single most important tokenomics decision because it determines the long-term inflation/deflation trajectory.

### The Supply Spectrum

```
    Fixed Supply              Disinflationary              Inflationary
    ◄─────────────────────────────────────────────────────────────────►
    BTC (21M cap)            ETH (post-merge,             DOGE (5B/year
    UNI (1B, no more)        sometimes deflationary)      perpetually)
                              SOL (decreasing inflation)
```

| Model | Supply Over Time | Price Pressure | When to Use |
|---|---|---|---|
| Fixed | Constant (or decreasing via burns) | Deflationary by default | Store of value, finished distribution |
| Disinflationary | Growing, but rate decreases | Moderating inflation | PoS networks, gradual transition |
| Inflationary | Growing at constant or increasing rate | Persistent sell pressure | Security budgets, ongoing distribution |
| Deflationary | Shrinking (burns > mints) | Buy pressure / scarcity | Revenue-funded burns, fee mechanisms |

---

## 2. Fixed Supply Tokens

### Definition

A fixed supply token has a predetermined maximum supply that will never be exceeded. Once all tokens are minted, no new tokens can be created.

### Examples

| Token | Max Supply | Current Supply | % Minted |
|---|---|---|---|
| BTC | 21,000,000 | ~19,800,000 | ~94% |
| UNI | 1,000,000,000 | 1,000,000,000 | 100% |
| YFI | 36,666 | 36,666 | 100% |

### Advantages

1. **Scarcity is credible**: Users know their ownership percentage cannot be diluted beyond a known maximum
2. **Simple to understand**: No complex emission math, no inflation calculations
3. **Store of value narrative**: Fixed supply enables "digital gold" narrative (Bitcoin)
4. **No dilution risk**: Token holders are not diluted by new emissions

### Disadvantages

1. **No security budget post-distribution**: Once all tokens are distributed, how do you incentivize validators/miners? Bitcoin's answer is transaction fees, but this is unproven at scale.
2. **No flexibility**: Cannot create new incentives, liquidity mining programs, or grants without reusing existing tokens
3. **Concentration risk**: If initial distribution is poor, there is no mechanism to redistribute
4. **Bootstrapping problem**: Hard to incentivize early adoption if all tokens are already allocated

### Implementation on Solana

On Solana, a fixed supply token is created by minting the total supply at creation, then either:

**Option A**: Revoking mint authority (permanent, irreversible)
```rust
// After minting total supply, revoke mint authority
let ix = spl_token::instruction::set_authority(
    &spl_token::id(),
    mint_pubkey,
    None,                    // New authority = None (revoked)
    AuthorityType::MintTokens,
    current_authority_pubkey,
    &[],
)?;
```

**Option B**: Setting mint authority to a PDA with no mint instruction (program-enforced cap)
```rust
// The program simply never exposes an instruction to mint more
// The mint authority is a PDA controlled by the program
// No mint instruction = no new tokens possible
```

Option A is stronger (trustless, on-chain proof of fixed supply). Option B is more flexible (the program could theoretically be upgraded to add minting — unless the program is also immutable).

---

## 3. Inflationary Supply Tokens

### 3.1 Linear Emission

**Definition**: A constant number of tokens emitted per time period, forever.

```
Supply(t) = Initial_Supply + (Emission_Rate × t)

Where:
  t = time since genesis
  Emission_Rate = constant tokens per period
```

**Inflation rate over time**: Even though emission is constant, the inflation *rate* decreases because the denominator (total supply) grows:

```
Inflation_Rate(t) = Emission_Rate / Supply(t)
                  = Emission_Rate / (Initial + Emission_Rate × t)
```

As t → ∞, inflation rate → 0. This is called **disinflationary** in practice.

**Example — Dogecoin**:
```
Initial supply: 100 billion DOGE (fully mined by 2015)
Emission: ~5 billion DOGE per year (perpetually)
Year 1:  5B / 105B = 4.76% inflation
Year 5:  5B / 125B = 4.00% inflation
Year 20: 5B / 200B = 2.50% inflation
Year 100: 5B / 600B = 0.83% inflation
```

**Advantages**: Simple, predictable, and the perpetual emission provides a permanent security budget. The inflation rate decreases over time without any halvings or complex mechanisms.

**Disadvantages**: Never truly reaches zero inflation. Can feel "inflationary" to holders even as the rate decreases.

### 3.2 Exponential Decay (Halving)

**Definition**: Emission rate is cut by a fixed percentage at regular intervals.

```
Emission(epoch) = Initial_Emission × (1 - decay_rate)^epoch

For Bitcoin (50% decay every ~4 years):
  Emission(epoch) = 50 × 0.5^epoch BTC per block
```

**Total supply converges**:
```
Max_Supply = Initial_Emission × Block_Per_Epoch / decay_rate
For Bitcoin: approaches 21,000,000
```

**Supply curve**:
```
Year 0:    0 BTC
Year 4:   10.5M BTC (50%)     ← Half of all BTC mined in first 4 years
Year 8:   15.75M BTC (75%)
Year 12:  18.375M BTC (87.5%)
Year 16:  19.69M BTC (93.75%)
Year 20:  20.34M BTC (96.88%)
...
Year 140: 21.0M BTC (100%)
```

**Key property**: The halving model front-loads distribution. Over 80% of tokens are distributed in the first third of the emission period. This creates strong early incentives but diminishing rewards over time.

### 3.3 Tail Emission

**Definition**: After the main emission schedule completes, a small perpetual emission continues indefinitely. This ensures a permanent minimum inflation rate.

**Example — Monero**:
```
Main emission: Block reward decreasing according to formula
Tail emission: 0.6 XMR per block, perpetually (started June 2022)
```

**Rationale**: Without tail emission, the network's security budget depends entirely on transaction fees. Tail emission guarantees a minimum level of miner/validator compensation regardless of fee market conditions.

**Solana's approach**: Solana uses a disinflationary model with an 8% initial inflation rate, decreasing 15% per year, targeting a long-term rate of 1.5%:

```
Year 0: 8.0%
Year 1: 6.8%   (8.0 × 0.85)
Year 2: 5.78%  (6.8 × 0.85)
Year 3: 4.91%
Year 5: 3.55%
Year 10: 1.89%
Year 15: 1.50% (floor reached)
Year 20+: 1.5% perpetually
```

This is effectively a tail emission model — inflation never reaches zero but stabilizes at 1.5%. However, Solana also burns 50% of transaction fees, so net inflation = emission - burns. During periods of high network activity, SOL can become temporarily deflationary.

### 3.4 Demand-Responsive Emission

**Definition**: Emission rate adjusts based on protocol metrics (usage, demand, price).

**Example — Helium (pre-migration)**:
- HNT emission fixed per epoch
- But allocation between hotspot operators, network users, and investors changes based on protocol usage
- When data transfer increases, more HNT flows to hotspot operators (supply side)
- When data transfer is low, more HNT flows to investors (demand side)

**Example — Algorithmic stablecoin emission**:
```
If stablecoin_price > $1:
  Mint new stablecoins (expansionary)
  Distribute to stakers/bondholders

If stablecoin_price < $1:
  Reduce supply (contractionary)
  Incentivize burn via bonds/discounts
```

**Risk**: Demand-responsive emission requires accurate oracles and can amplify volatility if the feedback loop is poorly calibrated (see: Terra/Luna death spiral).

---

## 4. Deflationary Mechanisms

### 4.1 Burn on Transaction

**Mechanism**: A percentage of every transaction is destroyed, permanently reducing supply.

```
Effective Transfer = Amount × (1 - burn_rate)
Burned = Amount × burn_rate
```

**Example — Token-2022 Transfer Fee**:
Solana's Token-2022 standard has a native transfer fee extension. A portion of every transfer is automatically collected and can be programmatically burned:

```rust
// Token-2022 mint with transfer fee
// On every transfer of 1000 tokens with 1% fee:
// - Recipient receives: 990 tokens
// - Fee account receives: 10 tokens
// - Protocol burns from fee account periodically
```

**Advantages**: Deflationary pressure is proportional to usage — more transactions = more burns. This directly ties scarcity to adoption.

**Disadvantages**: High burn rates penalize users and discourage transactions. The "tax" makes the token less useful as a medium of exchange.

### 4.2 Buyback and Burn

**Mechanism**: Protocol uses earned revenue to buy tokens on the open market, then burns them.

```
Revenue → Buy tokens on DEX → Send to burn address → Supply decreases
```

**Example — BNB**:
- Binance uses 20% of quarterly profits to buy and burn BNB
- Target: reduce supply from 200M to 100M
- As of 2026, approximately 50M BNB have been burned

**Example — MakerDAO**:
- When DAI surplus buffer is full, excess revenue buys and burns MKR
- More DAI loans = more stability fees = more MKR burned
- During DeFi boom: significant MKR burned
- During bad debt events: MKR is minted to cover losses (supply can increase too)

```
MKR Supply Change = Burned (from surplus) - Minted (for bad debt)

If protocol is profitable: supply decreases (deflationary)
If protocol has bad debt: supply increases (inflationary)
```

This is elegant — MKR holders benefit when the protocol is well-managed and suffer when it is not. The token's supply dynamics enforce accountability.

### 4.3 Buyback and Distribute

**Mechanism**: Instead of burning bought-back tokens, distribute them to stakers or holders.

```
Revenue → Buy tokens on DEX → Distribute to stakers → Stakers earn yield
```

**Example — GMX**:
- 30% of trading fees go to GMX stakers (paid in ETH/AVAX, not GMX)
- An additional portion buys GLP which benefits liquidity providers
- No burning, but real revenue distribution creates sustainable demand

**Advantages over burn**: Revenue distribution gives token holders direct cash flow, making the token easier to value using DCF models. Burn only helps through scarcity; distribution helps through cash flow.

**Disadvantages**: Creates tax events for holders in many jurisdictions. Might classify the token as a security.

### 4.4 Fee Burns

**Mechanism**: Protocol fees are denominated in the native token, and a portion is burned.

**Example — Ethereum EIP-1559**:
```
Transaction fee = Base fee + Priority tip

Base fee: BURNED (removed from circulation)
Priority tip: Goes to validators

If burns > new issuance: ETH is deflationary (net supply decrease)
If burns < new issuance: ETH is inflationary (net supply increase)
```

**Post-merge ETH supply dynamics**:
```
Issuance: ~1,700 ETH/day (PoS validator rewards)
Burns:    Varies with network activity
  - High activity: 3,000-10,000+ ETH/day burned → net deflationary
  - Low activity:  500-1,500 ETH/day burned → net inflationary
```

This creates a system where ETH is deflationary during periods of high demand and slightly inflationary during periods of low demand — a natural monetary policy that adjusts without human intervention.

**Solana's fee burn**:
```
Transaction fee:
  50% burned
  50% goes to the validator that processed the transaction
```

Additionally, Solana has priority fees (tips to validators) that are not burned. The net effect depends on the ratio of base fees to total inflation.

---

## 5. Hybrid Models

Most modern tokens use hybrid models that combine multiple supply mechanics.

### ETH: The "Ultrasound Money" Model

```
Supply sources:
  + PoS issuance: ~1,700 ETH/day (inflationary)
  - EIP-1559 burns: variable (deflationary)
  = Net: deflationary during high usage, slightly inflationary during low usage

Additional mechanics:
  - Staking locks (33M+ ETH staked, ~27% of supply)
  - DeFi locking (ETH used as collateral)
  - Beacon chain withdrawal queue (slows unlocking)
```

### SOL: Disinflationary with Fee Burns

```
Supply sources:
  + Staking inflation: decreasing from 8% toward 1.5% floor
  - Fee burns: 50% of base transaction fees
  = Net: decreasing inflation rate, approaching equilibrium

Additional mechanics:
  - ~65% of SOL is staked (locked, reducing circulating supply)
  - Validator economics (commission rates)
  - Priority fee market (not burned, goes to validators)
```

### CRV: Emission with Lock Incentives

```
Supply sources:
  + CRV emissions: decreasing over time (initially 2M/day, halving annually)
  - Burns: minimal
  + Voting: veCRV locks remove from circulation (up to 4 years)
  = Net: high nominal inflation, but massive lock-up reduces effective circulating supply

Key insight: ~50% of all CRV is locked as veCRV (average lock ~3.5 years)
Effective circulating inflation ≈ Emission rate × (1 - lock_percentage)
```

### JUP: Airdrop-Heavy Distribution with Active Supply Management (ASM)

```
Total supply: 10 billion JUP
Distribution:
  - 50% to community (airdrops, grants, ecosystem)
  - 50% to team/treasury

Supply management:
  - Active Supply Management (ASM): Jupiter DAO votes on burning unallocated tokens
  - January 2025: Voted to burn 3B JUP (~30% of total supply)
  - Reduces FDV and concentrates value to existing holders
```

---

## 6. Emission Schedules and Math

### Linear Emission with Decay

```rust
/// Compute tokens to emit in a given epoch
/// emission_rate decays by `decay_pct` each epoch
fn emission_for_epoch(
    initial_rate: u64,    // tokens per epoch at start
    epoch: u64,           // current epoch number
    decay_bps: u64,       // decay per epoch in basis points (e.g., 1500 = 15%)
) -> u64 {
    let mut rate = initial_rate as u128;
    for _ in 0..epoch {
        rate = rate * (10000 - decay_bps as u128) / 10000;
    }
    rate as u64
}

// Example: Solana-style
// Initial: 8% of supply, decay 15%/year
// Year 0: 8.00%
// Year 1: 6.80%
// Year 5: 3.55%
// Year 15+: 1.50% (floor)
```

### Halving Schedule

```rust
/// Bitcoin-style halving
fn block_reward(block_height: u64) -> u64 {
    let initial_reward: u64 = 50_0000_0000; // 50 BTC in satoshis
    let halving_interval: u64 = 210_000;    // blocks between halvings
    let halvings = block_height / halving_interval;

    if halvings >= 64 {
        return 0; // Reward becomes 0 after 64 halvings
    }

    initial_reward >> halvings // Right shift = divide by 2^halvings
}

// Block 0:       50 BTC
// Block 210,000: 25 BTC
// Block 420,000: 12.5 BTC
// ...
```

### Continuous Emission (e-based)

Some protocols use continuous (exponential) emission rather than discrete halvings:

```
Supply(t) = Max_Supply × (1 - e^(-λt))

Where:
  λ = emission rate constant
  t = time since genesis

Emission_Rate(t) = λ × Max_Supply × e^(-λt)
```

This produces a smooth curve instead of discrete halvings. The emission rate decreases continuously rather than in sudden steps.

### Cumulative Supply Calculator

```rust
/// Calculate total supply at a given point in time
/// for a halving-based emission schedule
fn total_supply_at_block(target_block: u64) -> u64 {
    let initial_reward: u64 = 50_0000_0000;
    let halving_interval: u64 = 210_000;
    let mut total: u64 = 0;
    let mut current_block: u64 = 0;

    while current_block < target_block {
        let halvings = current_block / halving_interval;
        if halvings >= 64 { break; }

        let reward = initial_reward >> halvings;
        let blocks_in_era = halving_interval - (current_block % halving_interval);
        let blocks_remaining = target_block - current_block;
        let blocks_to_count = blocks_in_era.min(blocks_remaining);

        total += reward * blocks_to_count;
        current_block += blocks_to_count;
    }

    total
}
```

---

## 7. Rebasing Tokens

### Definition

Rebasing tokens automatically adjust every holder's balance to maintain a target price or grow proportionally. Instead of price changing, supply changes.

### Positive Rebase (Inflationary)

```
If market_price > target_price:
  new_supply = old_supply × (market_price / target_price)

Every holder's balance increases proportionally.
```

**Example**: Ampleforth (AMPL) targets $1. If AMPL trades at $1.50, everyone's balance increases by 50%. If it trades at $0.50, everyone's balance decreases by 50%.

### Negative Rebase (Deflationary)

```
If market_price < target_price:
  new_supply = old_supply × (market_price / target_price)

Every holder's balance decreases proportionally.
```

### Why Rebasing Is Mostly Abandoned

1. **Tax nightmares**: Every rebase is a taxable event in most jurisdictions
2. **Confusing UX**: Users see their balance change daily and panic
3. **DeFi incompatibility**: Lending protocols, DEXes, and vaults struggle with fluctuating balances
4. **Doesn't actually solve the problem**: Market cap (supply × price) changes regardless — rebasing just shifts volatility from price to balance

**Modern alternative**: Share-based tokens (like stETH or cTokens) where the token balance stays constant but the exchange rate changes. This achieves the same economic effect (growing value) without the rebasing mechanics.

```
Rebasing: balance grows, price stable (in theory)
Share-based: balance stable, price grows

Market cap behavior is identical. Share-based is simpler.
```

---

## 8. Mint Authority and Supply Control on Solana

### SPL Token Mint Authority

On Solana, every SPL token mint has an optional `mint_authority` — the account authorized to mint new tokens.

```rust
// SPL Token Mint structure (simplified)
pub struct Mint {
    pub mint_authority: COption<Pubkey>,  // Who can mint
    pub supply: u64,                       // Current total supply
    pub decimals: u8,                      // Token decimals
    pub is_initialized: bool,
    pub freeze_authority: COption<Pubkey>, // Who can freeze accounts
}
```

### Supply Control Patterns

**Pattern 1: Revoked Authority (Fixed Supply)**
```rust
// Mint all tokens at creation, then revoke
spl_token::instruction::set_authority(
    &spl_token::id(),
    &mint_pubkey,
    None,  // Revoke — no one can ever mint again
    AuthorityType::MintTokens,
    &authority_pubkey,
    &[],
)?;
// Supply is now permanently fixed
```

**Pattern 2: Program-Controlled Authority (Programmatic Supply)**
```rust
// Mint authority is a PDA controlled by a program
// The program defines the emission rules
#[account(
    mut,
    mint::authority = emission_authority,
)]
pub token_mint: Account<'info, Mint>,

#[account(
    seeds = [b"emission_authority"],
    bump,
)]
/// CHECK: PDA that controls minting
pub emission_authority: UncheckedAccount<'info>,

// Only the program's emission instruction can mint:
pub fn emit(ctx: Context<Emit>) -> Result<()> {
    let clock = Clock::get()?;
    let tokens_to_emit = calculate_emission(clock.unix_timestamp);

    // Mint via CPI with PDA signer
    let seeds = &[b"emission_authority", &[ctx.bumps.emission_authority]];
    let signer = &[&seeds[..]];

    token::mint_to(
        CpiContext::new_with_signer(
            ctx.accounts.token_program.to_account_info(),
            MintTo {
                mint: ctx.accounts.token_mint.to_account_info(),
                to: ctx.accounts.recipient.to_account_info(),
                authority: ctx.accounts.emission_authority.to_account_info(),
            },
            signer,
        ),
        tokens_to_emit,
    )?;
    Ok(())
}
```

**Pattern 3: Multisig Authority (Semi-centralized)**
```
Mint authority = Squads multisig (e.g., 3-of-5 team members)
Pro: Flexibility to mint for grants, incentives, emergencies
Con: Centralized trust — multisig can inflate supply
```

**Pattern 4: Governance-Controlled Authority**
```
Mint authority = SPL Governance (Realms) PDA
Minting requires a governance proposal + quorum vote
Pro: Community-controlled supply
Con: Slow (governance proposals take days), governance attack risk
```

### Token-2022 Supply Features

Token-2022 adds extensions relevant to supply mechanics:

| Extension | Supply Impact | Use Case |
|---|---|---|
| **Transfer Fee** | Can burn fee portion → deflationary | Protocol revenue + burn |
| **Non-Transferable** | Soulbound tokens — no secondary market | Credentials, reputation |
| **Permanent Delegate** | Authority can burn/transfer any holder's tokens | Regulated tokens, emergency controls |
| **Interest-Bearing** | Display balance grows (cosmetic, not real minting) | Staking receipt tokens |
| **Confidential Transfer** | Hides amounts but supply is still public | Privacy-preserving tokens |

### Burn Implementation on Solana

```rust
// Burn tokens (reduce supply)
token::burn(
    CpiContext::new(
        ctx.accounts.token_program.to_account_info(),
        Burn {
            mint: ctx.accounts.token_mint.to_account_info(),
            from: ctx.accounts.token_account.to_account_info(),
            authority: ctx.accounts.owner.to_account_info(),
        },
    ),
    burn_amount,
)?;
// token_mint.supply automatically decreases
```

---

## 9. Supply Mechanics Case Studies

### Case Study 1: Bitcoin — The Gold Standard

```
Model: Fixed supply, exponential decay emission (halving)
Total: 21,000,000 BTC
Current: ~19.8M minted (~94%)
Inflation: ~0.85% annually (2024-2028 epoch)
Post-2140: 0% inflation, fee-funded security
```

**What works**: Absolute scarcity creates a powerful narrative. The halving events generate attention cycles that correlate with bull markets. The simplicity of "21 million, ever" is perhaps the most successful tokenomics meme in history.

**What's uncertain**: Will transaction fees alone sustain miner security post-emission? The fee market is currently insufficient (fees are <5% of miner revenue in most blocks).

### Case Study 2: ETH — Adaptive Supply

```
Model: Hybrid — PoS issuance + EIP-1559 burns
Issuance: ~1,700 ETH/day (PoS rewards)
Burns: Variable (1,000-10,000+ ETH/day)
Net: -0.2% to +0.5% annually depending on network activity
```

**What works**: The "ultrasound money" narrative leverages the unpredictable but often deflationary supply to position ETH as a superior store of value to Bitcoin (which has permanent, albeit diminishing, inflation). The fee burn creates a direct link between network usage and scarcity.

**Nuance**: ETH is only deflationary during periods of high demand. During bear markets or low activity periods, net issuance is positive.

### Case Study 3: SOL — Staking-Centric Inflation

```
Model: Disinflationary with fee burns
Current inflation: ~5.2% (Feb 2026)
Target floor: 1.5%
Staking: ~65% of supply staked
Fee burn: 50% of base fees
```

**What works**: The high staking ratio (65%+) means most inflation goes back to stakers. Non-stakers are diluted, which incentivizes staking. The fee burn creates a path to lower effective inflation as network activity grows.

**The "real" inflation for stakers**:
```
If you stake: Real dilution ≈ 0% (staking rewards offset inflation)
If you don't stake: Real dilution ≈ 5.2% (you're being diluted by staker rewards)
```

This creates a strong incentive to stake, which locks supply, which reduces selling pressure, which supports price. It's an elegant self-reinforcing loop.

### Case Study 4: CRV — High Emission, High Lock-up

```
Model: High initial emission, decreasing over time, massive ve-locking
Initial emission: ~2M CRV/day
Current emission: reduced significantly from peak
Total supply: 3.03B CRV
veCRV locked: ~50% of supply (average lock ~3.5 years)
```

**What works**: Despite enormous nominal emission rates, the ve-lock mechanism removes so many tokens from circulation that effective sell pressure is manageable. The lock creates commitment — veCRV holders can't sell during downturns, reducing panic selling.

**What's risky**: CRV emissions are still very high. The system depends on continued demand for veCRV governance power (gauge voting). If the "Curve Wars" (protocols competing for gauge votes) cool down, demand for locking could decrease.

### Case Study 5: BONK — Meme Supply at Scale

```
Model: Fixed supply, community distribution
Total: 93.5 trillion BONK (after burns)
Original: 100 trillion
Burned: ~6.5 trillion
Distribution: 50% airdropped to Solana community
```

**What works**: The enormous supply (trillions) means each token is worth a tiny fraction of a cent, making it psychologically accessible ("I own millions of BONK!" feels better than "I own 0.001 BTC"). The airdrops created wide distribution, and community initiatives have created real utility (BONK integrations across Solana ecosystem).

**Design lesson**: Supply number is cosmetic. A token with 100 trillion supply at $0.00001 each has the same market cap as a token with 100 supply at $10M each. But psychology matters — small numbers per token feel "cheap" and attract retail buyers.

---

## 10. References

1. **"Bitcoin: A Peer-to-Peer Electronic Cash System" — Satoshi Nakamoto (2008)**: The first emission schedule
2. **EIP-1559 specification**: Fee burn mechanism for Ethereum
3. **Solana tokenomics documentation**: Inflation schedule, fee burn mechanics
4. **"Monetary Policy for the 21st Century" — research on algorithmic monetary policies**
5. **Ampleforth whitepaper**: The canonical rebasing token design
6. **SPL Token documentation**: Mint authority, burn, freeze mechanics
7. **Token-2022 specification**: Transfer fees, confidential transfers, extensions
8. **"The Problem with Token Velocity" — Kyle Samani (Multicoin, 2017)**: Why high-velocity tokens accrue less value

---

*Next: [03 - Token Distribution and Launch Strategies](./03-token-distribution-and-launch.md) — ICOs, IDOs, airdrops, vesting, fair launches, and how distribution determines a token's fate.*
