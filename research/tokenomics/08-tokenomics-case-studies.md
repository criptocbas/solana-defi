# Tokenomics Case Studies: Successes and Failures

> Written for experienced Solana developers entering token design.
> Last updated: February 2026

---

## Table of Contents

1. [How to Evaluate Tokenomics](#1-how-to-evaluate-tokenomics)
2. [Bitcoin (BTC) — The Original Standard](#2-bitcoin-btc--the-original-standard)
3. [Ethereum (ETH) — Adaptive Monetary Policy](#3-ethereum-eth--adaptive-monetary-policy)
4. [Solana (SOL) — Staking-Centric Design](#4-solana-sol--staking-centric-design)
5. [Curve (CRV) — The ve-Token Pioneer](#5-curve-crv--the-ve-token-pioneer)
6. [MakerDAO (MKR) — Governance + Buyback](#6-makerdao-mkr--governance--buyback)
7. [Aave (AAVE) — Safety Module Staking](#7-aave-aave--safety-module-staking)
8. [Uniswap (UNI) — The Fee Switch Dilemma](#8-uniswap-uni--the-fee-switch-dilemma)
9. [Jupiter (JUP) — Community-Governed Supply](#9-jupiter-jup--community-governed-supply)
10. [Jito (JTO) — MEV-Aligned Staking](#10-jito-jto--mev-aligned-staking)
11. [Chainlink (LINK) — Work Token Model](#11-chainlink-link--work-token-model)
12. [Cautionary Tales](#12-cautionary-tales)
    - [12.1 FTT — The Collateral House of Cards](#121-ftt--the-collateral-house-of-cards)
    - [12.2 LUNA/UST — The Death Spiral](#122-lunaust--the-death-spiral)
    - [12.3 OHM — Unsustainable APY](#123-ohm--unsustainable-apy)
13. [Comparative Analysis](#13-comparative-analysis)
14. [References](#14-references)

---

## 1. How to Evaluate Tokenomics

### The Evaluation Framework

| Dimension | What to Check | Red Flag | Green Flag |
|---|---|---|---|
| **Supply** | Max supply, inflation rate, emission schedule | Unlimited with no burns | Fixed or net-deflationary |
| **Distribution** | Team %, community %, vesting schedules | >40% to insiders, short vesting | >50% community, multi-year vesting |
| **Utility** | What can you DO with the token? | "Payment only" or governance-only | Multiple demand drivers |
| **Value Accrual** | Does the token capture protocol value? | No fee sharing, no burns | Revenue sharing or burns |
| **Governance** | How are decisions made? | Team controls everything | Active community governance |
| **Incentive Alignment** | Do insiders and community benefit together? | Insiders can dump before community | Long vesting, skin in the game |

### The Quick Score

For each dimension, rate 1-5:

```
Strong tokenomics: 25-30 total
Average:           18-24 total
Weak:              12-17 total
Red flag:          <12 total
```

---

## 2. Bitcoin (BTC) — The Original Standard

### Tokenomics Profile

| Dimension | Design | Score |
|---|---|---|
| **Supply** | 21M fixed, halving every ~4 years | 5/5 |
| **Distribution** | 100% mined (no pre-mine) | 5/5 |
| **Utility** | Store of value, payment, collateral | 4/5 |
| **Value Accrual** | Scarcity (fixed supply, lost coins) | 4/5 |
| **Governance** | Off-chain (BIPs, social consensus) | 3/5 |
| **Incentive Alignment** | All participants benefit from adoption | 5/5 |
| **Total** | | **26/30** |

### What Makes BTC Tokenomics Strong

**1. Absolute scarcity**: 21M is the most credible supply cap in crypto. Changing it would require consensus from miners, nodes, developers, and users — effectively impossible.

**2. Fair launch**: No pre-mine means no insider advantage. Satoshi mined early blocks but so could anyone else. The playing field was level from day one.

**3. Simple and predictable**: Anyone can calculate the exact supply at any future date. No complex emission formulas, no governance decisions about supply.

**4. Halving creates event-driven attention**: The ~4-year halving cycle creates natural marketing events and supply shock narratives.

### Weaknesses

**1. Security budget uncertainty**: As block rewards approach zero, Bitcoin must rely on transaction fees for miner security. Current fee revenue is ~5% of miner income — far from sufficient.

**2. No on-chain governance**: Changes require off-chain social consensus, which is slow and sometimes contentious (see: block size wars, SegWit debates).

**3. No DeFi utility (native)**: Bitcoin's scripting language limits DeFi. Layer 2s (Lightning, Stacks) attempt to add utility but are separate systems.

### The Halving Impact

```
Pre-halving supply shock thesis:
  Halving reduces new BTC supply by 50%
  If demand is constant, reduced supply → price increase
  Historical data supports this (correlation, not necessarily causation):

  2012 halving: $12 → $1,100 (12 months later)
  2016 halving: $650 → $20,000 (18 months later)
  2020 halving: $9,000 → $69,000 (18 months later)
  2024 halving: $64,000 → price discovery ongoing
```

---

## 3. Ethereum (ETH) — Adaptive Monetary Policy

### Tokenomics Profile

| Dimension | Design | Score |
|---|---|---|
| **Supply** | No cap, but EIP-1559 burns + PoS = near-zero net issuance | 4/5 |
| **Distribution** | ICO (60%) + mining/staking (40%) | 3/5 |
| **Utility** | Gas, staking, collateral, DeFi composability | 5/5 |
| **Value Accrual** | Fee burns, staking yield, collateral demand | 5/5 |
| **Governance** | Off-chain (EIPs, rough consensus) | 3/5 |
| **Incentive Alignment** | Stakers, users, and developers all benefit from usage | 5/5 |
| **Total** | | **25/30** |

### The Three ETH Monetary Phases

**Phase 1: PoW Era (2015-2022)**: ~4% annual inflation from mining rewards. ETH was clearly inflationary with no burn mechanism. Miners received all transaction fees + block rewards.

**Phase 2: EIP-1559 (August 2021)**: Base fee burned, priority fee to miners. First introduction of deflationary pressure. During high-usage periods, burns exceeded issuance — ETH temporarily became deflationary.

**Phase 3: The Merge (September 2022)**: PoW → PoS reduced issuance by ~90% (from ~13,000 ETH/day to ~1,700 ETH/day). Combined with EIP-1559 burns, ETH became frequently deflationary. The "ultrasound money" narrative was born.

### ETH Demand Stack Analysis

```
1. Gas payment:     ████████████████████  (Required for every transaction)
2. Staking:         ██████████████████    (32 ETH per validator, ~27% staked)
3. DeFi collateral: ████████████████      (Most used collateral in DeFi)
4. Fee burns:       ██████████████        (EIP-1559 removes supply)
5. L2 settlement:   ████████████          (L2s need ETH for data posting)
6. Cultural asset:  ██████████            ("Ultrasound money" narrative)

Demand breadth: 6 layers → extremely robust
```

### Key Innovation: Adaptive Monetary Policy

ETH's monetary policy automatically adjusts to network demand:

```
High demand → High fees → More burns → Deflationary → Supply tightens
Low demand  → Low fees  → Less burns → Inflationary  → Supply loosens

This is the monetary policy equivalent of a thermostat:
  Network hot → cool it down (burn more, supply shrinks)
  Network cold → warm it up (burn less, supply grows slightly)
```

No governance vote required. No human intervention. The mechanism is algorithmic and self-adjusting.

---

## 4. Solana (SOL) — Staking-Centric Design

### Tokenomics Profile

| Dimension | Design | Score |
|---|---|---|
| **Supply** | Disinflationary (8% → 1.5% floor), 50% fee burn | 4/5 |
| **Distribution** | Foundation (12.5%), team (12.8%), investors (38%), community (36.7%) | 3/5 |
| **Utility** | Gas, staking, DeFi collateral, governance (informal) | 5/5 |
| **Value Accrual** | Fee burn, staking demand, MEV (via Jito) | 4/5 |
| **Governance** | Off-chain (SIMD proposals, validator signaling) | 3/5 |
| **Incentive Alignment** | Validators, stakers, users aligned through staking | 4/5 |
| **Total** | | **23/30** |

### SOL Economic Model

```
Annual token flow (approximate, Feb 2026):

  Inflows (inflationary):
    + Staking inflation: ~5.2% × ~590M SOL = ~30.7M SOL/year minted

  Outflows (deflationary):
    - Fee burns: ~50% of base fees burned
    - Net burns depend on network activity
    - High activity periods: significant burns

  Distribution of inflation:
    65% staked → stakers receive proportional inflation rewards
    35% not staked → diluted by inflation

  Effective yield:
    Stakers: ~7.5% gross - 5.2% inflation = ~2.3% real yield
    Non-stakers: -5.2% (dilution only)
```

### Liquid Staking's Role in SOL Tokenomics

```
Liquid staking tokens (mSOL, jitoSOL, bSOL) solve the staking trilemma:

  Without liquid staking:
    - 65% staked → secure but illiquid
    - Only 35% available for DeFi
    - Trade-off between security and DeFi activity

  With liquid staking:
    - 65% staked AND available for DeFi (as LSTs)
    - LSTs used as collateral, LP, etc.
    - No security vs. liquidity trade-off

  Jito's innovation:
    jitoSOL includes MEV rewards → higher yield than vanilla staking
    Additional ~1-2% from MEV → makes staking more attractive
    More staking → more security → virtuous cycle
```

### Distribution Criticism

SOL's initial distribution is its weakest point:

```
Initial allocation:
  Investors: 38%  ← Very high
  Team: 12.8%
  Foundation: 12.5%
  Community: 36.7%

  Insiders (team + investors): >50%
  This is higher than most modern launches target

  Mitigating factors:
  - Long vesting schedules (completed for most allocations)
  - Strong ecosystem growth validated the distribution
  - Liquid staking redistributes economic rights
  - Fee burn benefits all holders equally
```

---

## 5. Curve (CRV) — The ve-Token Pioneer

### Tokenomics Profile

| Dimension | Design | Score |
|---|---|---|
| **Supply** | 3.03B max, decreasing emissions | 3/5 |
| **Distribution** | 62% to community LPs, 30% team/investors, 5% reserves | 4/5 |
| **Utility** | Governance (gauge voting), boost, fee sharing | 5/5 |
| **Value Accrual** | 50% of fees to veCRV, gauge voting power | 5/5 |
| **Governance** | veCRV voting (time-weighted, decaying) | 5/5 |
| **Incentive Alignment** | Long locks = alignment, Curve Wars = demand | 4/5 |
| **Total** | | **26/30** |

### Why CRV Tokenomics Is Influential

CRV created the most sophisticated tokenomics model in DeFi. Every element reinforces every other element:

```
Flywheel:
  1. Lock CRV → Get veCRV
  2. veCRV → Vote on gauge weights (control emissions)
  3. veCRV → Earn 50% of trading fees
  4. veCRV → Boost LP farming by up to 2.5x
  5. Protocols want emissions → Buy CRV → Lock → More veCRV
  6. More locking → Less circulating CRV → Scarcity → Price support
  7. Higher CRV price → More valuable emissions → More demand for veCRV

  This is the most self-reinforcing tokenomics loop in DeFi.
```

### The veCRV Lock Distribution

```
Average lock duration: ~3.5 years (out of 4-year max)
% of CRV locked: ~50%
% locked for max duration: ~40% of lockers choose 4 years

This means half of all CRV is effectively illiquid for years.
No other major token has this level of supply lock-up.
```

### Vulnerabilities

**1. Convex concentration**: Convex Finance controls ~50% of all veCRV. This is governance centralization, even if the governance power is theoretically accessible to CVX holders.

**2. High emission rate**: CRV has emitted billions of tokens. Without the lock mechanism, the sell pressure would be catastrophic.

**3. Founder incident (2023)**: Michael Egorov had massive CRV positions used as collateral for loans. A price decline nearly triggered cascading liquidations, threatening the entire Curve ecosystem.

### Protocols That Adopted ve-Tokenomics

| Protocol | Token | Lock | What It Controls |
|---|---|---|---|
| Balancer | veBAL | Up to 1 year | Gauge emissions, fees |
| Frax | veFXS | Up to 4 years | Gauge emissions, protocol direction |
| Pendle | vePENDLE | Up to 2 years | Yield market gauge emissions |
| Velodrome | veVELO | Up to 4 years | Optimism liquidity incentives |
| Aerodrome | veAERO | Up to 4 years | Base chain liquidity incentives |

---

## 6. MakerDAO (MKR) — Governance + Buyback

### Tokenomics Profile

| Dimension | Design | Score |
|---|---|---|
| **Supply** | ~900K, deflationary via burns (but can mint for bad debt) | 4/5 |
| **Distribution** | ICO + gradual community distribution | 3/5 |
| **Utility** | Governance (stability fees, collateral, risk params) | 5/5 |
| **Value Accrual** | Buyback + burn from surplus revenue | 5/5 |
| **Governance** | Most active DeFi governance (MKR voting) | 5/5 |
| **Incentive Alignment** | MKR holders bear both upside (burns) and downside (dilution) | 5/5 |
| **Total** | | **27/30** |

### The MKR Economic Model

```
Revenue source: Stability fees on DAI loans (borrowers pay interest)

Revenue flows:
  1. Stability fees accrue in the Surplus Buffer
  2. When buffer exceeds threshold → Flap Auction
  3. Flap Auction: Surplus DAI buys MKR on market
  4. Bought MKR is burned → supply decreases

  Profitable protocol → MKR supply shrinks → each MKR is worth more

Bad debt protection:
  If a vault is liquidated and the collateral doesn't cover the debt:
  1. Deficit accrues in the system
  2. Flop Auction: NEW MKR is minted and sold for DAI
  3. DAI covers the deficit
  4. MKR supply INCREASES → existing holders are diluted

  Loss-making protocol → MKR supply grows → each MKR is worth less
```

### Why This Is Brilliant

MKR is one of the only tokens where the tokenomics **punish holders for bad governance**:

```
Good governance decisions:
  → Conservative collateral parameters
  → Appropriate risk premiums
  → Good liquidation systems
  → Result: Profitable → MKR burned → holders benefit

Bad governance decisions:
  → Too-risky collateral (accept volatile assets at high LTV)
  → Too-low stability fees (underprice risk)
  → Poor liquidation systems
  → Result: Bad debt → MKR minted → holders diluted

This is the only token where governance skill directly affects supply.
```

### MKR Performance Data

```
MKR supply over time:
  Launch: 1,000,000 MKR
  Peak burn era: ~977,000 MKR (significant burns during DeFi boom)
  Post-bad debt events: ~990,000 MKR (minted to cover losses)
  Current: ~900,000 MKR

  Net: ~10% supply reduction over ~8 years
  This represents ~$300M+ in buyback and burn value
```

---

## 7. Aave (AAVE) — Safety Module Staking

### Tokenomics Profile

| Dimension | Design | Score |
|---|---|---|
| **Supply** | 16M max, majority distributed | 4/5 |
| **Distribution** | Token swap from LEND (100:1), team + ecosystem | 3/5 |
| **Utility** | Governance, Safety Module staking, fee discounts | 5/5 |
| **Value Accrual** | Protocol revenue, Safety Module yields, buybacks | 4/5 |
| **Governance** | Active on-chain governance (Snapshot + on-chain) | 4/5 |
| **Incentive Alignment** | Safety Module creates skin-in-the-game | 5/5 |
| **Total** | | **25/30** |

### The Safety Module Innovation

AAVE's most innovative tokenomics feature is the Safety Module — stakers provide insurance to the protocol:

```
Safety Module Economics:

  Stakers deposit AAVE into Safety Module
  They earn: ~8-12% APY (from protocol revenue + emissions)
  They risk: Up to 30% slashing if protocol has bad debt

  Protocol security:
    If a lending market has bad debt:
    1. Up to 30% of staked AAVE is auctioned
    2. Proceeds cover the deficit
    3. Stakers lose up to 30% of their stake

  This creates a decentralized insurance fund:
    Total staked: ~$500M+ in AAVE
    Maximum coverage: ~$150M (30% of staked)
    Cost to protocol: staking emissions (~$50M/year)
    Insurance premium: effectively ~33% of staked value annually
```

### AAVE vs. Traditional Insurance

```
Traditional: Pay premiums → Insurance company covers losses
  - Centralized counterparty risk
  - Slow claims process
  - Opaque pricing

AAVE Safety Module: Stakers earn yield → Stakers cover losses
  - Decentralized (thousands of stakers)
  - Automatic execution (smart contract)
  - Transparent pricing (APY = insurance premium)
```

---

## 8. Uniswap (UNI) — The Fee Switch Dilemma

### Tokenomics Profile

| Dimension | Design | Score |
|---|---|---|
| **Supply** | 1B fixed, fully distributed | 4/5 |
| **Distribution** | 60% community (retroactive + mining), 40% insiders | 3/5 |
| **Utility** | Governance only (as of Feb 2026) | 2/5 |
| **Value Accrual** | None to UNI holders (fee switch off) | 1/5 |
| **Governance** | Governor Bravo (on-chain), active forums | 4/5 |
| **Incentive Alignment** | Misaligned — protocol generates $500M+ in fees, 0 goes to UNI | 2/5 |
| **Total** | | **16/30** |

### The UNI Paradox

UNI governs the most successful DEX in crypto but captures almost none of its value:

```
Uniswap metrics:
  Annual trading volume: ~$400-800B
  Annual LP fees (0.30%): ~$500M+
  UNI holder share: $0

  Uniswap Labs (company) revenue:
  Frontend fee (0.15% on select pairs): ~$100M+
  This goes to Uniswap Labs, not UNI holders

  Result:
  - Uniswap the protocol generates enormous value
  - UNI the token captures none of it
  - UNI's value is purely governance + speculative premium
```

### Why the Fee Switch Hasn't Been Activated

```
Arguments for activation:
  + UNI holders deserve protocol revenue
  + Creates fundamental value (DCF-based)
  + Aligns incentives between protocol and token holders
  + $500M+ annually at current volumes

Arguments against:
  - Regulatory risk (fee sharing → UNI looks like a security)
  - LP disincentive (LPs earn less → might migrate to competing DEXes)
  - Frontend fee already captures value for the company
  - Governance gridlock (can't reach consensus)
```

### Lessons from UNI

1. **Governance-only tokens are weak**: Without economic rights, token value depends entirely on speculation and treasury control
2. **The protocol-token gap**: A successful protocol does not guarantee a valuable token
3. **Regulatory ambiguity creates paralysis**: Fear of securities classification prevents value-accruing changes
4. **First-mover advantage expires**: Other DEXes (Aerodrome, Jupiter) have better tokenomics and are gaining market share

---

## 9. Jupiter (JUP) — Community-Governed Supply

### Tokenomics Profile

| Dimension | Design | Score |
|---|---|---|
| **Supply** | 10B initial, reduced to 7B via community burn vote | 4/5 |
| **Distribution** | 50% team, 50% community (multi-round airdrops) | 4/5 |
| **Utility** | Governance, staking, ecosystem access | 4/5 |
| **Value Accrual** | ASM (Active Supply Management), staking rewards | 4/5 |
| **Governance** | Active DAO with real power over supply | 5/5 |
| **Incentive Alignment** | Community controls supply → aligned incentives | 5/5 |
| **Total** | | **26/30** |

### Active Supply Management (ASM)

Jupiter's most innovative tokenomics feature: the community directly votes on token supply.

```
January 2025 vote: "Should we burn 3B JUP?"
  For: 95%
  Against: 5%
  Result: 3B JUP burned (30% of total supply)

  Impact:
    Supply: 10B → 7B
    Each remaining JUP represents a larger share
    FDV decreased by ~30%
    Per-token value increased proportionally

  This was the largest community-directed token burn in DeFi history
```

### The JUP Distribution Strategy

```
Round 1 (January 2024):
  ~1B JUP airdropped to 955,000 Solana wallets
  Criteria: Historical Jupiter trading volume and frequency
  Tiered distribution (not flat) — heavy users got more

Round 2 (planned):
  Additional community allocation
  Criteria likely based on ongoing JUP staking and governance participation

Ongoing:
  JUP staking rewards (governance + loyalty)
  Ecosystem grants via governance
  Strategic investments by DAO
```

### Why JUP Works

1. **Product-first**: Jupiter was already the dominant aggregator before the token
2. **Community control**: The burn vote proved governance has teeth
3. **Transparent team allocation**: 50% to team but with clear vesting
4. **Multiple utility layers**: Governance + staking + ecosystem access
5. **Solana-native narrative**: Largest airdrop in Solana history created loyalty

---

## 10. Jito (JTO) — MEV-Aligned Staking

### Tokenomics Profile

| Dimension | Design | Score |
|---|---|---|
| **Supply** | 1B max | 4/5 |
| **Distribution** | 34.3% community, 24.5% team, 16.2% investors, 10% airdrop | 4/5 |
| **Utility** | MEV governance, staking, tip distribution control | 4/5 |
| **Value Accrual** | JTO staking, MEV-enhanced yields via jitoSOL | 4/5 |
| **Governance** | Control over MEV strategy and tip distribution | 4/5 |
| **Incentive Alignment** | Token aligned with MEV extraction efficiency | 4/5 |
| **Total** | | **24/30** |

### JTO's Unique Position

JTO governs the MEV infrastructure layer of Solana:

```
Jito's role in Solana:
  1. Jito validator client: Modified Solana validator for MEV extraction
  2. Tip router: Distributes MEV tips to validators/stakers
  3. jitoSOL: Liquid staking token with MEV yield enhancement

  JTO governance controls:
  - How MEV tips are distributed
  - Validator delegation strategy
  - Protocol parameter changes
  - Treasury allocation for MEV research
```

### MEV as a Revenue Source

```
Solana MEV flow:
  Searchers find MEV opportunities → Submit bundles with tips
  Jito block engine processes bundles → Validators include profitable bundles
  Tips distributed: validators + jitoSOL stakers

  JTO captures value by governing this infrastructure:
  - More Solana activity → More MEV → More tips → More jitoSOL yield
  - More jitoSOL yield → More staking → More TVL → More JTO demand
```

---

## 11. Chainlink (LINK) — Work Token Model

### Tokenomics Profile

| Dimension | Design | Score |
|---|---|---|
| **Supply** | 1B fixed | 4/5 |
| **Distribution** | 35% public, 35% node operator incentives, 30% company | 3/5 |
| **Utility** | Oracle payments, node staking (work token) | 5/5 |
| **Value Accrual** | Required for oracle services, staking collateral | 4/5 |
| **Governance** | Limited (Chainlink Labs controls most decisions) | 2/5 |
| **Incentive Alignment** | Node operators stake LINK → honest data → earn fees | 4/5 |
| **Total** | | **22/30** |

### The LINK Work Token Model

```
LINK economics:
  1. Data consumers pay LINK for oracle data feeds
  2. Node operators stake LINK as collateral
  3. If operators provide bad data → slashed
  4. If operators provide good data → earn LINK fees

  Value equation:
    LINK demand = oracle_requests × price_per_request + staking_requirement

    As oracle usage grows → more LINK demanded for payments and staking
    Supply is fixed → price must increase to meet demand
```

### Staking v0.2

```
LINK staking launched in phases:
  Phase 1: Simple staking, limited pool (~25M LINK)
  Phase 2: Expanded staking, slashing live
  Future: Full staking with dynamic rewards based on oracle usage

  Staking yield: ~4-5% APY
  Source: LINK emissions + partial oracle fee revenue
  Slashing risk: Real (bad oracle data can be slashed)
```

---

## 12. Cautionary Tales

### 12.1 FTT — The Collateral House of Cards

```
FTT tokenomics (appeared strong on paper):
  - Buyback and burn (using FTX profits)
  - Exchange fee discounts
  - Collateral for FTX margin trading
  - Limited supply with regular burns

  Score at face value: 22/30 (seemed decent)

What actually happened:
  - Alameda Research (FTX's sister company) used FTT as collateral for billions in loans
  - FTT's "value" was circular: FTX → FTT burns → FTT price → Alameda collateral → loans back to FTX
  - When confidence collapsed (November 2022):
    FTT price dropped 90% → Alameda's collateral worthless → $8B hole → FTX bankrupt

  Lesson: Token buyback and burn is meaningless if the token's value is
  circular — propped up by the same entity that benefits from the burn.
  Real value accrual must come from external economic activity.
```

### 12.2 LUNA/UST — The Death Spiral

```
LUNA tokenomics:
  - LUNA burned to mint UST (1 UST = $1 of LUNA burned)
  - UST demand grows → more LUNA burned → LUNA supply shrinks → price up
  - Anchor protocol offered 20% APY on UST deposits (subsidized by reserves)

  The positive spiral (2021-2022):
    UST demand ↑ → LUNA burned ↑ → LUNA price ↑ → More UST confidence → UST demand ↑

  The death spiral (May 2022):
    UST depegs slightly → Users redeem UST for LUNA → LUNA minted → LUNA price ↓
    → LUNA price ↓ → More LUNA needed to back UST → More LUNA minted
    → More LUNA selling → LUNA price ↓↓ → Hyperinflation

  LUNA supply:
    Before collapse: ~350M LUNA
    After collapse: ~6.5 TRILLION LUNA
    Price: $80 → $0.00001

  $40B in value destroyed in 72 hours.

  Lesson: Reflexive tokenomics (where the token backs itself) creates
  unstoppable death spirals. Collateral must be EXTERNAL, not self-referential.
```

### 12.3 OHM — Unsustainable APY

```
OHM tokenomics:
  - "Protocol-owned liquidity" via bonding (users sell LP tokens for discounted OHM)
  - Massive staking APY: started at >10,000% APY
  - Stakers receive rebasing rewards (auto-compounding)
  - "Game theory" narrative: (3,3) = everyone stakes = everyone wins

  What actually happened:
    Peak OHM price: ~$1,400 (April 2022)
    1 year later: ~$15
    Decline: ~99%

  The APY illusion:
    10,000% APY sounds amazing → but it's paid in OHM
    If OHM price drops 99% → your 10,000% APY yield is worth 1% in dollar terms
    APY was just token printing, not real yield

  The bonding paradox:
    Bonding gave protocol-owned liquidity (good concept)
    But the discount was funded by diluting existing holders
    Net effect: existing holders subsidized new protocol-owned liquidity

  Lesson: High APY from emissions is NOT yield — it's dilution.
  Real yield = protocol revenue / staked value.
  If the APY is 3 digits or higher, it's almost certainly unsustainable.
```

---

## 13. Comparative Analysis

### Value Accrual Comparison

| Token | Revenue Source | Value Accrual Mechanism | Sustainability |
|---|---|---|---|
| BTC | Transaction fees | Scarcity (fixed supply) | High (simple) |
| ETH | Transaction fees | Fee burn + staking yield | Very high (adaptive) |
| SOL | Transaction fees | Fee burn + staking | High (with ecosystem growth) |
| CRV | Trading fees | 50% to veCRV + gauge voting | High (with lock incentives) |
| MKR | Stability fees | Buyback + burn | Very high (revenue-linked) |
| AAVE | Interest spread | Safety Module + buyback | High (revenue-linked) |
| UNI | None to holders | Governance only | Low (no value accrual) |
| JUP | Trading fees | Staking + burns + governance | High (active management) |
| JTO | MEV tips | Staking + MEV governance | High (infrastructure) |
| LINK | Oracle fees | Work staking | Medium (slow adoption) |

### Supply Mechanics Comparison

```
Net Deflationary:
  ████████████████████████  ETH (during high usage)
  ████████████████████      MKR (when profitable)
  ████████████████          BTC (lost coins, approaching zero inflation)

Low Inflation:
  ████████████              SOL (~5.2%, decreasing)
  ████████████              AAVE (~minimal emissions)

Moderate Inflation:
  ████████                  CRV (high emission, offset by locks)
  ████████                  JUP (reduced by community burns)

Fixed Supply (no change):
  ██████████████████████    UNI (1B, done)
  ██████████████████████    LINK (1B, done)
```

### Distribution Quality Comparison

```
Most Fair:
  BTC (100% mined, no pre-mine)
  YFI (100% community, zero team)

Community-Heavy:
  JUP (50% community)
  CRV (62% community LPs)

Balanced:
  AAVE (mix of swap + community + team)
  JTO (34.3% community + 10% airdrop)

Insider-Heavy:
  SOL (38% investors, 12.8% team)
  UNI (40% team + investors)
```

---

## 14. References

1. **Token Terminal**: Revenue and earnings data for all protocols analyzed
2. **DeFiLlama**: TVL, token holder, and staking data
3. **Dune Analytics**: On-chain metrics for token distributions
4. **CoinGecko**: Historical price and supply data
5. **Each protocol's documentation**: Official docs for tokenomics details
6. **"Reflexivity in Crypto" — various**: Analysis of LUNA/UST death spiral
7. **FTX post-mortem reports**: SEC filings and bankruptcy documents
8. **"The State of DeFi Governance" — Messari**: Governance participation data

---

*Next: [09 - Tokenomics Design Framework](./09-tokenomics-design-framework.md) — A step-by-step guide to designing tokenomics from scratch, including evaluation checklists, simulation tools, game theory considerations, and common pitfalls.*
