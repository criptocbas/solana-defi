# The Solana DeFi Ecosystem: A Comprehensive Guide for Experienced Solana Developers

> **Last Updated**: February 2026
> **Audience**: Experienced Solana developers new to DeFi
> **Scope**: Architecture, protocols, patterns, history, and practical development context

---

## Table of Contents

1. [Why Solana for DeFi](#1-why-solana-for-defi)
2. [Solana-Specific DeFi Architecture](#2-solana-specific-defi-architecture)
3. [Major Solana DeFi Protocols](#3-major-solana-defi-protocols)
4. [The Jupiter Ecosystem](#4-the-jupiter-ecosystem)
5. [Jito and MEV on Solana](#5-jito-and-mev-on-solana)
6. [Solana DeFi Program Patterns](#6-solana-defi-program-patterns)
7. [Key Solana DeFi Events and History](#7-key-solana-defi-events-and-history)
8. [Key Program IDs Reference](#8-key-program-ids-reference)
9. [Security Considerations](#9-security-considerations)
10. [Sources and References](#10-sources-and-references)

---

## 1. Why Solana for DeFi

### 1.1 Performance Advantages

Solana's architecture provides fundamental advantages for DeFi applications that simply cannot be replicated on EVM-based chains:

**Speed and Finality**:
- ~400-millisecond block times (slot times), with the upcoming "Alpenglow" overhaul (approved September 2025) targeting ~100-150ms finality via new components like Votor (direct voting) and Rotor (rapid propagation)
- Sub-second trade execution enables CEX-like trading experiences on-chain
- 100% uptime throughout 2025

**Transaction Costs**:
- Fees consistently under $0.001 per transaction
- Localized fee markets mean congestion in one program (e.g., an NFT mint) does not affect fees for other programs (e.g., a DEX swap). This is a critical advantage over EVM chains which use global fee markets
- Solana generates $1.03M/24h in chain fees compared to ~$182K across Ethereum L2s

**Parallel Execution via Sealevel**:
- Sealevel is Solana's parallel transaction execution engine
- Transactions declare their read/write account sets up front, allowing the scheduler to run non-overlapping work concurrently across all available CPU cores
- The SVM can process tens of thousands of contracts simultaneously, vs. the EVM which processes them sequentially
- Each token type lives in its own account, so transactions involving different tokens process in parallel without interference

**Composability**:
- Globally ordered state simplifies cross-program interactions
- A single transaction can atomically compose multiple DeFi operations (swap, deposit, stake) via Cross-Program Invocations (CPI)
- No need for flash loan wrappers or complex multi-transaction patterns common on EVM chains

### 1.2 Solana vs. EVM Chains for DeFi

| Feature | Solana (SVM) | Ethereum (EVM) |
|---|---|---|
| Execution Model | Parallel (Sealevel) | Sequential |
| Fee Market | Localized per program | Global |
| Block Time | ~400ms (targeting ~100ms) | ~12 seconds |
| Transaction Cost | <$0.001 | $0.50-$50+ (mainnet) |
| State Model | Accounts (code/data separated) | Contract storage (unified) |
| Composability | CPI (4-level depth) | Internal calls (unlimited depth) |
| Token Standard | SPL Token / Token-2022 | ERC-20 / ERC-721 |
| Oracle Updates | Pull-based (Pyth) | Push-based (Chainlink) |

### 1.3 Current Market Position (2025-2026)

- **DeFi TVL**: $11.5 billion (December 2025), second-largest DeFi blockchain
- **Market Share**: 7.05% of global DeFi market, ahead of BSC, Bitcoin, and Tron
- **Lending Markets**: $3.6 billion TVL in lending alone
- **Stablecoin Market Cap**: Exceeding $15.5 billion entering 2026
- **DEX Volume**: Jupiter alone processes >50% of Solana's DEX volume with ~93.6% aggregator market share
- **Liquid Staking**: 13.3% of all staked SOL (57 million SOL, ~$10B) held in LSTs as of October 2025

---

## 2. Solana-Specific DeFi Architecture

### 2.1 The Account Model and DeFi Program Design

Understanding Solana's account model is fundamental to DeFi development. Unlike EVM chains where contracts hold their own state, Solana separates code and data completely.

**Core Principles**:
- **Programs are stateless**: They contain only executable logic, no persistent storage
- **Data lives in accounts**: All state is stored in separate data accounts managed by the runtime
- **Ownership model**: Only an account's owner program can modify its data or debit its lamports
- **Transaction pre-declaration**: Every transaction must list all accounts it will read/write before execution

**Account Categories**:
```
Program Accounts       Data Accounts
+----------------+    +------------------+
| Executable     |    | Non-executable   |
| Immutable data |    | Mutable data     |
| Owner: BPF     |    | Owner: A program |
+----------------+    +------------------+
```

**Impact on DeFi Design**:
- A DEX pool is not "inside" the DEX program; it is a separate account owned by the program
- User token balances are individual accounts, not entries in a mapping
- Parallel processing is possible because account access is declared upfront
- Programs can serve multiple pools/vaults without redeployment (they are stateless interfaces to state accounts)

### 2.2 The SPL Token Program

The SPL Token Program is the foundation of all DeFi on Solana. It manages three core account types:

**Mint Account** (`TokenkegQfeZyiNwAJbNbGKPFXCWuBvf9Ss623VQ5DA`):
- Represents a specific token type
- Stores global state: total supply, decimal precision, mint authority, freeze authority
- One mint per token type

**Token Account**:
- Represents ownership of tokens for a given mint and owner
- Stores: mint reference, owner pubkey, balance, delegate info
- Multiple token accounts can exist for the same mint/owner

**Associated Token Account (ATA)**:
- A canonical token account whose address is deterministically derived from owner + mint
- Program: `ATokenGPvbdGVxr1b2hvZbsiqW5xWH25efTNsLJA8knL`
- Standard pattern for most DeFi interactions

```rust
// Deriving an ATA address
let ata = get_associated_token_address(
    &wallet_pubkey,    // owner
    &token_mint,       // mint
);
```

### 2.3 Token-2022 (Token Extensions) and DeFi Implications

Token-2022 (`TokenzQdBNbLqP5VEhdkAS6EPFLC1PHnBqCXEpPxuEb`) extends the original token program with new capabilities:

**Transfer Fee Extension**:
- Automatic fee collection on every token transfer
- Fees accumulate in recipient token accounts
- Withdrawable by a designated withdraw authority on the mint
- DeFi implication: Protocols can build sustainable fee models directly into the token
- Fee tiers configurable at mint creation

**Confidential Transfer Extension**:
- Enables token transfers with hidden balances and amounts
- Uses zero-knowledge proofs (Zk-Proofs) to verify correctness without revealing values
- DeFi implication: Enterprise tokens can enable privacy-preserving transactions
- **Critical limitation**: Cannot combine ConfidentialTransfer with TransferHook extensions because confidential transfers encrypt amounts, and transfer hooks rely on reading the transferred amount

**Other Extensions Relevant to DeFi**:
- **Interest-Bearing Tokens**: Token balances that display with accrued interest
- **Non-Transferable Tokens**: Soulbound tokens for KYC/reputation
- **Permanent Delegate**: A delegate that cannot be revoked (useful for compliance)
- **Transfer Hook**: Custom logic executed on every transfer (composable fee/compliance logic)

**Developer Warning**: When working with Token-2022, you must explicitly pass the Token-2022 program ID to your instructions, as the default typically points to the original Token Program. The ATA program handles both standards.

### 2.4 Cross-Program Invocations (CPI) for DeFi Composability

CPI is the mechanism that makes Solana DeFi composable. It allows any program to call any other program within a single transaction.

**How CPI Works**:
```rust
// Basic CPI: Your DeFi program calling the Token program to transfer tokens
invoke(
    &transfer_instruction,
    &[source_account, dest_account, authority_account],
)?;

// CPI with PDA signer: Your program signing as a PDA
invoke_signed(
    &transfer_instruction,
    &[vault_account, dest_account, pda_authority],
    &[&[b"vault", pool_id.as_ref(), &[bump]]],
)?;
```

**DeFi Composability Examples**:
- A lending protocol calls a DEX via CPI to liquidate collateral
- A vault strategy calls a lending protocol to deposit, then a DEX to swap rewards
- `swap_and_stake`: swap tokens on a DEX through CPI, then stake via another CPI

**Constraints**:
- **Stack depth limit**: CPI depth is limited to 4 levels (transaction -> program A -> program B -> program C -> program D)
- **Compute unit budget**: Default 200,000 CU per instruction, max 1.4M CU per transaction
- **Account list**: All accounts touched by any CPI must be declared in the original transaction

### 2.5 PDAs for Authority Management in DeFi Programs

Program Derived Addresses (PDAs) are the cornerstone of DeFi authority management on Solana.

**Why PDAs Matter for DeFi**:
- PDAs have no private key -- only the program can sign for them
- They serve as vault authorities, pool authorities, and protocol-controlled accounts
- They enable trustless custody: users deposit to a PDA-controlled account, and only the program logic can authorize withdrawals

**Common DeFi PDA Patterns**:
```rust
// Pool authority PDA
let (pool_authority, bump) = Pubkey::find_program_address(
    &[b"pool_authority", pool_id.as_ref()],
    &program_id,
);

// User position PDA
let (user_position, bump) = Pubkey::find_program_address(
    &[b"position", user.as_ref(), pool.as_ref()],
    &program_id,
);

// Vault PDA (holds tokens)
let (vault, bump) = Pubkey::find_program_address(
    &[b"vault", mint.as_ref()],
    &program_id,
);
```

**Security Consideration**: When deriving PDAs for vaults, always include sufficient seeds to prevent PDA sharing attacks. Using only the mint as a seed is insecure because multiple pool accounts could be created for the same vault token account with different withdrawal destinations. Include the pool ID or withdrawal destination as a seed.

---

## 3. Major Solana DeFi Protocols

### 3.1 Decentralized Exchanges (DEXes)

#### 3.1.1 Jupiter (Aggregator)

**Program ID**: `JUP6LkbZbjS1jKKwapdHNy74zcZ3tLUZoi5QNyVTaV4`

Jupiter is the dominant DEX aggregator on Solana, handling >50% of all Solana DEX volume and controlling ~93.6% of the aggregator market. It does not hold liquidity itself but routes trades through integrated DEXes.

**Architecture (Ultra V3, Late 2025)**:
- **Iris Router**: Replaced the prior Metis router with advanced algorithms (Golden-section search, Brent's method) for optimal order splitting across routes. 100x performance improvement in pathfinding
- **Meta-Aggregation**: Pulls quotes from other routers (DFlow, Hashflow, CEX RFQs from OKX) alongside its own routing
- **On-Chain Slippage Simulation**: Simulates each candidate route on-chain (dry-run) before executing, choosing the route with highest predicted output
- **Beam**: Private transaction relay for sub-second execution

**Products**:
- Token swaps (aggregated)
- Limit orders (via Phoenix/OpenBook integration)
- DCA (Dollar-Cost Averaging) / Recurring orders
- Perpetuals (see Section 4)
- Jupiter Lend (launched August 2025, $1.65B TVL by October 2025)

**Developer Integration**:
```bash
# Jupiter API endpoint
curl -s "https://quote-api.jup.ag/v6/quote?inputMint=So11111111111111111111111111111111111111112&outputMint=EPjFWdd5AufqSSqeM2qN1xzybapC8G4wEGGkZwyTDt1v&amount=1000000000"
```

#### 3.1.2 Raydium (AMM + CLMM)

**Program IDs**:
- CLMM: `CAMMCzo5YL8w4VFF8KVHrK22GGUsp5VTaW7grrKgrWqK`
- CPMM (Standard AMM): `CPMMoo8L3F4NbTegBCKVNunggL7H1ZpdTHKxQB5qKP1C`
- Legacy AMM v4: `675kPX9MHTjS2zt1qfr1NYHuzeLXfQM9H24wFSUt1Mp8`

Raydium is Solana's largest DEX by TVL ($2.3B in Q3 2025, growing 32.3% QoQ).

**Key Features**:
- Concentrated Liquidity Market Maker (CLMM) with multiple fee tiers
- Standard constant product AMM (CPMM)
- Integration with OpenBook central limit order book (CLOB) for limit orders
- Low-slippage trades in major SPL pairs
- Perps with 0% maker fees

**Architecture**:
- Hybrid model combining AMM pools with order book integration
- LP positions represented as on-chain accounts with position data
- Each pool is a set of accounts: pool state, token vaults (token A, token B), LP mint

#### 3.1.3 Orca (Whirlpools CLMM)

**Program IDs**:
- Whirlpool CLMM: `whirLbMiicVdio4qvUfM5KAg6Ct8VwpYzGff3uctyCc`
- Legacy AMM V1: `9W959DqEETiGZocYWCQPaJ6sBmUzgfxXfqGeTEdp3aQP`

Orca is Solana's second-largest DEX by TVL, known for its user-friendly design and concentrated liquidity "Whirlpools."

**Whirlpools Architecture**:
- Concentrated liquidity exclusively (no standard AMM pools)
- Fee tiers: 0.01%, 0.05%, 0.30%, 1.00% (selected based on expected pair volatility)
- LPs set price ranges for their liquidity
- Single-sided liquidity provision lowers barriers for retail users
- SOL/USDC pool: $31.3M TVL, $22.8B 30-day trading volume (May 2025)

**Developer Resources**: Orca provides a well-documented TypeScript SDK (`@orca-so/whirlpools-sdk`) for interacting with Whirlpools programmatically.

#### 3.1.4 Phoenix (On-Chain Order Book)

**Program ID**: `PhoeNiXZ8ByJGLkxNfZRnkUfjvmuYqLR89jjFHGqdXY`

Phoenix is an on-chain central limit order book (CLOB) DEX developed by Ellipsis Labs.

**Key Innovation -- Crankless Design**:
- Traditional on-chain order books (like the former Serum/OpenBook) require "cranking" -- external bots that process and match orders
- Phoenix eliminates this requirement, atomically settling trades on-chain without external cranks
- This reduces latency, removes dependence on third-party keepers, and simplifies the system

**Architecture**:
- Full on-chain CLOB with maker/taker orders
- Atomic settlement in a single transaction
- Extremely fast and low-cost order placement and cancellation
- Jupiter integrates Phoenix for limit order execution

#### 3.1.5 Lifinity (Proactive Market Making)

**Website**: lifinity.io

Lifinity is the first proactive market maker on Solana, introducing oracle-driven pricing instead of the standard AMM constant product formula.

**Key Innovation -- Oracle-Based Pricing**:
- Uses oracles (primarily Pyth) as the key pricing mechanism
- Eliminates reliance on arbitrageurs to correct pool prices
- Reduces or even *reverses* impermanent loss for market makers
- Concentrated liquidity with automatic range selection

**Tokenomics & Services**:
- Protocol continuously acquires permanent liquidity for all supported pairs
- veToken model with optional decaying, linear unlocking, native tokenization
- Liquidity as a Service (LaaS): protocols can bribe veToken holders for permanent liquidity
- Revenue is passed to token holders

#### 3.1.6 Meteora (Dynamic AMM + DLMM)

**Program ID (DLMM)**: `LBUZKhRxPF3XUpBCjp4YzTKgLccjZhTSDM9YuVaPwxo`

Meteora functions as a liquidity layer for Solana's DeFi ecosystem, providing backend liquidity infrastructure.

**Products**:
- **DLMM (Dynamic Liquidity Market Maker) Pools**: Dynamically adjusts fees and liquidity concentration based on market conditions. Bin-based liquidity system for precise price range allocation
- **Dynamic AMM Pools**: Automatically allocate idle liquidity to integrated lending protocols for additional LP yield
- **Dynamic Vaults**: Yield aggregation vaults that deploy across Solana lending markets and rebalance hourly

**2025 Performance**:
- Trading volume surged from $987M (Dec 2024) to $39.9B (Jan 2025) -- a 40x increase
- Became highest fee-generating platform on Solana in May 2025 ($5.37M daily fees)
- TVL: >$1.1B (September 2025)
- MET token TGE launched October 2025

### 3.2 Lending Protocols

#### 3.2.1 Kamino Finance (K-Lend)

**Program ID (K-Lend)**: `KLend2g3cP87fffoy8q1mQqGKjrxjC8boSyAYavgmjD`

Kamino dominates Solana's lending landscape with $3.5B TVL (December 2025), representing ~75% of Solana's lending market.

**Architecture**:
- **Market Layer**: Isolated lending markets for risk segmentation
- **Vault Layer**: Automated yield vaults managed by risk experts (Steakhouse, Re7 Labs)
- Modular architecture resembling Morpho's DeFi-as-infrastructure approach

**Products**:
- K-Lend: Supply assets to earn yield, borrow against collateral
- Curated "Earn" vaults (Conservative, Balanced, Aggressive risk profiles)
- Automated Liquidity Vaults for specific price ranges
- Multiply and Long/Short Vaults for leveraged positions

**Risk Management**:
- Zero bad debt since inception
- 18 audits + formal verification
- November 2025: processed $26.5M in collateral through 16,228 liquidation events with zero bad debt
- Real-World Asset (RWA) integration (partnerships with Hastra, Figure, Maple)

#### 3.2.2 MarginFi (mrgnlend)

**Program ID**: `MFv2hWf31Z9kbCa1snEPYctwafyhdvnV7FZnsebVacA`

MarginFi is a decentralized lending protocol offering overcollateralized lending/borrowing.

**Architecture**:
- **Global Market**: Main lending pool where all assets are interconnected (collateral in one asset enables borrowing another)
- **Isolated Markets**: Separate pools for new/riskier tokens that don't qualify for the global market
- Cross-asset collateralization within the global pool

**2025 Developments**:
- Acquired by Project 0 (0.xyz) with plans for TGE
- Points migration 1:1 for active depositors
- Cross-venue margining features planned with other platforms
- Q1 2025: 42% TVL drop but $1.7B in liquidations and $88.5M revenue demonstrated durability
- Typical yields: USDC 5-8% APY, SOL 10-15% in bull markets

#### 3.2.3 Save (formerly Solend)

**Website**: save.finance

Save is one of Solana's oldest lending protocols, rebranded from Solend in 2024.

**Rebrand Products**:
- Core lending/borrowing (continued from Solend)
- **sUSD**: Native stablecoin
- **saveSOL**: Liquid staking token
- Platform for shorting meme coins
- SLND token converted to SAVE token

**Historical Significance**: Solend was among the first lending protocols on Solana and played a key role in bootstrapping the ecosystem's DeFi TVL. Reached $400M+ TVL in August 2024.

### 3.3 Liquid Staking

#### 3.3.1 Marinade Finance (mSOL)

**Program ID**: `MarBmsSgKXdrN1egZf5sqe1TMai9K1rChYNDJgjq7aD`
**mSOL Mint**: `mSoLzYCxHdYgdzU16g5QSh3i5K3z3KZK7ytfqcJm7So`
**MNDE Token**: `MNDEFzGvMt87ueuHvVU9VcTqsAP5b3fTGPsHuuPA5ey`

Marinade Finance launched on mainnet in August 2021 and was the first liquid staking protocol on Solana.

**Architecture**:
- Users stake SOL, receive mSOL (value-accruing LST)
- Staking rewards accumulate directly in mSOL (the mSOL:SOL ratio increases over time)
- Delegates to 100+ validators, including MEV-optimized Jito validator clients
- MEV rewards collected and restaked on user's behalf

**Products**:
- **Liquid Staking**: Stake SOL -> receive mSOL for DeFi use
- **Native Staking**: Earn staking rewards without smart contract exposure (addresses ~60% of SOL supply that is natively staked vs. ~5% liquid staked)

**DeFi Integration**: mSOL is deeply integrated across lending (Kamino, MarginFi), DEXes (Raydium, Orca), and yield protocols as collateral, LP component, and borrowable asset.

#### 3.3.2 Jito (JitoSOL - MEV Rewards)

**Stake Pool Account**: `Jito4APyf642JPZPx3hGc6WWJ8zPKtRbRs4P815Awbb`
**Stake Pool Program**: `SPoo1Ku8WFXoNDMHPsrGSTSG1Y47rzgn41SLUNakuHy`
**JitoSOL Mint**: `J1toso1uCk3RLmjorhTtrVwY9HJ7X8V9yYac6Y7kGCPn`

Jito launched liquid staking in late 2022 with a unique MEV-enhanced yield model.

**Key Differentiator -- MEV Rewards**:
- Exclusively delegates to validators running the Jito-Solana validator client
- A portion of MEV rewards is distributed to JitoSOL holders
- Yields reach 6-7%+ depending on MEV activity
- ~4% of total rewards go to Jito as protocol fees

**Products**:
- Liquid staking (SOL -> JitoSOL)
- MEV infrastructure (Block Engine, bundles -- see Section 5)
- Jito Restaking
- TipRouter (see Section 5)

#### 3.3.3 BlazeStake (bSOL)

BlazeStake is a community-governed stake pool backed by the Solana Foundation.

**Key Features**:
- Distributes across 200+ validators (largest validator set of any Solana stake pool)
- First-ever global rewards program for SOL liquid staking
- Users can delegate to specific validators or use the balanced pool
- Steady ~6% APY

#### 3.3.4 JupSOL (Jupiter LST)

JupSOL is Jupiter's liquid staking token, powered by Sanctum's infrastructure.

**Key Features**:
- Introduced April 2024, quickly reached 3.7M SOL TVL (~8.9% LST market share)
- Combines institutional-grade security with Jupiter ecosystem integration
- One-click staking via Jupiter's interface
- Deep on-chain liquidity via Sanctum's Infinity pool

#### 3.3.5 Sanctum (LST Infrastructure)

**Sanctum's Infinity Pool**:
- Multi-LST liquidity pool enabling swaps between all LSTs
- Zero price impact trades (no constant-product formula)
- Redemptions can route through any asset in the pool, reducing depeg risk
- INF token: Multi-LST strategies yield 9.17%+ APY
- Facilitated >9.6M SOL (~$1.4B) in trades since launch

**Overall LST Market** (November 2025): 13.76% of all staked Solana is in LSTs (60.5M SOL, ~$10B).

### 3.4 Perpetuals and Derivatives

#### 3.4.1 Jupiter Perps

**Program ID**: `PERPHjGBqRHArX4DySjwM6UJHiR3sWAatqfdBS2qQJu`
**JLP Token**: `27G8MtK7VtTcCHkpASjSDdkWWYfoqT6ggEuKidVJidD4`

Jupiter Perps is an LP-based perpetual exchange using oracle prices.

**Architecture**:
- JLP Pool holds SOL, ETH, wBTC, USDC, and USDT
- Leveraged long/short trading up to 250x on major assets
- Oracle-priced (primarily Pyth) -- no AMM curve for price discovery
- Counter-based system for borrow fee calculation (efficient vs. real-time calculation)
- Gasless orders using keeper execution model

**How It Works**:
1. LPs deposit into the JLP pool, receiving JLP tokens
2. Traders open leveraged positions against the JLP pool
3. The pool acts as counterparty -- when traders profit, the pool pays; when traders lose, the pool gains
4. LPs earn from trading fees, borrow fees, and trader losses

See Section 4 for full Jupiter ecosystem details.

#### 3.4.2 Drift Protocol

**Program ID**: `dRiftyHA39MWEi3m9aunc5MzRF1JYuBsbn6VPcn33UH`

Drift is the largest open-source perpetual futures exchange on Solana, offering perpetual futures, spot trading, and vaults with up to 10x leverage.

**Hybrid Liquidity Architecture**:
- **Dynamic AMM (DAMM)**: Provides backstop liquidity with dynamic pricing
- **Decentralized Limit Order Book (DLOB)**: Off-chain orderbook maintained by a network of incentivized Keepers
- **Keeper Network**: Monitors on-chain limit orders, builds off-chain orderbook, matches taker orders with resting limit orders or routes to AMM

**Account Structure**:
- User Account/Subaccount: Holds collateral, open positions, P&L
- Multiple subaccounts per primary account (strategy/risk segregation)
- Cross-margined risk engine: Entire collateral balance used across assets/positions

**Drift v3 (December 2025)**:
- 10x faster trade execution (rebuilt backend)
- ~85% of market orders fill in under half a second
- Slippage on larger trades reduced to ~0.02%

#### 3.4.3 Zeta Markets / Bullet

Zeta Markets was a fully on-chain CLOB perpetuals DEX on Solana. It ceased Solana mainnet operations on May 1, 2025, pivoting to **Bullet**, a Solana Layer 2 trading layer.

**Bullet (Successor)**:
- Live since late September 2025
- 1.2ms latency data availability
- $ZEX token evolving into $BULLET (used for gas, node operations, staking)
- At peak, Zeta held ~40% of Solana's perp market with >6M trades

### 3.5 Yield Aggregators and Vaults

#### 3.5.1 Kamino Finance (Vaults)

Kamino's vault layer (separate from K-Lend) provides automated yield strategies:

- **Curated Vaults**: Managed by risk experts, dynamically rebalanced
- **Automated Liquidity Vaults**: Concentrated liquidity management
- **Vault Layer TVL**: ~$593M (September 2025), doubling month-over-month
- Strategies span stablecoin yield farming, leveraged positions, and LP management

#### 3.5.2 Tulip Protocol

Tulip was the first yield aggregation platform built on Solana with auto-compounding vault strategies.

**Products**:
- **Vaults**: Auto-compounding (harvest + reinvest every 10 minutes) for LP positions from Raydium, Orca, Saber
- **Lending**: Powered by Solend/Save, auto-supplies deposited tokens
- **Leveraged Farming**: Up to 3x leverage on yield farming positions
- TULIP token for governance and fee-sharing

### 3.6 Stablecoins on Solana

#### Native USDC
- Circle issues native USDC directly on Solana (not bridged)
- Primary stablecoin for Solana DeFi
- Visa settled live USDC transactions over Solana starting September 2023
- Mint: `EPjFWdd5AufqSSqeM2qN1xzybapC8G4wEGGkZwyTDt1v`

#### USDT
- Tether issues native USDT on Solana
- Second-most used stablecoin in the ecosystem
- Mint: `Es9vMFrzaCERmJfrF4H2FYD4KCoNkY11McCe8BenwNYB`

#### UXD Protocol
- Algorithmic stablecoin backed by delta-neutral positions
- Innovation: Uses perpetual futures contracts to create on-chain delta-neutral positions
- Mechanism: User deposits SOL -> receives $1 UXD. SOL is deposited to perp exchange as collateral + a short position is opened at equal value. Price moves cancel out (delta neutral)
- Addresses the stablecoin trilemma: decentralization, stability, capital efficiency
- Initially used Mango Markets for the perp positions (impacted by Mango exploit)

---

## 4. The Jupiter Ecosystem

Jupiter has evolved from a simple DEX aggregator into a comprehensive DeFi "superapp" on Solana.

### 4.1 The Aggregator Engine

**How Routing Works**:
1. User requests a quote (input mint, output mint, amount)
2. Iris router discovers all possible paths across integrated DEXes
3. Routes are split using Golden-section search and Brent's method for optimal distribution
4. On-chain slippage simulation runs a dry-run of each candidate route
5. Best route (highest actual predicted output) is selected
6. Transaction is built with multi-hop swap instructions via CPI

**Integrated Liquidity Sources**:
- Raydium (AMM + CLMM)
- Orca Whirlpools
- Phoenix order book
- Lifinity
- Meteora DLMM
- OpenBook
- DFlow, Hashflow
- OKX CEX RFQs (via meta-aggregation)

**Developer Integration**:
```typescript
// Jupiter V6 API
const quoteResponse = await fetch(
  `https://quote-api.jup.ag/v6/quote?` +
  `inputMint=${inputMint}&outputMint=${outputMint}&amount=${amount}&slippageBps=50`
).then(res => res.json());

const { swapTransaction } = await fetch('https://quote-api.jup.ag/v6/swap', {
  method: 'POST',
  headers: { 'Content-Type': 'application/json' },
  body: JSON.stringify({
    quoteResponse,
    userPublicKey: wallet.publicKey.toString(),
  })
}).then(res => res.json());
```

### 4.2 Limit Orders

**Program ID**: `jupoNjAxXgZ4rjzxzPMP4oxduvQsQtZzyknqvzYNrNu`

- On-chain limit orders that execute at specified price targets
- Leverages Phoenix and OpenBook for deterministic, on-chain settlement
- Keeper bots monitor and execute orders when price conditions are met
- No expiration by default (configurable)

### 4.3 Dollar-Cost Averaging (DCA) / Recurring Orders

- Schedule recurring buys or sells over time
- Funds execute in periodic batches per user-defined intervals
- One of Solana's most widely used on-chain automation tools
- Minimal fees, high reliability
- Enables time-based strategies impossible on slower/costlier chains

### 4.4 Jupiter Perpetuals

**Architecture**:
```
Trader ---- [Keeper] ----> Jupiter Perps Program
                                  |
                           JLP Pool (SOL, ETH, BTC, USDC, USDT)
                                  |
                           Pyth Oracle Prices
```

- LP-based perps (JLP pool acts as counterparty to all trades)
- Up to 250x leverage on SOL, BTC, ETH
- Oracle-priced execution (no AMM slippage for entry/exit)
- Counter-based borrow fee system (avoids real-time per-position calculation)
- Keeper-executed gasless orders
- JLP token accrues trading fees, borrow fees, and net trader P&L

### 4.5 Jupiter Lend

- Launched August 2025, crossed $500M TVL in 24 hours
- $1.65B TVL across isolated vaults by October 2025
- Features rehypothecation, high LTV ratios, low liquidation penalties
- Isolated vault design for risk segmentation

---

## 5. Jito and MEV on Solana

### 5.1 How MEV Works on Solana vs. Ethereum

**Ethereum MEV**:
- Block builders construct full blocks for validators
- MEV is extracted through transaction ordering within the block
- PBS (Proposer-Builder Separation) creates a marketplace for block space
- Flashbots enables bundles of transactions

**Solana MEV -- Key Differences**:
- Solana has no mempool -- transactions go directly to the current leader
- No PBS by default -- the validator is also the block builder
- MEV extraction happens through transaction ordering within the continuous stream of transactions
- Jito introduced the bundle mechanism to bring structured MEV to Solana

### 5.2 Jito Infrastructure

**Jito Validator Client**:
- Modified Solana validator client running on ~95% of stake (April 2025)
- Enables bundle processing alongside regular transactions
- Does NOT replace the standard transaction flow -- it adds bundle capability

**Jito Block Engine**:
- Receives bundles from searchers
- Runs an auction: bundles compete on tip amount for inclusion
- Forwards winning bundles to the current block leader
- Replaces transaction spamming with priority auctions (improves network health)

**Block Assembly Marketplace (BAM)** (July 2025):
- New system for block assembly and transaction sequencing
- More open, modular, transparent, and distributed
- Independently operated scheduler nodes
- Overhauls the existing block engine architecture

### 5.3 Jito Bundles

**What Are Bundles?**:
- Groups of up to 5 transactions
- Execute sequentially and atomically within the same block
- All succeed together or none are processed
- Minimum tip: 1,000 lamports (~$0.00015)

**When to Use Bundles**:
- Guaranteed sequential execution (e.g., create account + initialize + deposit)
- Operations exceeding 1.4M CU limit per transaction
- Atomic execution across multiple transactions (MEV protection, complex DeFi)
- Sandwich protection (bundling your swap with protection transactions)

**Tip Program**:
- **Tip Payment Program**: `GJHtFqM9agxPmkeKjHny6qiRKrXZALvvFGiKf11QE7hy`
- **Tip Distribution Program**: `DzvGET57TAgEDxvm3ERUM4GNcsAJdqjDLCne9sdfY4wf`

**Eight Static Tip Accounts** (randomly select one per bundle):
```
ADaUMid9yfUytqMBgopwjb2DTLSokTSzL1zt6iGPaS49
Cw8CFyM9FkoMi7K7Crf6HNQqf4uEMzpKw6QNghXLvLkY
HFqU5x63VTqvQss8hp11i4wVV8bD44PvwucfZ2bU7gRe
DfXygSm4jCyNCybVYYK6DwvWqjKee8pbDmJGcLWNDXjh
96gYZGLnJYVFmbjzopPSU6QiEV5fGqZNyN9nmNhvrZU5
ADuUkR4vqLUMWXxW9gh6D6L8pMSawimctcNZ5pGwDcEt
3AVi9Tg9Uo68tJfuvoKvqKNWKkC5wPdSSdeBnizKZ6jT
DttWaMuVvTiduZRnguLF7jNxTgiMBZ1hyAumKUi...
```

**Bundle Construction Example**:
```typescript
import { SearcherClient } from 'jito-ts/dist/sdk/block-engine/searcher';

// Create bundle with tip
const bundle = new Bundle(transactions, tipLamports);

// Send to block engine
const result = await searcherClient.sendBundle(bundle);
```

### 5.4 MEV Strategies on Solana

Common MEV strategies specific to Solana DeFi:

1. **Arbitrage**: Price discrepancies between DEXes (Raydium vs. Orca vs. Phoenix)
2. **Liquidations**: Monitoring lending protocols (Kamino, MarginFi) for undercollateralized positions
3. **Sandwich Attacks**: Front-running + back-running user swaps (mitigated by Jito bundles and Jupiter's Beam)
4. **JIT (Just-In-Time) Liquidity**: Providing concentrated liquidity moments before a large swap executes
5. **Backrunning**: Executing after large trades to capture price impact

---

## 6. Solana DeFi Program Patterns

### 6.1 Vault/Pool Patterns

The vault pattern is the most fundamental DeFi pattern on Solana. It manages pooled assets under program authority.

**Standard Vault Architecture**:
```
+------------------+     +------------------+
| Vault State      |     | Token Vault      |
| (Data Account)   |     | (Token Account)  |
|                  |     |                  |
| - authority PDA  |     | - owner: PDA     |
| - total_deposits |     | - mint: USDC     |
| - share_supply   |     | - balance: 1M    |
| - config params  |     +------------------+
+------------------+

+------------------+
| LP Mint          |
| (Mint Account)   |
|                  |
| - mint_authority: |
|   vault PDA      |
| - supply: shares |
+------------------+
```

**Key Pattern -- Deposit Flow**:
```rust
pub fn deposit(ctx: Context<Deposit>, amount: u64) -> Result<()> {
    // 1. Calculate shares based on current pool ratio
    let shares = calculate_shares(amount, vault.total_deposits, vault.share_supply);

    // 2. Transfer tokens from user to vault (CPI to Token program)
    token::transfer(
        CpiContext::new(
            ctx.accounts.token_program.to_account_info(),
            Transfer {
                from: ctx.accounts.user_token_account.to_account_info(),
                to: ctx.accounts.vault_token_account.to_account_info(),
                authority: ctx.accounts.user.to_account_info(),
            },
        ),
        amount,
    )?;

    // 3. Mint LP tokens to user (CPI with PDA signer)
    token::mint_to(
        CpiContext::new_with_signer(
            ctx.accounts.token_program.to_account_info(),
            MintTo {
                mint: ctx.accounts.lp_mint.to_account_info(),
                to: ctx.accounts.user_lp_account.to_account_info(),
                authority: ctx.accounts.vault_authority.to_account_info(),
            },
            &[&[b"vault_authority", vault.key().as_ref(), &[vault.bump]]],
        ),
        shares,
    )?;

    // 4. Update vault state
    vault.total_deposits += amount;
    vault.share_supply += shares;

    Ok(())
}
```

**AMM Pool Pattern**:
```
+-------------------+
| Pool State        |
| - token_a_vault   | --> Token Account (owned by pool PDA)
| - token_b_vault   | --> Token Account (owned by pool PDA)
| - lp_mint         | --> Mint Account (authority: pool PDA)
| - fee_rate        |
| - sqrt_price      | (for CLMM)
| - tick_current    | (for CLMM)
+-------------------+
```

### 6.2 Oracle Integration (Pyth, Switchboard)

#### Pyth Network

**Pyth Solana Receiver Program**: `rec5EKMGg6MxZYaMdyBfgwp4d5rB9T1VQH5pJv5LtFJ`

Pyth is the dominant oracle on Solana DeFi, powering ~60% of derivatives protocols.

**Architecture -- Pull Model**:
- Price data is aggregated on Pythnet (an SVM appchain) every slot
- Data is NOT automatically pushed to Solana (unlike Chainlink on Ethereum)
- Users "pull" price updates to Solana when they need them
- This dramatically reduces costs (no continuous on-chain updates)

**Integration Pattern**:
```rust
use pyth_solana_receiver_sdk::price_update::PriceUpdateV2;

pub fn use_price(ctx: Context<UsePrice>) -> Result<()> {
    let price_update = &ctx.accounts.price_update;

    // Get price, ensuring it's not stale (max_age in seconds)
    let price = price_update.get_price_no_older_than(
        &Clock::get()?,
        60,  // max 60 seconds old
        &price_feed_id,
    )?;

    // price.price is the price (i64)
    // price.conf is the confidence interval
    // price.expo is the exponent (negative)
    // Actual price = price.price * 10^price.expo

    Ok(())
}
```

**First-Party Data Sources**: Pyth gets data directly from market makers and exchanges (not scraped), providing institutional-quality pricing with confidence intervals.

#### Switchboard

**Program ID**: `SBondMDrcV3K4kxZR1HNVT7osZxAHVHgYXL5Ze1oMUv`

Switchboard is a permissionless, customizable oracle network.

**Key Differentiator -- Customizable Feeds**:
- Users can create custom data feeds from multiple sources
- Can aggregate from Pyth, Chainlink, and Web2 APIs simultaneously
- TEE (Trusted Execution Environment) integration for secure data processing
- **Switchboard Surge**: Fastest oracle on Solana (latency-optimized)

**Use Cases**:
- Custom price feeds for long-tail assets
- Multi-source aggregation (Pyth + Chainlink + custom API)
- VRF (Verifiable Random Function) for on-chain randomness
- Used by Kamino Lend, MarginFi, and others as secondary/backup oracle

### 6.3 Liquidation Bot Architecture

Liquidation bots are essential infrastructure for lending protocol health on Solana.

**How Liquidation Works**:
1. Monitor all user positions on lending protocols (Kamino, MarginFi, Drift)
2. Detect positions where collateral value drops below the liquidation threshold
3. Execute liquidation transaction: assume the borrower's liabilities, receive collateral at a discount
4. Sell received collateral on a DEX for profit

**Architecture Pattern**:
```
+-----------------+     +------------------+     +-----------------+
| Price Monitor   |     | Position Scanner |     | Executor        |
| (Pyth/Swbd     |---->| (Account         |---->| (Jito Bundle    |
|  websocket)     |     |  subscription)   |     |  or priority tx)|
+-----------------+     +------------------+     +-----------------+
        |                       |                        |
   Oracle prices         Unhealthy positions        Liquidation tx
                                                   + DEX swap
                                                   = profit
```

**Implementation Considerations**:
- Subscribe to account changes via `onAccountChange` or Geyser gRPC for real-time monitoring
- Use Jito bundles for atomic liquidation (liquidate + swap in one atomic bundle)
- Account for oracle price staleness (Pyth pull model requires fresh price updates)
- Gas optimization: Pre-compute all required accounts to minimize transaction size
- Competition: Multiple bots compete; tip higher for priority

**Drift Protocol Liquidation Bot Example** (from Drift docs):
```typescript
// Drift provides keeper bot infrastructure
// Liquidation bots inherit liabilities and receive assets at a discount
const liquidatorBot = new LiquidatorBot({
    driftClient,
    userMap,
    // The liquidator inherits the liabilities they liquidate
    // and receives collateral at a discount
});
await liquidatorBot.start();
```

### 6.4 Crank/Keeper Patterns

The crank/keeper pattern is used when on-chain state needs periodic updates or order execution.

**What is a Crank?**:
- An off-chain process that submits transactions to trigger on-chain state updates
- Named after the "cranking" mechanism in early Solana DEXes (Serum)
- Modern protocols often call these "keepers" with economic incentives

**Keeper Pattern Architecture**:
```
+------------------+     +------------------+     +------------------+
| Off-Chain Keeper |     | On-Chain Program  |     | On-Chain State   |
| (TypeScript/     |---->| (Validates        |---->| (Updated by      |
|  Rust bot)       |     |  conditions)      |     |  keeper tx)      |
+------------------+     +------------------+     +------------------+
```

**Common Keeper Use Cases in DeFi**:

| Use Case | Protocol Example | What the Keeper Does |
|---|---|---|
| Order Matching | Drift DLOB | Matches limit orders with resting orders or AMM |
| Liquidation | Kamino, MarginFi | Liquidates undercollateralized positions |
| DCA Execution | Jupiter DCA | Executes scheduled recurring swaps |
| Limit Order Fill | Jupiter Limit | Fills limit orders when price is met |
| Funding Rate | Jupiter Perps | Updates funding rates for perp positions |
| Rebalancing | Meteora Vaults | Rebalances vault allocations across lending protocols |
| Oracle Updates | Pyth | Posts fresh price data to Solana |

**Economic Incentives**:
- Keepers earn fees/tips for successful execution
- Drift: Keepers earn a portion of the order's fee for filling it
- Jupiter DCA: Keepers earn a small fee per executed order
- Liquidation keepers earn the discount on collateral

**Phoenix's Innovation**: Phoenix eliminated the need for cranks entirely with its "crankless" CLOB design, where orders settle atomically within the maker/taker transaction itself.

---

## 7. Key Solana DeFi Events and History

### 7.1 The Solana DeFi Boom (2021-2022)

**Early DeFi Ecosystem**:
- Raydium launched as the first major AMM (February 2021)
- Marinade Finance launched liquid staking (August 2021)
- Serum (backed by FTX/Alameda) served as the central order book for the ecosystem
- Solend brought lending/borrowing
- TVL grew rapidly, reaching ~$10B at peak

**Key Characteristics**:
- Serum's on-chain CLOB was a core primitive -- many DEXes built on top of it
- FTX/Alameda Research was deeply integrated (backed ~20% of Solana projects)
- Fast, cheap transactions attracted DeFi innovation that was impractical on Ethereum

### 7.2 The Wormhole Hack (February 2022)

**What Happened**:
- A signature verification flaw in Wormhole's Solana-side program allowed an attacker to forge valid signatures
- Bypassed Guardian validation, enabling unauthorized minting of 120,000 wETH without depositing ETH
- ~$326 million stolen (second-largest DeFi hack at the time)

**Impact**:
- Vulnerability patched within hours (February 2, 2022)
- Jump Crypto (Wormhole's parent) reimbursed 120,000 ETH the next day
- Demonstrated the systemic risk of bridge protocols
- Led to increased scrutiny of cross-chain bridge security

**Lesson for Developers**: Always verify that the correct program signed a message. The bug was in failing to properly validate that the `SignatureSet` account was created by the Wormhole program itself.

### 7.3 The Mango Markets Exploit (October 2022)

**What Happened**:
- Avraham Eisenberg executed a market manipulation attack on Mango Markets
- Manipulated MNGO token price by taking large positions, inflating collateral value
- Borrowed (and withdrew) ~$114 million against the inflated collateral
- Not a smart contract bug -- an economic/oracle manipulation attack

**Impact**:
- Exposed vulnerabilities in oracle-dependent DeFi designs
- Led to improved oracle price feed mechanisms and circuit breakers
- Eisenberg was later arrested and prosecuted
- Partial fund recovery through negotiation

**Lesson for Developers**: Economic attacks are as dangerous as code exploits. Implement oracle guardrails (price deviation limits, TWAP vs. spot checks, confidence interval validation).

### 7.4 The FTX Collapse (November 2022)

**Impact on Solana DeFi**:
- SOL price crashed from $35 to under $14 (eventually reached ~$8)
- DeFi TVL dropped from ~$1B to ~$300M
- Serum (the core order book DEX) became unusable -- FTX controlled its upgrade authority
- ~20% of Solana projects had FTX/Alameda investments
- ~5% of ecosystem startups had funds on FTX
- Developer confidence severely shaken

**Immediate Aftermath**:
- Community forked Serum into **OpenBook** (community-controlled)
- Projects that survived demonstrated genuine product-market fit
- Solana Foundation increased grants and developer support

### 7.5 Recovery and Growth (2023-2025)

**2023: Resilient Comeback**:
- SOL price: +918.4% gain in 2023 alone
- September 2023: Visa expanded USDC settlement to Solana (institutional validation)
- 83% YoY growth in active developers by 2024
- New protocols launched without FTX dependency

**2024: DeFi Renaissance**:
- SOL price: +85.6% gain
- DeFi TVL grew 213% QoQ in Q4 2024, reaching $8.6B
- Ecosystem revenue increased >15x
- DApp revenue increased >14x
- Kamino, Jupiter, and Drift emerged as dominant protocols

**2025: Maturity and Scale**:
- DeFi TVL reached $11.5B (December 2025)
- Lending markets grew to $3.6B
- Jupiter Lend launched ($500M TVL in 24 hours)
- Drift v3 launched (10x performance improvement)
- Jito BAM introduced (block assembly marketplace overhaul)
- Meteora became top fee-generator on Solana
- Stablecoin market cap exceeded $15.5B
- 100% network uptime throughout the year
- Alpenglow consensus overhaul approved (targeting ~100ms finality)

**Key Takeaway**: The FTX collapse was existentially threatening but ultimately beneficial -- it purged unhealthy dependencies, and the projects that survived emerged stronger. Solana's DeFi ecosystem in 2025/2026 is more decentralized, battle-tested, and genuinely innovative than the pre-FTX era.

---

## 8. Key Program IDs Reference

### Core Infrastructure

| Program | Program ID | Notes |
|---|---|---|
| SPL Token Program | `TokenkegQfeZyiNwAJbNbGKPFXCWuBvf9Ss623VQ5DA` | Original token program |
| Token-2022 | `TokenzQdBNbLqP5VEhdkAS6EPFLC1PHnBqCXEpPxuEb` | Extended token program |
| Associated Token Account | `ATokenGPvbdGVxr1b2hvZbsiqW5xWH25efTNsLJA8knL` | ATA derivation |
| Stake Pool Program | `SPoo1Ku8WFXoNDMHPsrGSTSG1Y47rzgn41SLUNakuHy` | Used by Jito, others |

### DEXes

| Protocol | Program ID | Type |
|---|---|---|
| Jupiter Aggregator | `JUP6LkbZbjS1jKKwapdHNy74zcZ3tLUZoi5QNyVTaV4` | Aggregator |
| Jupiter Limit Orders | `jupoNjAxXgZ4rjzxzPMP4oxduvQsQtZzyknqvzYNrNu` | Limit orders |
| Jupiter Perpetuals | `PERPHjGBqRHArX4DySjwM6UJHiR3sWAatqfdBS2qQJu` | Perps |
| Raydium CLMM | `CAMMCzo5YL8w4VFF8KVHrK22GGUsp5VTaW7grrKgrWqK` | Concentrated liquidity |
| Raydium CPMM | `CPMMoo8L3F4NbTegBCKVNunggL7H1ZpdTHKxQB5qKP1C` | Standard AMM |
| Raydium Legacy AMM v4 | `675kPX9MHTjS2zt1qfr1NYHuzeLXfQM9H24wFSUt1Mp8` | Legacy AMM |
| Orca Whirlpool | `whirLbMiicVdio4qvUfM5KAg6Ct8VwpYzGff3uctyCc` | CLMM |
| Phoenix | `PhoeNiXZ8ByJGLkxNfZRnkUfjvmuYqLR89jjFHGqdXY` | Order book |
| Meteora DLMM | `LBUZKhRxPF3XUpBCjp4YzTKgLccjZhTSDM9YuVaPwxo` | Dynamic liquidity |

### Lending

| Protocol | Program ID | Notes |
|---|---|---|
| Kamino Lend (K-Lend) | `KLend2g3cP87fffoy8q1mQqGKjrxjC8boSyAYavgmjD` | Largest Solana lender |
| MarginFi v2 | `MFv2hWf31Z9kbCa1snEPYctwafyhdvnV7FZnsebVacA` | Global + isolated markets |

### Liquid Staking

| Protocol | Key Address | Type |
|---|---|---|
| Marinade | `MarBmsSgKXdrN1egZf5sqe1TMai9K1rChYNDJgjq7aD` | Program ID |
| mSOL Mint | `mSoLzYCxHdYgdzU16g5QSh3i5K3z3KZK7ytfqcJm7So` | Token mint |
| JitoSOL Mint | `J1toso1uCk3RLmjorhTtrVwY9HJ7X8V9yYac6Y7kGCPn` | Token mint |
| Jito Stake Pool | `Jito4APyf642JPZPx3hGc6WWJ8zPKtRbRs4P815Awbb` | Pool account |

### Perpetuals / Derivatives

| Protocol | Program ID | Notes |
|---|---|---|
| Jupiter Perps | `PERPHjGBqRHArX4DySjwM6UJHiR3sWAatqfdBS2qQJu` | LP-based perps |
| JLP Token | `27G8MtK7VtTcCHkpASjSDdkWWYfoqT6ggEuKidVJidD4` | JLP mint |
| Drift | `dRiftyHA39MWEi3m9aunc5MzRF1JYuBsbn6VPcn33UH` | Hybrid perps |

### Oracles

| Protocol | Program ID | Notes |
|---|---|---|
| Pyth Solana Receiver | `rec5EKMGg6MxZYaMdyBfgwp4d5rB9T1VQH5pJv5LtFJ` | Pull oracle |
| Switchboard | `SBondMDrcV3K4kxZR1HNVT7osZxAHVHgYXL5Ze1oMUv` | Customizable oracle |

### Jito MEV Infrastructure

| Component | Address |
|---|---|
| Tip Payment Program | `GJHtFqM9agxPmkeKjHny6qiRKrXZALvvFGiKf11QE7hy` |
| Tip Distribution Program | `DzvGET57TAgEDxvm3ERUM4GNcsAJdqjDLCne9sdfY4wf` |

### Key Token Mints

| Token | Mint Address |
|---|---|
| USDC (Native) | `EPjFWdd5AufqSSqeM2qN1xzybapC8G4wEGGkZwyTDt1v` |
| USDT | `Es9vMFrzaCERmJfrF4H2FYD4KCoNkY11McCe8BenwNYB` |
| mSOL | `mSoLzYCxHdYgdzU16g5QSh3i5K3z3KZK7ytfqcJm7So` |
| JitoSOL | `J1toso1uCk3RLmjorhTtrVwY9HJ7X8V9yYac6Y7kGCPn` |
| MNDE | `MNDEFzGvMt87ueuHvVU9VcTqsAP5b3fTGPsHuuPA5ey` |
| JLP | `27G8MtK7VtTcCHkpASjSDdkWWYfoqT6ggEuKidVJidD4` |

> **Note**: Program IDs are subject to change with protocol upgrades. Always verify against official documentation before production use.

---

## 9. Security Considerations

### 9.1 Common Solana DeFi Vulnerabilities

**Account Validation Failures**:
- Solana allows any account to be passed to a program function
- Without strict validation, attackers can inject unexpected or malicious accounts
- Always verify: account ownership, account data matches expectations, PDA derivation is correct

**Arbitrary CPI**:
- Programs invoking other programs without verifying the target program's identity
- An attacker could substitute a malicious program with the same interface
- Always validate the callee's program ID

**PDA Sharing**:
- Reusing PDAs across different contexts without sufficient seed differentiation
- Example: Using only `mint` as a seed for a vault PDA allows multiple pools to control the same vault
- Fix: Include pool ID, withdraw destination, or other unique identifiers in seeds

**Oracle Manipulation**:
- Relying on spot prices without confidence interval checks
- Not enforcing staleness bounds on oracle data
- Not using TWAP for critical operations (liquidations, large trades)

**Missing Signer Checks**:
- Failing to verify that required authorities have signed the transaction
- Always check `is_signer` on authority accounts

### 9.2 Security Best Practices

1. **Validate every account**: Check ownership, derivation, data contents
2. **Use Anchor constraints**: `#[account(has_one = authority)]`, `#[account(seeds = [...], bump)]`
3. **Verify CPI targets**: Ensure the program ID of CPI callees matches expected addresses
4. **Oracle safety**: Check staleness, confidence intervals, use TWAP for critical operations
5. **Audit thoroughly**: Engage reputable Solana audit firms
6. **Formal verification**: Where possible, especially for core DeFi logic
7. **Rate limits and circuit breakers**: Protect against economic attacks

### 9.3 Leading Solana Audit Firms

- **OtterSec**: Comprehensive code reviews, vulnerability assessments, formal verification. Dominant in Solana ecosystem
- **Halborn**: Standard of excellence for Solana, large client portfolio
- **Neodyme**: Deep Solana expertise, known for Token-2022 security research
- **Zellic**: Focus on Solana and Move-based chains
- **Cantina**: Open marketplace model for security researchers
- **Trail of Bits**: Broader scope but strong Solana practice

### 9.4 Total Historical Losses

Total recorded losses from Solana-based exploits (2020-April 2025) exceeded $530M, overwhelmingly from DeFi application exploits. Core protocol/network issues, while disruptive (outages), did not directly steal user funds. By late 2023 and 2024, the time between incidents increased significantly, indicating a maturing security posture.

---

## 10. Sources and References

### Official Documentation

- [Solana Developer Docs](https://solana.com/docs)
- [Jupiter Developer Docs](https://dev.jup.ag/get-started/development-basics)
- [Raydium Docs - Addresses](https://docs.raydium.io/raydium/for-developers/program-addresses)
- [Orca Whirlpools SDK](https://github.com/orca-so/whirlpools)
- [Phoenix V1 (GitHub)](https://github.com/Ellipsis-Labs/phoenix-v1)
- [Drift Protocol V2 (GitHub)](https://github.com/drift-labs/protocol-v2)
- [Kamino Finance - K-Lend (GitHub)](https://github.com/Kamino-Finance/klend)
- [MarginFi V2 (GitHub)](https://github.com/mrgnlabs/marginfi-v2)
- [Marinade Liquid Staking (GitHub)](https://github.com/marinade-finance/liquid-staking-program)
- [Meteora Documentation](https://docs.meteora.ag)
- [Jito MEV Documentation](https://jito-foundation.gitbook.io/mev/)
- [Jito Foundation Docs](https://docs.jito.wtf/)
- [Pyth Developer Hub](https://docs.pyth.network/price-feeds/core/contract-addresses/solana)
- [Switchboard Documentation](https://docs.switchboard.xyz/docs-by-chain/solana-svm)
- [SPL Token Program](https://spl.solana.com/token)
- [Token-2022 Program](https://spl.solana.com/token-2022)
- [Solana CPI Documentation](https://solana.com/docs/core/cpi)
- [Save (formerly Solend) Docs](https://docs.save.finance)
- [Lifinity Documentation](https://docs.lifinity.io/)

### Research and Analysis

- [Solana DeFi Deep Dives: Jupiter Ultra V3 (Medium/@Scoper)](https://medium.com/@Scoper/solana-defi-deep-dives-jupiter-ultra-v3-next-gen-dex-aggregator-late-2025-2cef75c97301)
- [Solana DeFi Deep Dives: Kamino (Medium/@Scoper)](https://medium.com/@Scoper/solana-defi-deep-dives-kamino-late-2025-080f6f52fa29)
- [Solana Lending Markets Report 2025 (RedStone)](https://blog.redstone.finance/2025/12/11/solana-lending-markets/)
- [Inside Drift: Architecting a High-Performance Orderbook (Medium)](https://extremelysunnyyk.medium.com/inside-drift-architecting-a-high-performance-orderbook-on-solana-612a98b8ac17)
- [Jito Bundling and MEV Optimization Strategies (Medium)](https://medium.com/@gwrx2005/jito-bundling-and-mev-optimization-strategies-on-solana-an-economic-analysis-c035b6885e1f)
- [How Jito-Solana Works: A Deep Dive](https://thogiti.github.io/2025/01/01/How-Jito-Solana-Works.html)
- [Solana MEV: An Introduction (Helius)](https://www.helius.dev/blog/solana-mev-an-introduction)
- [Solana MEV Economics (QuickNode)](https://blog.quicknode.com/solana-mev-economics-jito-bundles-liquid-staking-guide/)
- [Solana Hacks, Bugs, and Exploits: A Complete History (Helius)](https://www.helius.dev/blog/solana-hacks)
- [Solana Liquid Staking: Everything You Need to Know (Nansen)](https://www.nansen.ai/post/solana-liquid-staking-everything-you-need-to-know-in-2025)
- [Sanctum Infinity Guide](https://blog.sanctum.so/inf-guide)
- [EVM vs. SVM Comparison (Coin Bureau)](https://coinbureau.com/technology/evm-vs-svm)
- [Sealevel: Parallel Processing (Solana)](https://solana.com/news/sealevel---parallel-processing-thousands-of-smart-contracts)
- [Securing Solana: A Developer's Guide (Cantina)](https://cantina.xyz/blog/securing-solana-a-developers-guide)
- [A Hitchhiker's Guide to Solana Program Security (Helius)](https://www.helius.dev/blog/a-hitchhikers-guide-to-solana-program-security)
- [Token-2022 Security (Neodyme)](https://neodyme.io/en/blog/token-2022/)
- [Solana vs Ethereum L2s: 2026 Fundamental Analysis (MEXC)](https://www.mexc.com/learn/article/solana-vs-ethereum-l2s-2026-fundamental-analysis-tvl-revenue-stablecoin-metrics/1)
- [Solana Recovery After the FTX Collapse (Fystack)](https://fystack.io/blog/solana-recovery-after-the-ftx-collapse-2025-guide-for-web3-builders)

### Data and Rankings

- [DeFiLlama - Solana TVL](https://defillama.com/chains)
- [DappRadar - Solana DeFi Rankings](https://dappradar.com/narratives/defi/protocols/chain/solana)
- [CoinLaw - Solana Statistics 2025](https://coinlaw.io/solana-statistics/)
- [Solana Compass - Projects](https://solanacompass.com/projects)

### Guides and Tutorials

- [Jito Bundles Guide (QuickNode)](https://www.quicknode.com/guides/solana-development/transactions/jito-bundles)
- [Pyth Price Feeds on Solana (QuickNode)](https://www.quicknode.com/guides/solana-development/3rd-party-integrations/pyth-price-feeds)
- [Token-2022 Transfer Fees (Solana Docs)](https://solana.com/developers/guides/token-extensions/transfer-fee)
- [Confidential Transfers (Solana Docs)](https://solana.com/docs/tokens/extensions/confidential-transfer)
- [Drift Liquidation Bot Tutorial](https://docs.drift.trade/tutorial-bots/keeper-bots/tutorial-liquidation-bot)
- [PDA Sharing Security (Solana)](https://solana.com/developers/courses/program-security/pda-sharing)
- [CPI with PDA Signer (Solana)](https://solana.com/developers/guides/getstarted/how-to-cpi-with-signer)
- [Mastering CPI in Anchor (Medium)](https://medium.com/@ancilartech/mastering-cross-program-invocations-in-anchor-a-developers-guide-to-solana-s-cpi-patterns-0f29a5734a3e)
- [Solana Development for EVM Developers (QuickNode)](https://www.quicknode.com/guides/solana-development/getting-started/solana-development-for-evm-developers)

---

*This document was compiled from extensive web research conducted in February 2026. Protocol details, TVL figures, and program IDs are current as of the research date but should be verified against official sources before use in production. The DeFi ecosystem evolves rapidly -- always check official documentation for the latest information.*
