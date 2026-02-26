# Tokenomics Design Framework

> Written for experienced Solana developers entering token design.
> Last updated: February 2026

---

## Table of Contents

1. [The Design Process](#1-the-design-process)
2. [Step 1: Define the Token's Purpose](#2-step-1-define-the-tokens-purpose)
3. [Step 2: Design Supply Mechanics](#3-step-2-design-supply-mechanics)
4. [Step 3: Plan Distribution](#4-step-3-plan-distribution)
5. [Step 4: Build the Demand Stack](#5-step-4-build-the-demand-stack)
6. [Step 5: Design Governance](#6-step-5-design-governance)
7. [Step 6: Model and Simulate](#7-step-6-model-and-simulate)
8. [Step 7: Plan the Launch](#8-step-7-plan-the-launch)
9. [Evaluation Checklists](#9-evaluation-checklists)
10. [Common Mistakes and How to Avoid Them](#10-common-mistakes-and-how-to-avoid-them)
11. [Game Theory Considerations](#11-game-theory-considerations)
12. [Regulatory Considerations](#12-regulatory-considerations)
13. [Tokenomics Simulation Template](#13-tokenomics-simulation-template)
14. [References](#14-references)

---

## 1. The Design Process

### Overview

Designing tokenomics is engineering a game — you define the rules, the players follow rational self-interest, and the system must produce desirable outcomes under adversarial conditions.

```
The Tokenomics Design Pipeline:

  1. Purpose    → Why does this token exist?
  2. Supply     → How many tokens, how does supply change?
  3. Distribution → Who gets tokens, when, how?
  4. Demand     → Why would anyone buy/hold?
  5. Governance → How do token holders influence the protocol?
  6. Simulation → Does the model work under stress?
  7. Launch     → How do you execute the distribution?
  8. Iterate    → Monitor and adjust post-launch
```

### Principles

| Principle | Explanation |
|---|---|
| **Simplicity** | If you can't explain it in 2 minutes, it's too complex |
| **Alignment** | Insiders and community must benefit from the same outcomes |
| **Sustainability** | The system must work without perpetual token printing |
| **Anti-fragility** | The system should get stronger under stress, not weaker |
| **Transparency** | All supply, distribution, and governance data must be public |
| **Irreversibility** | Credible commitments (revoked mint authority, on-chain vesting) build trust |

---

## 2. Step 1: Define the Token's Purpose

### The Purpose Decision Tree

```
Does your protocol need decentralized consensus?
  YES → Network token (staking, slashing, validation)
  NO  ↓

Does your protocol need to bootstrap liquidity/usage?
  YES → Consider governance + incentive token
  NO  ↓

Does your protocol generate revenue for stakeholders?
  YES → Revenue-sharing or buyback token
  NO  ↓

Does your protocol need decentralized governance?
  YES → Governance token
  NO  → You probably don't need a token.
```

### Purpose Templates

**Type A: Network Security Token** (ETH, SOL)
```
Purpose: Secure a decentralized network
Supply: Inflationary (pays validators)
Utility: Gas fees, staking, slashing
Demand: Required for every transaction
Example: SOL is needed for every Solana transaction + staking
```

**Type B: Protocol Governance + Revenue Token** (CRV, MKR)
```
Purpose: Govern protocol parameters + capture protocol value
Supply: Fixed or deflationary (buyback + burn)
Utility: Voting, revenue sharing, boosting
Demand: Governance power + yield + scarcity
Example: MKR governs MakerDAO and benefits from surplus revenue
```

**Type C: Liquidity Incentive Token** (SUSHI, early UNI)
```
Purpose: Bootstrap initial liquidity and user adoption
Supply: High initial emission, decreasing over time
Utility: Primarily governance, potential fee switch
Demand: Emission farming (short-term) + governance (long-term)
Example: SUSHI emissions attracted LPs, governance controls fee sharing
```

**Type D: Work/Service Token** (LINK, GRT)
```
Purpose: Coordinate and pay decentralized service providers
Supply: Fixed or low inflation
Utility: Required to operate as a service provider + payment for services
Demand: Service demand drives token demand directly
Example: LINK is required for oracle operations and payment
```

---

## 3. Step 2: Design Supply Mechanics

### Supply Decision Matrix

| Factor | Fixed Supply | Disinflationary | Inflationary |
|---|---|---|---|
| **Scarcity narrative** | Strong | Moderate | Weak |
| **Security budget** | Fee-dependent | Transitioning | Sustainable |
| **Flexibility** | None | Some | High |
| **DeFi composability** | Excellent | Good | Good |
| **Best for** | Store of value, governance | Network tokens | Service networks |

### Supply Design Checklist

```
□ What is the maximum supply? (Fixed number, or infinite with target inflation?)
□ What is the initial circulating supply at TGE?
□ What is the FDV to market cap ratio? (Lower is better — less future dilution)
□ What is the emission schedule? (Linear, halving, decay?)
□ Is there a floor inflation rate? (For perpetual security budget)
□ Are there burn mechanisms? (Fee burns, buyback burns, transfer tax?)
□ What is the net annual inflation/deflation? (Emission - burns = net)
□ Can mint authority be revoked? (Fixed supply) Or is it program-controlled?
□ Who controls supply changes? (No one, program, governance, multisig?)
```

### Recommended Supply Parameters

| Protocol Type | Max Supply | Initial Circ | Year 1 Inflation | Target Inflation |
|---|---|---|---|---|
| Layer 1 | None (disinflationary) | 30-60% | 5-10% | 1-3% |
| DeFi protocol | Fixed (1B-10B) | 5-20% | 10-30% (emissions) | 0% |
| Governance token | Fixed | 10-25% | 5-15% | 0% |
| Work token | Fixed | 20-40% | 5-10% (node incentives) | 0-2% |

---

## 4. Step 3: Plan Distribution

### Distribution Design Checklist

```
□ What percentage goes to the community? (Target: >50%)
□ What percentage goes to team/founders? (Target: 15-25%)
□ What percentage goes to investors? (Target: 10-20%)
□ What is the vesting schedule for insiders? (Target: 12-month cliff, 36-month vest)
□ Is there a TGE unlock for insiders? (Target: 0-10%)
□ How is the community allocation distributed?
   □ Airdrop (retroactive usage)
   □ Liquidity mining (ongoing participation)
   □ Grants (ecosystem development)
   □ Future incentives (reserved for governance)
□ Is the distribution Sybil-resistant?
□ Are there multiple distribution rounds? (Spreads risk, rewards ongoing users)
□ Is there a treasury for ongoing development?
```

### Distribution Red Flags

| Red Flag | Why It's Bad | Threshold |
|---|---|---|
| Team > 30% | Excessive insider allocation | Max 25% with long vest |
| No vesting | Insiders dump at TGE | Always vest insiders |
| TGE unlock > 20% for insiders | Immediate sell pressure | Max 10% for insiders |
| Community < 40% | Not enough for bootstrapping | Min 50% community |
| No treasury | No development funding | 10-20% treasury |
| Single airdrop round | One-time event, no retention | Multiple rounds |

### Optimal Distribution Template

```
Community (55%):
  ├── Airdrop Round 1:      10% (retroactive users)
  ├── Airdrop Round 2-3:    10% (ongoing users over 12-24 months)
  ├── Liquidity mining:     15% (decreasing over 3 years)
  ├── Ecosystem grants:     10% (DAO-managed)
  └── Future reserves:      10% (locked, community-governed)

Team & Advisors (20%):
  ├── Core team:            15% (12-month cliff, 36-month vest)
  └── Advisors:              5% (6-month cliff, 24-month vest)

Investors (15%):
  ├── Seed:                  7% (12-month cliff, 24-month vest)
  └── Series A:              8% (6-month cliff, 18-month vest)

Treasury (10%):
  ├── Development:           5% (governance-controlled)
  └── Strategic reserve:     5% (emergency fund)
```

---

## 5. Step 4: Build the Demand Stack

### Demand Design Framework

Design demand in layers, from most reliable to least:

**Layer 1: Functional Requirement** (strongest)
```
The token MUST be used for core protocol operations
  - Gas fees (ETH, SOL)
  - Oracle payments (LINK)
  - Staking collateral (work tokens)

  If you can create this layer, the token has a guaranteed demand floor.
```

**Layer 2: Revenue Sharing** (strong)
```
Holding/staking the token entitles you to protocol revenue
  - Fee sharing (GMX: 30% of trading fees)
  - Buyback + burn (MKR: surplus buys and burns MKR)
  - Boosted yields (CRV: up to 2.5x for veCRV)

  Creates a DCF-based valuation floor.
```

**Layer 3: Governance Power** (moderate)
```
The token controls meaningful protocol decisions
  - Treasury allocation
  - Fee parameters
  - Asset listings
  - Emission direction (gauge voting)

  Valuable when the governance controls real resources.
```

**Layer 4: Lock/Staking Incentives** (moderate)
```
Locking tokens reduces supply and provides benefits
  - ve-locks (CRV: up to 4 years)
  - Safety Module (AAVE: earn yield, risk slashing)
  - Staking for access (tiered benefits)

  Effective at reducing sell pressure and circulating supply.
```

**Layer 5: Speculative/Narrative** (weakest)
```
The token benefits from a compelling narrative
  - "Ultrasound money" (ETH)
  - "Digital gold" (BTC)
  - Community momentum (meme tokens)

  Important for bootstrapping but not sustainable alone.
```

### Demand Stack Quality Assessment

```
Strong demand stack (4+ layers):
  ETH:  Gas + Staking + Collateral + Fee burn + L2 settlement + Narrative
  CRV:  Gauge voting + Fee share + Boost + Lock incentives
  MKR:  Governance + Buyback/burn + Collateral

Medium demand stack (2-3 layers):
  SOL:  Gas + Staking + Fee burn
  AAVE: Governance + Safety Module + Revenue
  JUP:  Governance + Supply management + Staking

Weak demand stack (1 layer):
  UNI:  Governance only (fee switch off)
  Most governance-only tokens
```

---

## 6. Step 5: Design Governance

### Governance Design Checklist

```
□ What decisions does governance control?
□ What decisions are immutable (not governable)?
□ What is the voting mechanism? (Token-weighted, quadratic, ve-weighted?)
□ What are the proposal requirements? (Minimum tokens to propose)
□ What is the quorum? (Minimum participation for valid vote)
□ What is the voting period? (Too short = manipulable, too long = slow)
□ Is there a timelock? (Delay between vote and execution)
□ Is there an emergency mechanism? (Guardian, pause, multisig veto)
□ Is delegation supported?
□ How does governance transition from centralized to decentralized?
```

### Governance Parameters Template

```
Phase 1 (Year 1) — Guided Governance:
  Proposal threshold:  1% of supply
  Quorum:             4% of supply
  Voting period:      3 days
  Timelock:           48 hours
  Guardian:           Team multisig can veto (emergency only)
  Scope:              Parameters + treasury spending

Phase 2 (Year 2-3) — Community Governance:
  Proposal threshold:  0.5% of supply
  Quorum:             3% of supply
  Voting period:      5 days
  Timelock:           72 hours
  Guardian:           Community multisig (elected)
  Scope:              + Protocol upgrades

Phase 3 (Year 3+) — Full Sovereignty:
  Proposal threshold:  0.25% of supply
  Quorum:             2% of supply
  Voting period:      7 days
  Timelock:           72 hours
  Guardian:           Removed (or elected council)
  Scope:              Everything including tokenomics changes
```

---

## 7. Step 6: Model and Simulate

### Key Metrics to Model

```
For each year (1-5), project:

  Supply side:
    - Total supply
    - Circulating supply
    - Vesting unlocks
    - Emissions (LP mining, staking)
    - Burns
    - Net supply change

  Demand side:
    - Protocol revenue
    - Revenue to token holders
    - Staking ratio
    - TVL
    - Active users

  Derived metrics:
    - FDV
    - Market cap
    - Token price (supply/demand equilibrium)
    - Real yield (revenue / staked value)
    - Emission yield (emissions × price / staked value)
    - Total APY (real + emission)
```

### Scenario Analysis

```
Scenario 1: Bull market
  - TVL grows 3x
  - Protocol revenue grows 5x
  - Token demand high
  - Result: Burns > emissions, token appreciates

Scenario 2: Stable growth
  - TVL grows 50%
  - Protocol revenue grows 2x
  - Moderate demand
  - Result: Near-equilibrium, modest appreciation

Scenario 3: Bear market
  - TVL drops 50%
  - Protocol revenue drops 60%
  - Low demand
  - Result: Emissions > burns, token under pressure
  - KEY QUESTION: Does the protocol survive? Are incentives still aligned?

Scenario 4: Black swan
  - Smart contract exploit
  - Regulatory action
  - Key team member departure
  - How does the tokenomics handle crisis?
  - Is there an emergency mechanism?
```

### The Death Spiral Test

```
Test your tokenomics for death spiral risk:

  1. Assume token price drops 80%
  2. Are emission incentives still sufficient? (In dollar terms, APY drops 80%)
  3. Do LPs/stakers leave? (Mercenary capital exits)
  4. Does TVL drop? (Less usage = less revenue)
  5. Does less revenue = less burns/buybacks? (Supply grows faster)
  6. Does growing supply = more sell pressure? (Price drops further)

  If each step leads to the next → DEATH SPIRAL RISK

  Mitigation:
  - Revenue-based rewards (not emission-based)
  - ve-locks (locked tokens can't leave during downturns)
  - Protocol-owned liquidity (permanent, not rented)
  - Emergency halt on emissions (governance can pause)
```

---

## 8. Step 7: Plan the Launch

### Launch Timeline

```
T-6 months: Finalize tokenomics design
T-5 months: Smart contract development + audit
T-4 months: Testnet deployment + community testing
T-3 months: Announce token + tokenomics paper
T-2 months: Points program or retroactive criteria published
T-1 month:  Final audit + security review
T-0:        TGE (Token Generation Event)
T+1 week:   Initial liquidity provision
T+1 month:  First governance proposal
T+3 months: First vesting cliff
T+12 months: Major vesting unlock + assessment
```

### Launch Checklist

```
Pre-launch:
  □ Smart contracts audited by 2+ independent firms
  □ Tokenomics paper published and reviewed
  □ Vesting contracts deployed and verified
  □ Initial liquidity sourced (DEX pools, market makers)
  □ Distribution mechanism tested (airdrop, claim portal)
  □ Anti-sybil measures implemented and tested
  □ Communication plan ready (blog, Twitter, Discord)
  □ Legal review completed (securities analysis)

At launch:
  □ Mint authority set correctly (PDA or revoked)
  □ Freeze authority set correctly (revoked for DeFi tokens)
  □ DEX pool seeded with initial liquidity
  □ Claim portal live and tested
  □ Token metadata registered (name, symbol, logo)
  □ Explorer/tracker verified (CoinGecko, CoinMarketCap)
  □ Monitoring dashboards live

Post-launch:
  □ Monitor claim rates and distribution
  □ Track selling pressure vs. staking
  □ Watch for smart contract issues
  □ Engage community governance
  □ Publish first transparency report
```

---

## 9. Evaluation Checklists

### For Evaluating Existing Tokens

**Quick Red Flags** (any one is concerning):
```
□ Team holds >40% with <1 year vesting
□ No audits or security reviews
□ FDV > 10x market cap (massive future dilution)
□ >90% of yield comes from emissions (unsustainable)
□ Mint authority is an EOA (centralized, can rug)
□ No clear utility beyond speculation
□ Anonymous team with no track record
□ Token launched before product
```

**Quick Green Flags** (multiple together is strong):
```
□ Product has real users before token launch
□ Revenue exceeds emissions in dollar terms
□ Community holds >50% of supply
□ Team has multi-year vesting (12-month cliff minimum)
□ Mint authority revoked or program-controlled
□ Multiple demand drivers (governance + revenue + staking)
□ Active governance with >10% participation
□ Multiple independent audits
```

### The 10-Question Token Evaluation

```
1. Does the token need to exist?                         (Y/N)
2. Is there organic demand beyond speculation?            (Y/N)
3. Does the token capture protocol revenue?               (Y/N)
4. Is the supply model sustainable?                       (Y/N)
5. Is the distribution fair?                              (Y/N)
6. Are insiders properly vested?                          (Y/N)
7. Is there active governance?                            (Y/N)
8. Does the token survive a bear market?                  (Y/N)
9. Are the smart contracts audited and secure?            (Y/N)
10. Is the team transparent and credible?                 (Y/N)

Score:
  8-10 YES: Strong tokenomics
  5-7 YES:  Average, some concerns
  <5 YES:   Weak, significant risks
```

---

## 10. Common Mistakes and How to Avoid Them

### Mistake 1: Launching a Token Too Early

```
Problem: Token launches before the product has market fit
Result:  Token price driven by speculation, not usage
         When speculation fades, there's no fundamental demand

Fix: Build product first, launch token when there are real users
     Use points or off-chain incentives during the product-building phase
     Token should amplify existing success, not substitute for it
```

### Mistake 2: Over-Engineering

```
Problem: Complex tokenomics with 5 token types, nested staking,
         cross-protocol composability, and algorithmic rebasing
Result:  Nobody understands it, bugs in complex systems, hard to audit

Fix: Start simple. One token, clear utility, straightforward staking.
     Add complexity only when the base model is proven.
     If you can't explain it in a 2-minute elevator pitch, simplify.
```

### Mistake 3: Ignoring Sell Pressure

```
Problem: Focus on minting (emissions, rewards) without modeling selling
Result:  Perpetual sell pressure from:
         - Emission farmers dumping tokens
         - Team/VC vesting unlocks
         - Staking rewards being sold

Fix: For every token minted, ask: "Who will sell this, and when?"
     Model the sell side explicitly.
     Ensure buy pressure > sell pressure at equilibrium.
```

### Mistake 4: Conflating Revenue and Emissions

```
Problem: Advertising "50% APY!" when 95% comes from printing tokens
Result:  Attracts mercenary capital, token price declines, APY becomes 5%,
         capital leaves

Fix: Always separate real yield from emission yield in all communications.
     Target: Revenue should fund >50% of incentives by year 3.
```

### Mistake 5: No Exit Strategy for Emissions

```
Problem: "We'll figure out how to reduce emissions later"
Result:  Governance can never agree to cut their own rewards
         Emissions continue indefinitely, diluting everyone

Fix: Encode emission reduction in the smart contract.
     Make it automatic (decay function), not governance-dependent.
     Set a hard sunset date for emissions.
```

### Mistake 6: Circular Value

```
Problem: Token's value depends on itself
         "Token is valuable because it's staked, staking is attractive
          because the token is valuable"
Result:  Works in bull markets (positive reflexivity)
         Collapses in bear markets (negative reflexivity)
         See: LUNA/UST, OHM

Fix: Token value must be grounded in EXTERNAL economic activity.
     Protocol revenue from real users doing real things.
     Not from other token holders in a circular system.
```

### Mistake 7: Ignoring Tax Implications

```
Problem: Rebasing tokens, airdrop farming, and complex staking
         create tax nightmares for users
Result:  Institutional and sophisticated users avoid the token
         Retail users face unexpected tax bills

Fix: Favor share-based models over rebasing.
     Document tax implications clearly.
     Design with tax efficiency in mind (fewer taxable events).
```

---

## 11. Game Theory Considerations

### Nash Equilibria in Token Systems

A Nash equilibrium is a state where no participant can benefit by unilaterally changing their strategy. Good tokenomics creates Nash equilibria at desirable states:

```
Example: ve-CRV locking

  If everyone locks CRV:
    - High governance participation
    - Low circulating supply
    - Strong price support
    - Boosted yields for all

  If you're the only one who doesn't lock:
    - You miss governance, fee sharing, and boost
    - Your LP farming is 2.5x less than lockers
    - You're worse off than if you locked

  Nash equilibrium: Everyone locks → this is the stable state
  The system is designed so that locking is ALWAYS the dominant strategy
```

### Schelling Points

A Schelling point is a natural focal point that people converge on without communication. In tokenomics:

```
Staking ratio Schelling points:
  Too low (<30%):  "I should stake because rewards are high"
  Too high (>80%): "I should unstake and use DeFi for better yield"
  Natural equilibrium: ~50-65% staked

Token price Schelling points:
  FDV < revenue × 10: "Undervalued, should buy"
  FDV > revenue × 100: "Overvalued, should sell"
  Natural equilibrium: FDV = revenue × 20-50 (varies by growth)
```

### Mechanism Design Principles Applied

**Incentive compatibility**: Design so that honest behavior is profitable.

```
Bad: Oracle reporter can profit from lying → will lie
Good: Oracle reporter is slashed more than they gain from lying → won't lie

Formula: Expected profit from honesty > Expected profit from cheating
         Reward_honest + Stake × (1 - slash_probability) > Reward_cheat - Stake × slash_probability
```

**Budget balance**: The protocol should be self-sustaining.

```
Revenue from users ≥ Rewards to providers + Operating costs

If this inequality holds without external funding (emissions),
the protocol is economically self-sustaining.
```

---

## 12. Regulatory Considerations

### The Securities Question

The fundamental regulatory question: **Is your token a security?**

In the US, the Howey Test determines this:

```
An instrument is a security if it involves:
  1. An investment of money          ← Buying the token
  2. In a common enterprise          ← The protocol
  3. With expectation of profits     ← Token appreciation, staking yield
  4. Derived from the efforts of others ← Team develops the protocol

If ALL FOUR prongs are met → likely a security
```

### Strategies to Reduce Securities Risk

| Strategy | Mechanism | Example |
|---|---|---|
| **Sufficient decentralization** | No single entity controls the protocol | Bitcoin, Ethereum |
| **Utility-first design** | Token required for protocol function | LINK (oracle payments) |
| **No promise of profits** | Never market the token as an investment | Legal disclaimers |
| **Community distribution** | Wide distribution, low insider % | YFI fair launch |
| **Governance token framing** | Token grants voting rights, not dividends | UNI (no fee switch) |

### MiCA (EU) Considerations

The EU's Markets in Crypto-Assets regulation (effective 2024-2025) creates clearer categories:

```
MiCA token categories:
  1. E-money tokens (stablecoins pegged to one fiat currency)
  2. Asset-referenced tokens (stablecoins pegged to multiple assets)
  3. Utility tokens (provide access to a service)

  DeFi protocols with sufficient decentralization may be exempt
  But the "sufficient decentralization" test is unclear
```

### Legal Design Tips

```
DO:
  ✓ Get legal advice before launching a token
  ✓ Structure the token as a utility or governance token
  ✓ Ensure the protocol can function without the team
  ✓ Distribute tokens widely (not just to insiders)
  ✓ Use a foundation or DAO entity for governance

DON'T:
  ✗ Promise returns or profits to token holders
  ✗ Market the token as an "investment opportunity"
  ✗ Maintain centralized control after claiming decentralization
  ✗ Ignore jurisdictional differences (US, EU, Asia all differ)
  ✗ Skip the legal review to save money (the cost of enforcement is far higher)
```

---

## 13. Tokenomics Simulation Template

### Spreadsheet Model

Build a year-by-year model with these inputs and outputs:

```
INPUTS:
  Token parameters:
    - Max supply: 1,000,000,000
    - Initial circulating: 100,000,000 (10%)
    - Annual emission (year 1-3): 200M, 150M, 100M
    - Annual burn rate: f(protocol_revenue)

  Protocol metrics:
    - Year 1 revenue: $10M
    - Year 1 TVL: $200M
    - Growth rate: 50% year-over-year
    - Revenue to token holders: 30%

  Market assumptions:
    - Token price: market cap / circulating supply
    - Market sentiment multiplier: 1.0 (neutral)

OUTPUTS (per year):
  Supply:
    Total supply: previous + emissions - burns
    Circulating: previous + vesting_unlocks + emissions - burns - locked
    FDV: total_supply × price
    Market cap: circulating × price

  Economics:
    Protocol revenue: TVL × fee_rate × utilization
    Revenue to holders: revenue × share_percentage
    Emission value: emission_tokens × price
    Real yield: holder_revenue / (staked_tokens × price)
    Emission yield: emission_value / (staked_tokens × price)
    Total APY: real_yield + emission_yield

  Sustainability:
    Revenue / emissions ratio: (>1 = sustainable)
    Months of treasury runway
    Staking ratio
    Price needed for break-even
```

### Python Simulation

```python
import numpy as np

class TokenomicsSimulator:
    def __init__(self, config):
        self.max_supply = config["max_supply"]
        self.initial_circulating = config["initial_circulating"]
        self.emission_schedule = config["emission_schedule"]  # per year
        self.burn_rate = config["burn_rate"]  # % of revenue burned
        self.revenue_share = config["revenue_share"]  # % to token holders
        self.base_revenue = config["base_revenue"]
        self.growth_rate = config["growth_rate"]
        self.staking_ratio = config["staking_ratio"]

    def simulate(self, years=5):
        results = []
        circulating = self.initial_circulating
        total_supply = self.max_supply  # All minted, vesting over time

        for year in range(1, years + 1):
            # Revenue grows
            revenue = self.base_revenue * (1 + self.growth_rate) ** (year - 1)
            holder_revenue = revenue * self.revenue_share

            # Emissions decrease
            emission = self.emission_schedule[min(year - 1, len(self.emission_schedule) - 1)]

            # Burns from revenue
            burn_amount = revenue * self.burn_rate  # In dollar terms

            # Update circulating supply
            circulating += emission  # New tokens enter circulation

            # Staking
            staked = circulating * self.staking_ratio
            free_float = circulating - staked

            # Estimate price (very simplified)
            # Real models use supply/demand curves
            annual_demand = holder_revenue + burn_amount
            annual_sell_pressure = emission  # Assumption: all emitted tokens are sold

            results.append({
                "year": year,
                "circulating": circulating,
                "revenue": revenue,
                "holder_revenue": holder_revenue,
                "emission": emission,
                "staked": staked,
                "real_yield_pct": (holder_revenue / staked * 100) if staked > 0 else 0,
                "revenue_emission_ratio": revenue / (emission or 1),
            })

        return results
```

---

## 14. References

### Essential Reading for Token Designers

1. **"Token Economics" — Shermin Voshmgir**: Academic textbook on token design
2. **"Tokenomics" — Sean Au & Thomas Power**: Practical guide to token creation
3. **"Mechanism Design" — Nisan et al. (Chapter from "Algorithmic Game Theory")**: Mathematical foundations
4. **"The Economics of Token-Mediated Platforms" — Cong, Li, Wang (2021)**: Academic framework
5. **"Placeholder VC Token Taxonomy"**: Framework for classifying token types
6. **"Delphi Digital Tokenomics Reports"**: In-depth analysis of specific token designs

### Tools and Resources

| Tool | Purpose | URL |
|---|---|---|
| Token Terminal | Protocol revenue data | tokenterminal.com |
| DeFiLlama | TVL and protocol metrics | defillama.com |
| Token Unlocks | Vesting schedule data | tokenunlocks.app |
| Dune Analytics | On-chain analytics | dune.com |
| CoinGecko | Supply and market data | coingecko.com |
| Snapshot | Off-chain governance | snapshot.org |
| Realms | Solana governance | realms.today |
| Tally | Governance analytics | tally.xyz |

### Audit Firms for Token Contracts

| Firm | Specialization |
|---|---|
| Trail of Bits | Solidity, Rust, cryptography |
| Halborn | Solana, Ethereum, cross-chain |
| OtterSec | Solana-focused |
| Neodyme | Solana-focused |
| OpenZeppelin | Ethereum, Solidity |
| Certora | Formal verification |
| MadShield | Solana audits |

---

*This concludes the Tokenomics Research Compendium. For DeFi protocol fundamentals, see the companion [DeFi Research Compendium](../README.md).*
