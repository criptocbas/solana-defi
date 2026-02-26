# Tokenomics Fundamentals: A Comprehensive Technical Reference

> Written for experienced Solana developers entering token design.
> Last updated: February 2026

---

## Table of Contents

1. [What Is Tokenomics?](#1-what-is-tokenomics)
2. [History and Evolution of Token Design](#2-history-and-evolution-of-token-design)
3. [Token Taxonomy](#3-token-taxonomy)
   - [3.1 Utility Tokens](#31-utility-tokens)
   - [3.2 Governance Tokens](#32-governance-tokens)
   - [3.3 Security Tokens](#33-security-tokens)
   - [3.4 LP and Receipt Tokens](#34-lp-and-receipt-tokens)
   - [3.5 Stablecoins as Tokens](#35-stablecoins-as-tokens)
   - [3.6 Meme and Community Tokens](#36-meme-and-community-tokens)
   - [3.7 Work and Staking Tokens](#37-work-and-staking-tokens)
4. [Supply and Demand Framework](#4-supply-and-demand-framework)
5. [The Role of Tokens in Protocol Design](#5-the-role-of-tokens-in-protocol-design)
6. [Token Value Theories](#6-token-value-theories)
7. [The "Does This Need a Token?" Test](#7-the-does-this-need-a-token-test)
8. [Key Terminology Glossary](#8-key-terminology-glossary)
9. [References](#9-references)

---

## 1. What Is Tokenomics?

**Tokenomics** (token + economics) is the study and design of the economic systems surrounding blockchain tokens. It encompasses every decision about a token's creation, distribution, utility, governance, supply dynamics, and incentive structures. Where traditional economics studies how societies allocate scarce resources, tokenomics studies how digital tokens allocate value, rights, and incentives within decentralized systems.

### Why Tokenomics Matters

A protocol's tokenomics is arguably more important than its code. Perfectly written smart contracts with broken tokenomics will fail — users won't participate if the economic incentives are wrong. Conversely, protocols with mediocre code but brilliant tokenomics have attracted billions in TVL. The reason is simple: tokenomics determines the game-theoretic equilibrium that emerges when thousands of self-interested actors interact with a protocol.

### The Three Pillars of Tokenomics

| Pillar | Key Questions | Why It Matters |
|---|---|---|
| **Supply** | How many tokens exist? How does supply change over time? Is it capped, inflationary, deflationary? | Supply mechanics directly determine scarcity and inflation pressure |
| **Distribution** | Who gets tokens? When? How? What vesting applies? | Distribution determines who controls the protocol and incentive alignment |
| **Utility** | What can you do with the token? Why would someone buy/hold it? | Utility creates organic demand beyond speculation |

### Tokenomics vs. Token Engineering

These terms are often conflated but have distinct scopes:

- **Tokenomics**: The broader economic design — supply, demand, distribution, incentives, game theory
- **Token engineering**: The technical implementation — smart contract mechanics, emission algorithms, staking formulas
- **Mechanism design**: The mathematical foundation — how to design rules so rational agents produce desired outcomes

A complete token designer must understand all three. This research series covers all of them, with particular emphasis on the Solana program-level implementation that most resources neglect.

---

## 2. History and Evolution of Token Design

### Phase 0: Pre-Token Digital Scarcity (1990s-2008)

The concept of digital scarcity predates Bitcoin. David Chaum's DigiCash (1989), Adam Back's Hashcash (1997), Wei Dai's b-money (1998), and Nick Szabo's Bit Gold (1998) all explored how cryptography could create scarce digital assets. None solved the double-spend problem without a central authority.

**Key insight**: Digital scarcity is the foundation of all tokenomics. Without it, tokens have no economic properties because they can be copied infinitely.

### Phase 1: Bitcoin — The First Tokenomics (2009-2013)

Bitcoin's whitepaper (2008) implemented the first complete tokenomics system:

| Property | Bitcoin's Design | Significance |
|---|---|---|
| Total supply | 21,000,000 BTC | First credibly scarce digital asset |
| Emission | Block rewards, halving every ~4 years | Programmatic, predictable monetary policy |
| Distribution | Mining (proof of work) | Anyone can participate, no pre-mine |
| Utility | Peer-to-peer payments, store of value | Simple, focused use case |
| Governance | Off-chain (BIPs, social consensus) | No on-chain governance token |

Bitcoin's halving schedule is the most studied emission curve in tokenomics:

```
Block 0-209,999:        50 BTC per block
Block 210,000-419,999:  25 BTC per block
Block 420,000-629,999:  12.5 BTC per block
Block 630,000-839,999:  6.25 BTC per block
Block 840,000+:         3.125 BTC per block
...approaches 0 asymptotically
```

By 2140, all 21M BTC will have been mined. After that, miners are compensated solely through transaction fees. This creates a natural transition from inflation-funded security to fee-funded security.

**Design lesson**: Bitcoin proved that simple, predictable, and transparent supply mechanics create trust. Every subsequent token design builds on or departs from these principles.

### Phase 2: Ethereum and the Token Explosion (2014-2017)

Ethereum's ERC-20 standard (2015) democratized token creation. Before ERC-20, launching a new token required building an entire blockchain. After ERC-20, anyone could deploy a token contract with a single transaction. This was the Cambrian explosion of tokenomics.

**The ICO Era (2016-2018)**: Initial Coin Offerings raised over $20 billion. Most were pure fundraising mechanisms with no thought given to token utility. The typical ICO token had:
- Fixed supply (sometimes arbitrary, like 1 billion)
- Large team/advisor allocation (20-40%)
- No vesting (tokens immediately liquid)
- No real utility (just "will be used for payments on our platform")
- No burn, no staking, no governance

**What went wrong**: 90%+ of ICO tokens went to zero because they had no organic demand. Holding the token conferred no rights, generated no yield, and was not required for any action. The tokens existed solely for speculation.

**Design lesson**: A token without utility is a token without long-term value. Tokenomics must create genuine reasons to hold beyond speculation.

### Phase 3: DeFi Summer and Yield Farming (2018-2020)

Compound's COMP token launch in June 2020 catalyzed "DeFi Summer." COMP was distributed to users who supplied or borrowed assets — the first major instance of **liquidity mining**. Within weeks, competitors launched similar programs:

| Protocol | Token | Innovation |
|---|---|---|
| Compound | COMP | Governance token distributed to protocol users |
| Yearn | YFI | "Fair launch" — no pre-mine, no VC, fully community-distributed |
| SushiSwap | SUSHI | "Vampire attack" — migration incentives to fork Uniswap's liquidity |
| Curve | CRV | Vote-escrow model (veCRV) — lock tokens for governance + boosted rewards |
| Uniswap | UNI | Retroactive airdrop to all historical users |

**Key innovations**:
- **Liquidity mining**: Paying users in governance tokens to provide liquidity
- **Fair launch**: No pre-mine, no VC allocation (YFI)
- **Vote-escrow (ve)**: Locking tokens for time-weighted governance power (Curve)
- **Retroactive airdrop**: Rewarding historical users who contributed before the token existed (Uniswap)

**What went wrong**: Liquidity mining created **mercenary capital** — users who farmed tokens purely for profit and sold immediately, crashing token prices. The "yield" was just token emissions, not real economic value.

### Phase 4: Sustainable Tokenomics (2021-2023)

The industry learned from DeFi Summer's excesses:

- **Revenue sharing**: Protocols started distributing real revenue (not just emissions) to token holders. GMX pioneered this with 30% of trading fees going to GMX stakers.
- **Real yield**: The term "real yield" emerged to distinguish fee-based income from emission-based income.
- **Buyback and burn**: Protocols like Binance (BNB) and Maker (MKR) used revenue to buy and burn tokens, creating deflationary pressure.
- **ve-tokenomics matured**: Curve's vote-escrow model was adopted by dozens of protocols (Balancer, Frax, Pendle), proving that time-locked governance creates long-term alignment.

### Phase 5: Points, Airdrops, and Distribution Innovation (2024-2026)

The current era is defined by:

- **Points systems**: Protocols award off-chain "points" for usage, then convert to tokens at TGE. Examples: EigenLayer, Blast, Jupiter.
- **Airdrop farming**: Users optimize behavior to qualify for airdrops, creating a meta-game around token distribution.
- **Progressive decentralization**: Teams launch with more centralized control, then gradually transfer power to token holders.
- **Token buybacks as a service**: Real economic activity (trading fees, MEV, interest) funds programmatic buybacks.
- **Solana-native innovation**: Jupiter's JUP distribution (50% to community), Jito's JTO staking model, and the meme coin phenomenon (BONK, WIF) have created Solana-specific tokenomics patterns.

---

## 3. Token Taxonomy

### 3.1 Utility Tokens

A utility token grants the holder access to a specific function within a protocol. The token is **required** to use the service — it is not optional.

**Characteristics**:
- Token must be spent or staked to access functionality
- Demand is driven by protocol usage, not speculation
- Value correlates with protocol adoption

**Examples**:

| Token | Utility | Mechanism |
|---|---|---|
| LINK (Chainlink) | Pay oracle node operators for data feeds | Spent as payment for oracle services |
| FIL (Filecoin) | Pay for decentralized storage | Spent to store/retrieve files |
| HNT (Helium) | Reward hotspot operators, pay for network usage | Burn for data credits (DC) |
| RENDER (Render) | Pay for GPU rendering services | Spent as payment for compute |

**The utility token problem**: Many tokens claim utility but have none. If a protocol works identically whether or not its token exists, the token is not truly a utility token — it is a speculation vehicle with a utility label.

### 3.2 Governance Tokens

A governance token grants voting rights over protocol parameters, treasury allocation, and upgrade decisions. This is the most common token type in DeFi.

**Characteristics**:
- 1 token = 1 vote (or weighted by lock duration in ve-models)
- Controls protocol parameters (fees, collateral factors, whitelists)
- Often controls a treasury of accumulated fees or tokens

**Examples**: UNI, COMP, AAVE, MKR, JUP, CRV

**The governance token dilemma**: Pure governance tokens have a value problem. If the token only grants voting rights over a protocol that generates no revenue, the rational value of the token is zero (or near-zero). This is why most governance tokens have added additional utility (fee sharing, staking rewards, boosted yields) over time.

### 3.3 Security Tokens

Security tokens represent ownership in a real-world or protocol asset, entitling holders to dividends, revenue, or equity. In most jurisdictions, they are regulated as securities.

**Characteristics**:
- Represent a claim on revenue or assets
- Subject to securities regulation (SEC, MiCA, etc.)
- Often require KYC/AML compliance

**Why most DeFi avoids them**: The regulatory burden of securities tokens is enormous. Most DeFi protocols structure their tokens as utility/governance specifically to avoid securities classification. However, the line is often blurry — if a governance token entitles holders to protocol revenue, regulators may classify it as a security regardless of the label.

### 3.4 LP and Receipt Tokens

These tokens represent a position in a DeFi protocol — they are receipts for deposited assets.

| Token Type | What It Represents | Example |
|---|---|---|
| LP tokens | Share of liquidity pool | UNI-V2 LP tokens, Orca LP |
| cTokens/aTokens | Share of lending pool | Compound cUSDC, Aave aDAI |
| Vault shares | Share of yield vault | Yearn yUSDC, Kamino kTokens |
| Liquid staking tokens | Staked asset + accrued rewards | Marinade mSOL, Jito jitoSOL |

**Key property**: These tokens are **fully backed** by underlying assets. 1 cUSDC is always redeemable for some amount of USDC (the exchange rate grows over time as interest accrues). This makes them fundamentally different from governance or utility tokens.

**Composability**: Receipt tokens are the backbone of DeFi composability. You deposit ETH, receive stETH, deposit stETH into Aave, receive astETH, use that as collateral to borrow... each layer issues a receipt token that the next layer can consume.

### 3.5 Stablecoins as Tokens

Stablecoins have their own tokenomics, focused on maintaining a peg rather than appreciating in value.

**Types by backing**:
- **Fiat-backed**: USDC, USDT (1:1 reserves in bank accounts)
- **Crypto-overcollateralized**: DAI, LUSD (backed by >100% crypto collateral)
- **Algorithmic**: (UST was the cautionary example — now mostly avoided)
- **Hybrid**: FRAX (partially algorithmic, partially collateralized)

**The tokenomics of stablecoin governance**: The stablecoin itself has simple tokenomics (1 token = $1), but the governance token of the stablecoin protocol has complex tokenomics. MKR governs MakerDAO/DAI, and its value depends on stability fee revenue minus bad debt losses.

### 3.6 Meme and Community Tokens

Meme tokens have no formal utility, governance, or backing. Their value derives entirely from community attention, cultural relevance, and speculative momentum.

**Examples**: DOGE, SHIB, BONK, WIF, PEPE

**Tokenomics characteristics**:
- Very large or unlimited supply (billions to trillions)
- Simple token contracts (no staking, no governance, no burns in the original design)
- Value driven by social coordination, not economic fundamentals
- Often have later retrofitted utility (BONK integration into Solana ecosystem apps)

**Why they matter for tokenomics study**: Meme tokens prove that community and narrative can be more powerful than utility in driving token demand. They also demonstrate that simple tokenomics (just a mint) can be more effective than complex, over-engineered systems.

### 3.7 Work and Staking Tokens

Work tokens must be staked by service providers to participate in a network. They align operator incentives through the threat of slashing.

| Network | Token | Stake Requirement | Slash Condition |
|---|---|---|---|
| Ethereum | ETH | 32 ETH to run a validator | Double-signing, inactivity |
| Chainlink | LINK | Stake to be an oracle node | Providing incorrect data |
| The Graph | GRT | Stake to index subgraphs | Incorrect indexing |
| Helium | HNT | Stake to run a hotspot | Providing incorrect coverage proofs |

**Economic model**: Work tokens create a direct link between token value and network demand. If a network processes $1M/day in fees and requires 10% of tokens staked, the staked tokens must be worth at least $10M for the security model to hold.

---

## 4. Supply and Demand Framework

All token economics ultimately reduce to supply and demand. Understanding the specific forces on each side is essential for evaluating any token.

### Supply-Side Forces

| Force | Effect on Price | Examples |
|---|---|---|
| **Token emissions** (mining/staking rewards) | Sell pressure (inflationary) | BTC block rewards, SOL staking inflation |
| **Vesting unlocks** | Sell pressure (team/VC selling) | Monthly linear vests, cliff unlocks |
| **Token burns** | Reduces supply (deflationary) | ETH EIP-1559, BNB quarterly burns |
| **Buybacks** | Removes from market (deflationary) | MKR surplus buybacks |
| **Locking/staking** | Reduces circulating supply | veCRV 4-year locks, SOL staking |
| **Lost/inaccessible tokens** | Permanently reduces supply | Lost BTC wallets (~3.7M BTC) |

### Demand-Side Forces

| Force | Effect on Price | Examples |
|---|---|---|
| **Protocol usage** (utility demand) | Buy pressure | LINK for oracle payments, FIL for storage |
| **Governance rights** | Hold pressure | UNI for Uniswap governance, JUP for votes |
| **Staking rewards** | Hold pressure | SOL staking (~7% APY), ETH staking |
| **Fee sharing** | Hold pressure | GMX stakers earn 30% of trading fees |
| **Speculation** | Volatile buy/sell pressure | All tokens experience this |
| **Collateral demand** | Hold/lock pressure | ETH as collateral in Aave/MakerDAO |

### The Supply/Demand Equilibrium

Token price is the clearing price where:

```
Marginal Buyer's Willingness to Pay = Marginal Seller's Willingness to Accept

Or equivalently:
Price = f(Demand Factors) / f(Supply Factors)
```

**Healthy tokenomics** create sustainable demand forces that exceed or match supply forces. When a token's only demand is speculation but supply increases through emissions, the long-term trajectory is down.

### Circulating vs. Total vs. Max Supply

These are critical but often confused metrics:

| Metric | Definition | Example (SOL) |
|---|---|---|
| **Max supply** | Hard cap (may be infinite) | No hard cap (inflationary, offset by burns) |
| **Total supply** | All minted tokens minus burned | ~590M SOL (Feb 2026) |
| **Circulating supply** | Tokens available for trading | ~430M SOL (excludes staked, locked) |
| **Fully diluted valuation (FDV)** | Price × max supply | If max supply is infinite, FDV is undefined |
| **Market cap** | Price × circulating supply | The standard valuation metric |

**The FDV trap**: Many tokens launch with a small circulating supply (5-10% of total), giving a low market cap but enormous FDV. As vesting unlocks increase circulating supply, the price tends to fall. A token at $1B market cap but $20B FDV means 95% of tokens haven't entered circulation yet — massive future sell pressure.

### Inflation Rate and Its Impact

```
Annual Inflation Rate = (New Tokens Minted per Year) / (Current Circulating Supply)

Real Yield = Nominal Staking APY - Inflation Rate
```

| Token | Nominal Staking APY | Inflation Rate | Real Yield |
|---|---|---|---|
| SOL | ~7.2% | ~5.4% | ~1.8% |
| ETH | ~3.5% | ~0.5% (post-merge, net deflationary in high usage) | ~3.0% |
| ATOM | ~18% | ~14% | ~4% |
| DOT | ~15% | ~10% | ~5% |

Tokens with high nominal APY but equally high inflation are not creating real value for stakers — they are just redistributing newly minted tokens.

---

## 5. The Role of Tokens in Protocol Design

### Why Protocols Issue Tokens

| Reason | Mechanism | Example |
|---|---|---|
| **Fundraising** | Sell tokens to fund development | ICOs, IDOs, private sales |
| **Bootstrap liquidity** | Incentivize early users/LPs | Liquidity mining (COMP, SUSHI) |
| **Decentralize governance** | Distribute control to community | UNI airdrop, JUP distribution |
| **Align incentives** | Make stakeholders economically aligned | Validators staking ETH/SOL |
| **Coordinate behavior** | Reward desired actions, penalize bad ones | Slashing, ve-boosting |
| **Capture value** | Give protocol revenue a destination | Fee sharing to stakers |

### The Token-Protocol Feedback Loop

Well-designed tokenomics create a positive feedback loop:

```
Protocol Usage ↑ → Protocol Revenue ↑ → Token Value ↑ → More Users/LPs Attracted → Protocol Usage ↑
```

Poorly designed tokenomics create a death spiral:

```
Emission Rewards ↑ → Token Price ↓ → APY in Dollar Terms ↓ → Users Leave → TVL ↓ → Token Price ↓↓
```

This is why "real yield" (from actual protocol revenue) is fundamentally superior to "emission yield" (from token printing). Real yield is backed by economic activity; emission yield dilutes existing holders.

### Tokens as Coordination Mechanisms

The deepest insight of tokenomics is that tokens are **coordination tools**, not just assets. They solve coordination problems that traditional companies solve through employment contracts, stock options, and legal agreements.

**Traditional company coordination**: Hire employees → pay salary → give stock options → vest over 4 years → aligned incentives

**Token coordination**: Launch protocol → distribute tokens → vest over time → holders vote on governance → aligned incentives

The key difference: token coordination is **permissionless** and **global**. Anyone can acquire tokens and participate. There are no HR departments, no geographic restrictions, no employment contracts.

---

## 6. Token Value Theories

### The Quantity Theory of Money (Adapted)

The traditional equation of exchange: `MV = PQ`

Adapted for tokens:
```
Token Velocity × Token Market Cap = Protocol Transaction Volume

Where:
  M = Market cap of the token
  V = Velocity (how often each token changes hands per period)
  P = Price level of services denominated in the token
  Q = Quantity of services consumed
```

**Implication**: If velocity is very high (tokens are bought and immediately used/sold), market cap stays low even with high transaction volume. This is the **velocity problem** — tokens that are spent immediately don't accrue value.

**Solutions to the velocity problem**:
- Staking (locks tokens, reducing velocity)
- Burn mechanics (permanently removes tokens)
- Governance rights (reason to hold, not spend)
- Work staking requirements (must hold to participate)

### The Discounted Cash Flow Model

For tokens that entitle holders to cash flows (fee sharing, staking rewards from real yield):

```
Token Value = Σ (Expected Cash Flow_t) / (1 + r)^t

Where:
  Cash Flow_t = Protocol revenue distributed to token holders in period t
  r = Discount rate (risk-adjusted required return)
```

This applies to tokens like MKR (surplus revenue), GMX (30% of trading fees), and ETH (after EIP-1559, net issuance can be negative).

### The Network Value Model (Metcalfe's Law)

```
Network Value ∝ n²

Where n = number of active participants
```

This model suggests that token value scales with the square of users. Empirically, this has been observed in many crypto networks, though the exponent varies (some studies find n^1.5 rather than n^2).

### The Speculative Premium

In practice, most token prices include a large speculative component:

```
Token Price = Fundamental Value + Speculative Premium

Where:
  Fundamental Value = DCF value + Utility value + Governance value
  Speculative Premium = f(narrative, momentum, market conditions, community)
```

For many tokens, the speculative premium dominates. This is not inherently bad — early-stage protocols need speculative interest to bootstrap. But sustainable tokenomics must eventually transition speculative demand into fundamental demand.

---

## 7. The "Does This Need a Token?" Test

The most important question in token design is whether the token should exist at all. Many protocols would be better off without a token. Here is a framework for evaluating necessity:

### The Token Necessity Framework

| Question | If Yes → Token Needed | If No → Probably Not |
|---|---|---|
| Does the protocol require decentralized consensus? | Validators need economic incentives (stake/slash) | Centralized validation works fine |
| Does the protocol need permissionless coordination? | Token aligns strangers' incentives | Traditional contracts suffice |
| Is there a bootstrapping problem (chicken-and-egg)? | Token incentives can kickstart supply/demand | Organic growth is possible |
| Does the protocol generate revenue that should flow to a decentralized set of stakeholders? | Token enables permissionless revenue distribution | Revenue flows to a company |
| Is there a governance coordination problem? | Token enables decentralized decision-making | Team/foundation governs adequately |

### Red Flags: Tokens That Shouldn't Exist

- **The protocol works identically without the token**: If the token is bolted on for fundraising, it adds no value
- **The token is only used as "payment" for services that could accept any currency**: Accepting USDC directly is simpler
- **100% of token demand comes from speculation or emission farming**: No sustainable demand floor
- **The team holds >50% and there is no meaningful vesting**: The token exists to enrich insiders
- **The token creates worse UX than the non-token alternative**: Forcing users to acquire and manage a token they don't need

### Green Flags: Tokens That Should Exist

- **The protocol genuinely decentralizes a service**: Validators, oracle operators, storage providers
- **The token enables a coordination mechanism that wouldn't otherwise exist**: Vote-escrow, gauge systems
- **The token captures real economic value**: Fee sharing, burn mechanics tied to revenue
- **The community benefits from governance rights**: Parameter tuning, treasury allocation
- **The token enables composability**: Receipt tokens (LP, cTokens) that unlock further DeFi usage

---

## 8. Key Terminology Glossary

| Term | Definition |
|---|---|
| **APR** | Annual Percentage Rate — simple interest, not compounded |
| **APY** | Annual Percentage Yield — includes compounding |
| **Airdrop** | Free token distribution to wallets meeting criteria |
| **Bonding curve** | Mathematical function that determines token price based on supply |
| **BPS** | Basis points — 1 bps = 0.01%, 100 bps = 1%, 10000 bps = 100% |
| **Burn** | Permanently destroy tokens (send to dead address or burn function) |
| **Circulating supply** | Tokens currently tradeable on the market |
| **Cliff** | Period before any vesting tokens unlock |
| **DAO** | Decentralized Autonomous Organization — governance by token holders |
| **Dilution** | Reduction in ownership percentage from new token minting |
| **Emission** | New tokens created and distributed over time |
| **Epoch** | Fixed time period in a protocol's schedule (often used in staking) |
| **Fair launch** | No pre-mine, no VC allocation — all tokens earned by community |
| **FDV** | Fully Diluted Valuation — price × maximum supply |
| **Fee switch** | Mechanism to activate/deactivate protocol revenue sharing |
| **Gauge** | Mechanism for token holders to vote on emission distribution |
| **Halving** | Periodic reduction in emission rate (Bitcoin is the canonical example) |
| **IDO** | Initial DEX Offering — token sale on a decentralized exchange |
| **Inflation** | Rate at which new tokens enter circulation |
| **Liquidity mining** | Distributing tokens to liquidity providers |
| **Lock** | Committing tokens for a fixed period (cannot withdraw) |
| **Market cap** | Price × circulating supply |
| **Mercenary capital** | Liquidity that moves to highest yield with no protocol loyalty |
| **Mint** | Create new tokens |
| **Pre-mine** | Tokens created before public launch, allocated to team/investors |
| **Real yield** | Yield from protocol revenue, not token emissions |
| **Rebasing** | Automatically adjusting every holder's balance (up or down) |
| **Slashing** | Penalizing staked tokens for misbehavior |
| **TGE** | Token Generation Event — when the token first becomes tradeable |
| **TVL** | Total Value Locked — assets deposited in a protocol |
| **ve-token** | Vote-escrowed token — locked for governance power |
| **Vesting** | Gradual unlock of tokens over time |
| **Velocity** | How often each token changes hands per period |

---

## 9. References

### Essential Reading

1. **"A Letter to the Community" — Hayden Adams (Uniswap UNI launch, 2020)**: The retroactive airdrop that defined a generation of token distribution
2. **"An Introduction to Token Economics" — Vitalik Buterin (various blog posts)**: First-principles thinking about when tokens are necessary
3. **"Fat Protocols" thesis — Joel Monegro (USV, 2016)**: The argument that value accrues to protocol layers, not application layers
4. **"Thin Applications" counter-thesis — various (2022-2023)**: The evolving debate about where value accrues
5. **Bitcoin whitepaper — Satoshi Nakamoto (2008)**: The original tokenomics design
6. **Curve Finance whitepaper (2020)**: The definitive vote-escrow tokenomics model
7. **MakerDAO whitepaper (2017)**: Governance + stability + economics in one system
8. **"Tokens are not equity" — Placeholder (2017)**: Distinguishing token ownership from equity ownership

### Data Sources

- **Token Terminal**: Protocol revenue, token metrics, FDV vs. market cap
- **DeFiLlama**: TVL, protocol comparisons, yield data
- **Dune Analytics**: On-chain token distribution, holder analysis
- **CoinGecko/CoinMarketCap**: Supply metrics, market cap data
- **Messari**: Token unlock schedules, governance proposals

---

*Next: [02 - Token Supply Mechanics](./02-token-supply-mechanics.md) — Fixed, inflationary, and deflationary supply models, emission schedules, burns, buybacks, and the math behind them.*
