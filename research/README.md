# DeFi Research Compendium

A comprehensive research library for mastering Decentralized Finance, from first principles to advanced protocol design. Written for experienced Solana developers entering the DeFi space.

**Total**: ~11,300 lines / ~74,000 words across 9 deep-dive documents.

---

## Table of Contents

### [01 - DeFi Fundamentals](./01-defi-fundamentals.md) (815 lines)
The foundation. What DeFi is, its history from Bitcoin to modern multi-chain, the seven core primitives (DEXes, lending, stablecoins, derivatives, insurance, yield aggregators, liquid staking), composability/money legos, TVL as a metric, smart contract risk, and a comprehensive terminology glossary.

### [02 - AMMs and DEXes](./02-amms-and-dexes.md) (1,110 lines)
Deep mathematical treatment of automated market makers. The constant product formula (x*y=k) with worked numerical examples, LP token mechanics, impermanent loss derivation, Uniswap v3 concentrated liquidity, Curve StableSwap invariant, order book DEXes, Balancer weighted pools, MEV/sandwich attacks, and Solana DEX landscape (Jupiter, Raydium, Orca, Phoenix, Meteora).

### [03 - Lending and Borrowing](./03-lending-and-borrowing.md) (954 lines)
How DeFi lending works end-to-end. Overcollateralized lending mechanics, interest rate models with exact formulas (kink/jump model), collateral factors, health factor calculations, liquidation mechanics with worked examples, flash loans and attack case studies (Euler $197M, Beanstalk $182M), and protocol deep dives (Aave, Compound, MakerDAO, Kamino, MarginFi).

### [04 - Where Yield Comes From](./04-where-yield-comes-from.md) (1,092 lines)
**The most critical file.** The iron law: every yield has a source. Eight sustainable yield sources (trading fees, borrowing interest, staking, liquid staking, protocol revenue, MEV, RWA, funding rates). Five unsustainable sources (token emissions, ponzi mechanics, subsidized yields, recursive looping, rebasing). Case studies, a yield evaluation framework, realistic yield ranges, and the "you are the yield" principle.

### [05 - Stablecoins and DeFi Failures](./05-stablecoins-and-defi-failures.md) (993 lines)
Stablecoin mechanics (fiat-backed, crypto-backed, algorithmic, hybrid) and stability mechanisms. Then the failures: Terra/Luna/UST collapse day-by-day timeline, Iron Finance, OlympusDAO, Wonderland, Celsius, FTX/Alameda contagion, Mango Markets exploit, Wormhole hack, Ronin bridge hack, Beanstalk governance attack. Common failure patterns and a due diligence checklist.

### [06 - Solana DeFi Ecosystem](./06-solana-defi-ecosystem.md) (1,252 lines)
Solana-specific. Why SVM advantages matter for DeFi, 15+ protocol deep dives (Jupiter, Raydium, Orca, Phoenix, Kamino, MarginFi, Marinade, Jito, Drift), account model implications, Token-2022 for DeFi, CPI composability patterns, oracle integration (Pyth, Switchboard), key program IDs reference table, historical events (Wormhole hack, Mango exploit, FTX impact, recovery), and security considerations.

### [07 - Key Papers and Math](./07-key-papers-and-math.md) (1,688 lines)
Foundational whitepapers (Uniswap v1/v2/v3, Curve, Compound, Aave, MakerDAO, Balancer, Bancor) with key formulas extracted. Academic papers (Angeris CFMM theory, Flash Boys 2.0, Loss-Versus-Rebalancing). Mathematical foundations: CFMM general theory, bonding curves, interest rate models, options pricing, risk modeling. Security research and audit checklists. Recommended reading order.

### [08 - DeFi Protocol Design](./08-defi-protocol-design.md) (1,951 lines)
**The builder's guide.** Pool architectures, vault patterns, share-based accounting math (with rounding and first-depositor attack defenses), oracle design (TWAP, Pyth, Switchboard with code), liquidation engine design (fixed-price, Dutch auction, LLAMMA), risk management frameworks, governance models, fee design, ve-token model with Rust code, the Curve Wars, and comprehensive security best practices with invariant testing.

### [09 - Advanced Strategies and Trends](./09-advanced-strategies-and-trends.md) (1,486 lines)
Yield strategies (looping, delta-neutral, basis trading, JLP, vaults, carry trades), liquid staking and restaking, perpetual DEXes (vAMM, LP-as-counterparty, order book), MEV deep dive (Solana vs Ethereum, Jito architecture), cross-chain bridges and their vulnerabilities, DeFi composability in practice, and emerging trends (intent-based trading, modular DeFi, RWA, AI agents/DeFAI, confidential balances, account abstraction).

---

## Suggested Reading Order

**Phase 1 - Foundations** (start here):
1. `01-defi-fundamentals.md` - Get the lay of the land
2. `04-where-yield-comes-from.md` - Understand the economics before the mechanics

**Phase 2 - Core Mechanics**:
3. `02-amms-and-dexes.md` - How trading works on-chain
4. `03-lending-and-borrowing.md` - How lending works on-chain
5. `05-stablecoins-and-defi-failures.md` - What can go wrong (and has)

**Phase 3 - Building**:
6. `06-solana-defi-ecosystem.md` - Your home chain's DeFi landscape
7. `08-defi-protocol-design.md` - How to architect DeFi programs
8. `07-key-papers-and-math.md` - The formal foundations (reference as needed)

**Phase 4 - Mastery**:
9. `09-advanced-strategies-and-trends.md` - Advanced strategies and where DeFi is heading

---

*Research compiled February 2026. Sources include protocol documentation, academic papers, whitepapers, and industry analysis.*
