# Tokenomics Research Compendium

A comprehensive research library for mastering token economics, from first principles to production-ready program design on Solana. Written for experienced developers entering the token design space.

---

## Table of Contents

### [01 - Tokenomics Fundamentals](./01-tokenomics-fundamentals.md)
The foundation. What tokenomics is, the three pillars (supply, distribution, utility), history from Bitcoin to modern points systems, token taxonomy (utility, governance, security, LP/receipt, meme, work tokens), supply/demand framework, token value theories (QTM, DCF, Metcalfe), the "does this need a token?" test, and a terminology glossary.

### [02 - Token Supply Mechanics](./02-token-supply-mechanics.md)
Deep dive into supply models. Fixed supply, linear emission, exponential decay (halving), tail emission, demand-responsive emission, deflationary mechanisms (burn on transaction, buyback-and-burn, fee burns), hybrid models (ETH, SOL, CRV), emission schedule math with Rust code, rebasing tokens, and mint authority patterns on Solana (SPL Token + Token-2022).

### [03 - Token Distribution and Launch Strategies](./03-token-distribution-and-launch.md)
How tokens reach users. Allocation models, ICOs/IDOs/LBPs, fair launches (YFI), retroactive airdrops (UNI), tiered airdrops (JUP), points systems, vesting and lockup design with recommended schedules, treasury management, the insider problem (VC discounts), anti-sybil techniques (quadratic, cluster analysis), and Solana program implementations (vesting contract, Merkle airdrop, emission controller).

### [04 - Token Utility and Value Accrual](./04-token-utility-and-value-accrual.md)
Why anyone should hold a token. Governance rights, economic rights (fee sharing, discounts, boosts, insurance), functional requirements (gas, oracle, storage), collateral demand, fee switches and revenue distribution (GMX direct, MKR buyback-burn, SUSHI redistribute, CRV ve-lock), buyback mechanics (TWAP), staking types, the demand stack framework, DCF and P/E valuation math, anti-patterns (fake utility), and Solana implementations (fee distribution, buyback-burn, Token-2022 transfer fee).

### [05 - Staking, Vote-Escrow, and Incentive Design](./05-staking-and-vote-escrow.md)
The deepest mechanics. PoS staking economics (SOL vs ETH), liquid staking, the veCRV model (lock, decay, boost, gauge voting), the Curve Wars (Convex, bribery markets, meta-governance), mercenary capital problem and solutions, mechanism design principles (incentive compatibility, budget balance), emission schedule design with revenue transition targets, slashing design, and Solana implementations (ve-lock, gauge voting, boost calculator).

### [06 - Governance Tokenomics](./06-governance-tokenomics.md)
How token holders make decisions. Voting mechanisms (token-weighted, quadratic, conviction, optimistic, futarchy), delegation models, on-chain vs off-chain governance, voter apathy, governance attacks (flash loan, bribery, capture, Beanstalk exploit), progressive decentralization phases, DAO treasury management (RPGF), case studies (MakerDAO, Uniswap, Compound, Jupiter), and Solana implementations (governance program, snapshot voting, delegation, timelock execution).

### [07 - Tokenomics at the Program Level](./07-tokenomics-program-level.md)
**The builder's guide.** SPL Token architecture (Mint, TokenAccount, ATA), authority patterns, Token-2022 extensions (transfer fees, interest-bearing, soulbound, permanent delegate, confidential transfers, transfer hooks), complete Anchor implementations: emission controller, multi-recipient distribution, linear vesting with cliff and revocation, Synthetix-style staking pool, Merkle airdrop distributor, buyback-and-burn program, governance integration, security checklist, and LiteSVM testing patterns.

### [08 - Tokenomics Case Studies](./08-tokenomics-case-studies.md)
Deep analysis of 11 token designs rated on a 30-point framework. Successes: BTC (26/30 — scarcity standard), ETH (25/30 — adaptive monetary policy), SOL (23/30 — staking-centric), CRV (26/30 — ve-pioneer), MKR (27/30 — governance+buyback), AAVE (25/30 — Safety Module), JUP (26/30 — community-governed supply), JTO (24/30 — MEV-aligned), LINK (22/30 — work token). Cautionary tales: FTT (circular collateral), LUNA/UST (death spiral), OHM (unsustainable APY). Comparative analysis across all dimensions.

### [09 - Tokenomics Design Framework](./09-tokenomics-design-framework.md)
**The design manual.** Step-by-step process (purpose → supply → distribution → demand → governance → simulate → launch), decision matrices and templates for each step, evaluation checklists (red/green flags, 10-question assessment), common mistakes and fixes (launching too early, over-engineering, ignoring sell pressure, circular value), game theory (Nash equilibria, Schelling points, mechanism design), regulatory considerations (Howey Test, MiCA), simulation template (spreadsheet + Python), launch timeline and checklist, and recommended tools and audit firms.

---

## Suggested Reading Order

**Phase 1 — Foundations** (start here):
1. `01-tokenomics-fundamentals.md` — Core concepts and token taxonomy
2. `04-token-utility-and-value-accrual.md` — Why tokens have value

**Phase 2 — Mechanics**:
3. `02-token-supply-mechanics.md` — Supply models and math
4. `03-token-distribution-and-launch.md` — How tokens reach users
5. `05-staking-and-vote-escrow.md` — Staking, ve-tokens, incentive design

**Phase 3 — Governance and Implementation**:
6. `06-governance-tokenomics.md` — DAO governance design
7. `07-tokenomics-program-level.md` — Building on Solana (Anchor code)

**Phase 4 — Application**:
8. `08-tokenomics-case-studies.md` — Learn from successes and failures
9. `09-tokenomics-design-framework.md` — Design your own tokenomics

---

## Companion Research

This compendium is designed to be read alongside the [DeFi Research Compendium](../README.md), which covers AMMs, lending, yield sources, stablecoins, and protocol design patterns. Together, they provide a complete foundation for building DeFi protocols with sound tokenomics on Solana.

---

*Research compiled February 2026. Sources include protocol documentation, whitepapers, governance forums, on-chain data, and industry analysis.*
