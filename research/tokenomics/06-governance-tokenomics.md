# Governance Tokenomics

> Written for experienced Solana developers entering token design.
> Last updated: February 2026

---

## Table of Contents

1. [What Is Token Governance?](#1-what-is-token-governance)
2. [Governance Models](#2-governance-models)
   - [2.1 Token-Weighted Voting](#21-token-weighted-voting)
   - [2.2 Quadratic Voting](#22-quadratic-voting)
   - [2.3 Conviction Voting](#23-conviction-voting)
   - [2.4 Optimistic Governance](#24-optimistic-governance)
   - [2.5 Futarchy](#25-futarchy)
3. [Delegation](#3-delegation)
4. [On-Chain vs. Off-Chain Governance](#4-on-chain-vs-off-chain-governance)
5. [The Voter Apathy Problem](#5-the-voter-apathy-problem)
6. [Governance Attacks](#6-governance-attacks)
7. [Progressive Decentralization](#7-progressive-decentralization)
8. [DAO Treasuries and Spending](#8-dao-treasuries-and-spending)
9. [Governance Case Studies](#9-governance-case-studies)
10. [Program-Level Implementation on Solana](#10-program-level-implementation-on-solana)
11. [References](#11-references)

---

## 1. What Is Token Governance?

Token governance is the process by which token holders collectively make decisions about a protocol's parameters, upgrades, treasury, and future direction. It replaces centralized corporate governance (board of directors, CEO, shareholders) with decentralized, permissionless decision-making.

### What Governance Controls

| Control Domain | Examples | Sensitivity |
|---|---|---|
| **Protocol parameters** | Fee rates, collateral factors, interest models | Low-medium |
| **Asset listings** | Which tokens to support, oracle sources | Medium |
| **Treasury spending** | Grants, team compensation, partnerships | High |
| **Protocol upgrades** | Smart contract changes, new features | Very high |
| **Emergency actions** | Pausing the protocol, draining funds | Critical |
| **Tokenomics changes** | Supply cap, emission schedule, burn rate | Critical |

### The Governance Spectrum

```
Fully Centralized          Multi-sig           Token Governance         Immutable
◄────────────────────────────────────────────────────────────────────────────────►
Team controls everything    3/5 signers         Token holders vote       No changes
                            can execute          on everything            possible
Fast but trusting           Moderate trust       Slow but trustless       Maximum trust
                                                                          but inflexible
```

Most protocols sit somewhere in the middle, using a combination of token governance for major decisions and multi-sig or core team authority for day-to-day operations.

---

## 2. Governance Models

### 2.1 Token-Weighted Voting

The simplest and most common model: 1 token = 1 vote.

```
Proposal: "Increase protocol fee from 0.3% to 0.5%"

Vote results:
  FOR:     2,500,000 tokens (62.5%)
  AGAINST: 1,500,000 tokens (37.5%)
  Quorum:  4,000,000 tokens (above 2,000,000 minimum)

  Result: PASSED ✓
```

**Advantages**:
- Simple to understand and implement
- Directly aligns voting power with economic stake
- Standard across DeFi

**Problems**:
- **Plutocratic**: Whales dominate governance. A single entity with 10% of supply has more voting power than 10,000 small holders combined.
- **Low participation**: Most token holders never vote. Typical participation rates: 5-15% of supply.
- **Short-term bias**: Large holders might vote for their immediate benefit rather than long-term protocol health.

### 2.2 Quadratic Voting

Voting power scales with the square root of tokens held, not linearly.

```
Token-weighted:                 Quadratic:
  100 tokens = 100 votes         100 tokens = 10 votes (√100)
  10,000 tokens = 10,000 votes   10,000 tokens = 100 votes (√10,000)

Ratio:
  Token-weighted: Whale has 100x more power
  Quadratic:      Whale has 10x more power
```

**Advantages**:
- More egalitarian than token-weighted
- Reduces whale dominance
- Better representation of preference intensity

**Problems**:
- **Sybil-vulnerable**: Create 100 wallets with 100 tokens each instead of 1 wallet with 10,000 tokens. Cost: same. Quadratic voting power: 100 × 10 = 1,000 vs. 100. A 10x advantage from sybiling.
- Requires identity/sybil resistance (which is hard in crypto)
- Gitcoin uses this with passport verification to mitigate sybil attacks

### 2.3 Conviction Voting

Voting power accumulates over time. The longer you vote for a proposal, the more weight your vote carries.

```
Conviction mechanics:
  Day 1: You allocate 1000 tokens to a proposal → conviction = 100
  Day 2: Still allocated → conviction = 190
  Day 3: Still allocated → conviction = 271
  ...
  Day 30: conviction = 958 (approaching maximum of 1000)

  Conviction follows exponential approach:
  conviction(t) = tokens × (1 - α^t) / (1 - α)
  where α = decay factor (e.g., 0.9)
```

**Advantages**:
- Rewards sustained commitment, not flash voting
- Prevents last-minute vote manipulation
- Proposals that maintain support over time are stronger
- No need for discrete voting periods (continuous governance)

**Problems**:
- Complex to understand
- Slow — urgent decisions can't pass quickly
- Early voters have disproportionate conviction advantage

**Used by**: 1Hive (Gardens), various community DAOs

### 2.4 Optimistic Governance

Default: proposals pass automatically unless vetoed.

```
Optimistic governance flow:
  1. Core team proposes action
  2. Proposal enters a "challenge period" (e.g., 3 days)
  3. If no one objects with sufficient voting power → passes automatically
  4. If challenged → falls back to full token vote

  Only controversial proposals require full votes
  Routine operations proceed without voting overhead
```

**Advantages**:
- Fast for uncontroversial decisions
- Reduces governance fatigue
- Core team can operate efficiently while maintaining accountability

**Problems**:
- Requires active monitoring (someone must notice bad proposals)
- Asymmetric: easier to pass proposals than to block them
- "Silent approval" can be gamed if community is not paying attention

**Used by**: Optimism (for some decisions), Nouns DAO (fork mechanism)

### 2.5 Futarchy

"Vote on values, bet on beliefs." Governance by prediction markets.

```
Futarchy concept:
  Question: "Should we increase the fee to 0.5%?"

  Create two prediction markets:
    Market A: "Token price if fee = 0.5%" → trades at $52
    Market B: "Token price if fee = 0.3% (status quo)" → trades at $48

  Decision: Go with Market A (higher predicted value)
  Rationale: If the market predicts higher token value with the change,
             the change is likely beneficial
```

**Advantages**:
- Harnesses collective intelligence of markets
- Financial incentive for accurate predictions (bad predictors lose money)
- Removes emotional/political bias from governance

**Problems**:
- Extremely complex to implement
- Low liquidity in prediction markets → unreliable signals
- Manipulation risk (buy up one market to force a governance outcome)
- Token price is not the only thing that matters

**Status**: Mostly theoretical. MetaDAO on Solana is one of the few implementations attempting futarchy in practice.

---

## 3. Delegation

### Why Delegation Matters

Most token holders lack the time, expertise, or interest to evaluate every proposal. Delegation allows them to assign their voting power to someone else.

```
Without delegation:
  - 5% of token holders vote
  - Proposals pass with <10% of supply participating
  - "Governance" is controlled by a few whales

With delegation:
  - Token holders delegate to experts
  - Delegates vote on their behalf
  - 30-50% of supply can be represented
  - More informed voting decisions
```

### Delegation Models

**Liquid delegation** (most common):
```
1. You delegate your tokens to a delegate
2. Delegate votes on your behalf
3. You can un-delegate at any time
4. Your tokens remain in your wallet (not transferred)
```

**Fixed-term delegation** (ve-model):
```
1. Lock tokens for a fixed period
2. Voting power assigned for the lock duration
3. Cannot un-delegate until lock expires
4. Stronger commitment but less flexibility
```

### The Delegate Landscape

```
Delegate types:
  1. Protocol core team: Deep knowledge, potential conflict of interest
  2. Venture funds: Large allocations, may prioritize returns
  3. Community members: Aligned with users, limited bandwidth
  4. Professional delegates: Full-time governance participation
  5. DAOs/protocols: Vote as a collective entity
```

**Delegate accountability**:
- Public voting records (on-chain, transparent)
- Delegate platforms with profiles and voting rationale
- Community can un-delegate if dissatisfied
- Some protocols reward active delegates

---

## 4. On-Chain vs. Off-Chain Governance

### On-Chain Governance

Votes are cast as on-chain transactions. The result is binding and automatically executed.

```
On-chain flow:
  1. Create proposal (on-chain transaction)
  2. Voting period opens (e.g., 3 days)
  3. Token holders submit vote transactions
  4. Proposal reaches quorum → automatically executed
  5. Smart contract state changes without human intervention

Implementation: Solana SPL Governance (Realms), Compound Governor
```

**Advantages**:
- Trustless execution (no one can block an approved proposal)
- Transparent (all votes are on-chain)
- Binding (the vote IS the execution)

**Disadvantages**:
- Gas costs for every vote (significant on Ethereum, minimal on Solana)
- Slow (proposals take days to weeks)
- Cannot handle nuanced decisions (binary yes/no)
- Governance attacks are harder to reverse

### Off-Chain Governance

Votes are cast off-chain (e.g., Snapshot), and execution is handled by a multi-sig or core team.

```
Off-chain flow:
  1. Post proposal on governance forum (Discourse, Commonwealth)
  2. Discussion period (1-2 weeks)
  3. Snapshot vote (gasless, off-chain)
  4. If passed: multi-sig executes the decision
  5. Multi-sig is trusted to follow the vote result

Implementation: Snapshot + Gnosis Safe multi-sig
```

**Advantages**:
- Free voting (no gas costs)
- Flexible (can handle complex decisions with multiple options)
- Discussion-first (encourages deliberation before voting)
- Faster iteration (lower barrier to propose)

**Disadvantages**:
- Not binding (multi-sig could ignore the vote — requires trust)
- Centralized execution (the multi-sig is a trust point)
- Sybil risk (off-chain voting is harder to verify)

### Hybrid Models

Most mature protocols use both:

```
Uniswap governance:
  Temperature check: Off-chain (Snapshot) — is there interest?
  Consensus check: Off-chain (Snapshot) — is there majority support?
  On-chain vote:    On-chain (Governor Bravo) — binding execution

MakerDAO governance:
  Forum discussion → Polling vote (off-chain) → Executive vote (on-chain)
```

---

## 5. The Voter Apathy Problem

### Scale of the Problem

```
Governance participation rates:
  Uniswap:  ~5% of supply votes on average proposal
  Compound: ~10% of supply
  AAVE:     ~5% of supply
  MakerDAO: ~10-15% of supply

  Even "active" governance communities struggle to reach 20% participation
```

### Why Token Holders Don't Vote

| Reason | % of Non-Voters | Solution |
|---|---|---|
| **Unaware** of proposals | ~40% | Better notification systems |
| **Don't understand** the proposal | ~25% | Delegate system, plain-language summaries |
| **Gas costs** too high | ~15% | Off-chain voting (Snapshot), L2, Solana |
| **Don't think their vote matters** | ~15% | Quadratic voting, delegation rewards |
| **Intentionally abstaining** | ~5% | - |

### Incentivizing Participation

| Approach | Mechanism | Example |
|---|---|---|
| **Vote mining** | Earn tokens for voting | Some DAOs reward voters |
| **Delegation rewards** | Delegates earn compensation | Optimism RPGF for delegates |
| **Voting = access** | Must vote to unlock features | - |
| **Conviction decay** | Non-voters lose voting power | Conviction voting models |
| **Quorum reduction** | Accept lower participation | Adjust quorum to realistic levels |

---

## 6. Governance Attacks

### Flash Loan Governance Attack

```
Attack vector:
  1. Flash borrow massive amount of governance tokens
  2. Create malicious proposal (drain treasury, change parameters)
  3. Vote with borrowed tokens → pass proposal
  4. Execute proposal → extract value
  5. Repay flash loan

  Total cost: flash loan fee (~0.09%)
  Potential profit: entire treasury
```

**Defenses**:
- **Snapshot voting power at proposal creation**: Your voting power is based on your holdings at the block the proposal was created, not the current block. Flash-borrowed tokens don't count.
- **Time delay between proposal and execution**: Even if a malicious proposal passes, there is a timelock (e.g., 48 hours) before execution, allowing the community to respond.
- **Minimum proposal threshold**: Must hold X% of tokens to create a proposal (prevents spam).

### Bribery Attack

```
Attack vector:
  1. Identify a proposal that benefits the attacker
  2. Bribe token holders to vote in favor
  3. Bribe cost < value extracted from the protocol

  Example:
    Protocol treasury: $100M
    Proposal: "Grant $10M to [attacker's address]"
    Cost to bribe 51% of voting power: $2M
    Profit: $10M - $2M = $8M
```

**Defenses**:
- High quorum requirements (harder to bribe enough voters)
- Timelock + emergency veto (council can veto malicious proposals)
- ve-model (locked tokens are harder to bribe — holders have skin in the game)

### Governance Capture

```
Long-term capture:
  A single entity (VC, protocol, whale) gradually accumulates enough tokens
  to control governance permanently.

  Symptoms:
  - Same entity wins every vote
  - Proposals that benefit one entity consistently pass
  - Community proposals consistently fail
```

**Defenses**:
- ve-model with decay (must keep re-locking to maintain power)
- Maximum voting cap per address (soft defense, sybilable)
- Separation of powers (different voting bodies for different decisions)
- Immutable core parameters (some things can't be changed by governance)

### The Beanstalk Governance Attack ($182M, April 2022)

```
What happened:
  1. Attacker borrowed $1B in stablecoins via flash loan
  2. Deposited into Beanstalk to get Stalk (governance tokens)
  3. Created a malicious proposal (BIP-18)
  4. Voted for the proposal with flash-loaned governance power
  5. Proposal passed immediately (no timelock!)
  6. Proposal executed: drained $182M from Beanstalk
  7. Repaid flash loan, kept $80M profit

Root cause:
  - No snapshot voting (current balance used for votes)
  - No timelock between vote and execution
  - Emergency execution allowed instant passage
```

**Lessons**:
- ALWAYS snapshot voting power at proposal creation block
- ALWAYS have a timelock between vote passage and execution
- NEVER allow single-transaction governance (create + vote + execute)

---

## 7. Progressive Decentralization

### The Three Phases

Most successful protocols follow a progressive decentralization path:

**Phase 1: Centralized (Launch)**
```
Control: Core team via multi-sig
Governance: None
Token: May not exist yet
Duration: 6-18 months

Rationale: Fast iteration, bug fixes, parameter tuning
The protocol needs to evolve rapidly and can't wait for governance votes
```

**Phase 2: Limited Governance (Growth)**
```
Control: Multi-sig + token governance for major decisions
Governance: On-chain or Snapshot for key parameters
Token: Launched, distributing to community
Duration: 12-24 months

Rationale: Community involvement in direction-setting
Team retains ability to act quickly for emergencies
```

**Phase 3: Full Governance (Maturity)**
```
Control: Fully on-chain governance, team is one of many stakeholders
Governance: All protocol changes require token vote
Token: Widely distributed, active delegation
Duration: Ongoing

Rationale: Protocol is stable, well-understood
Community has sufficient expertise and tooling
```

### Examples

```
MakerDAO:
  2017: Rune Christensen + team control
  2018-2019: MKR governance for collateral types and risk parameters
  2020-2023: SubDAO structure, delegated governance
  2024-2025: "Endgame" plan — AI-governed SubDAOs

Uniswap:
  2018-2020: No governance (team controlled)
  Sept 2020: UNI launch → governance enabled
  2021-2023: Governance forums, Snapshot, Governor Bravo
  2024+: Active governance with delegates, fee discussions

Jupiter:
  2022-2023: Team controlled
  Jan 2024: JUP launch, DAO governance
  2024-2025: Community votes on ASM (Active Supply Management)
  Community voted to burn 3B tokens
```

---

## 8. DAO Treasuries and Spending

### Treasury Sizes

```
Notable DAO treasuries (approximate, Feb 2026):
  Uniswap (UNI):     $3-4B
  Optimism (OP):      $2-3B
  Arbitrum (ARB):     $3-4B
  MakerDAO (MKR):     $2-3B
  Lido (LDO):         $200-500M
  Jupiter (JUP):      $1-2B
```

### The Treasury Spending Problem

Most DAO treasuries are massive but poorly deployed:

```
Common failure modes:
  1. "Spray and pray" grants: Small grants to many projects with no follow-up
  2. Treasury hoarding: Treasury grows but nothing is spent
  3. Insider enrichment: Grants disproportionately go to team-affiliated entities
  4. Lack of accountability: No metrics for grant success
  5. Governance theater: Proposals pass but nothing happens
```

### Effective Treasury Deployment

| Strategy | Description | Example |
|---|---|---|
| **Focused grants** | Large grants with milestones and accountability | Optimism RPGF (Retroactive Public Goods Funding) |
| **Protocol-owned liquidity** | Use treasury to provide permanent liquidity | OlympusDAO bonding (concept) |
| **Revenue diversification** | Convert native token to stablecoins | MakerDAO treasury diversification |
| **Bug bounties** | Reward security researchers | Immunefi bounties funded by treasury |
| **Development funding** | Core team compensation via governance | Uniswap Foundation grants |
| **Strategic investments** | Invest in complementary protocols | Aave treasury investments |

### Retroactive Public Goods Funding (RPGF)

Optimism pioneered RPGF — funding projects AFTER they've proven value, rather than speculatively funding proposals:

```
RPGF Process:
  1. Projects build and ship (no upfront funding)
  2. Periodically, governance reviews what shipped
  3. Projects that created value receive retroactive funding
  4. Incentivizes building first, applying second

  Advantage: No wasted grants on projects that never ship
  Disadvantage: Builders need alternative funding to survive until RPGF
```

---

## 9. Governance Case Studies

### Case Study 1: MakerDAO — The Most Battle-Tested DAO

```
Governance scope:
  - Collateral types (which assets back DAI)
  - Risk parameters (LTV, liquidation penalty, stability fees)
  - Interest rates for different vault types
  - Treasury allocation and protocol strategy
  - MKR burn vs. accumulation policy

Notable decisions:
  - 2020: Added USDC as collateral (controversial — centralized asset)
  - 2022: Invested in real-world assets (US treasuries)
  - 2023: "Endgame" restructuring into SubDAOs
  - 2024-2025: Rebranded to Sky/USDS, then partially reverted

Governance structure:
  - MKR token voting (on-chain)
  - Governance facilitators (paid delegates)
  - Risk teams (evaluate collateral)
  - SubDAOs for specific domains
```

**Lessons**: MakerDAO demonstrates both the power and limitations of token governance. It has successfully governed a $5B+ stablecoin for years, but governance fatigue, complexity, and political infighting have been persistent challenges. The "Endgame" restructuring attempts to address scaling governance through SubDAOs.

### Case Study 2: Uniswap — Governance Gridlock

```
Key issue: Fee switch debate (2022-2026)

  For the fee switch:
    - UNI holders earn protocol revenue
    - Creates fundamental value for UNI
    - Protocol generates $500M+ in annual fees

  Against the fee switch:
    - May classify UNI as a security (regulatory risk)
    - Reduces LP incentives (LPs earn less)
    - Uniswap Labs already charges interface fee

  Result: Years of discussion with no activation
  Multiple proposals, forum debates, research reports
  Meanwhile, Uniswap Labs earns interface fees directly

  Lesson: Governance can be too slow for contentious decisions
  The ability to NOT decide is itself a decision
```

### Case Study 3: Compound — Governor Bravo Template

```
Compound Governor Bravo became the standard governance contract:

  Proposal threshold: Must hold 100,000 COMP to propose (1% of supply)
  Voting period: 3 days
  Timelock: 2 days after passage
  Quorum: 400,000 COMP (4% of supply)

  This pattern was copied by dozens of protocols.

Governance incident (September 2021):
  - Proposal 62 passed with a bug
  - Mistakenly distributed $80M in extra COMP rewards
  - Community could see the bug but couldn't fix it until next proposal
  - Timelock prevented rapid response
  - Eventual recovery through social pressure + governance fix

  Lesson: Timelocks are critical for security but can delay critical fixes
  Consider guardian/pause mechanisms for emergencies
```

### Case Study 4: Jupiter DAO — Active Supply Management

```
Innovation: Community directly controls token supply

  January 2025 vote: Burn 3B JUP (30% of total supply)
    - 95% voted in favor
    - Largest token burn in DeFi history
    - FDV decreased by ~30%
    - Per-token value increased (same market cap, fewer tokens)

  The vote demonstrated:
  1. Community can make bold economic decisions
  2. Token holders will act against dilution
  3. Governance can be a feature, not just overhead

  Ongoing: Community votes on quarterly token management
```

---

## 10. Program-Level Implementation on Solana

### SPL Governance (Realms)

Solana's native governance framework. Used by most Solana DAOs.

```
Realms architecture:
  Realm PDA          → Top-level DAO configuration
  Governance PDA     → Governs a specific target (program, mint, token account)
  Proposal PDA       → A specific governance proposal
  VoteRecord PDA     → Individual vote on a proposal
  TokenOwnerRecord   → User's governance token deposit + delegation

Flow:
  1. User deposits governance tokens into Realm
  2. Creates a proposal on a specific Governance
  3. Voting period opens
  4. Token holders vote (for/against)
  5. If quorum met and majority for: proposal passes
  6. After cool-off period: anyone can execute the proposal
  7. Execution performs the on-chain transaction
```

### Governance Program (Simplified Anchor Implementation)

```rust
use anchor_lang::prelude::*;

#[account]
pub struct GovernanceConfig {
    pub token_mint: Pubkey,
    pub min_proposal_threshold: u64,    // Tokens needed to propose
    pub quorum: u64,                     // Tokens needed for valid vote
    pub voting_period: i64,              // Seconds
    pub execution_delay: i64,            // Timelock (seconds after vote)
    pub proposal_count: u64,
    pub admin: Pubkey,                   // Emergency guardian
    pub bump: u8,
}

#[account]
pub struct Proposal {
    pub id: u64,
    pub proposer: Pubkey,
    pub description_hash: [u8; 32],      // IPFS hash of proposal description
    pub instruction_data: Vec<u8>,       // The instruction to execute if passed
    pub target_program: Pubkey,          // Program to invoke
    pub for_votes: u64,
    pub against_votes: u64,
    pub created_at: i64,
    pub voting_ends_at: i64,
    pub executed: bool,
    pub cancelled: bool,
}

impl Proposal {
    pub fn status(&self, current_time: i64, config: &GovernanceConfig) -> ProposalStatus {
        if self.cancelled {
            return ProposalStatus::Cancelled;
        }
        if self.executed {
            return ProposalStatus::Executed;
        }
        if current_time < self.voting_ends_at {
            return ProposalStatus::Voting;
        }
        if self.for_votes + self.against_votes < config.quorum {
            return ProposalStatus::Defeated; // Quorum not met
        }
        if self.for_votes <= self.against_votes {
            return ProposalStatus::Defeated;
        }
        let execution_time = self.voting_ends_at + config.execution_delay;
        if current_time < execution_time {
            return ProposalStatus::Queued; // In timelock
        }
        ProposalStatus::ReadyToExecute
    }
}

#[derive(AnchorSerialize, AnchorDeserialize, Clone, PartialEq)]
pub enum ProposalStatus {
    Voting,
    Defeated,
    Queued,
    ReadyToExecute,
    Executed,
    Cancelled,
}
```

### Snapshot Voting Power (Anti-Flash Loan)

```rust
/// Record token balance at proposal creation for snapshot voting
#[account]
pub struct VoteSnapshot {
    pub proposal_id: u64,
    pub voter: Pubkey,
    pub token_balance: u64,     // Balance at proposal creation time
    pub delegated_balance: u64, // Delegated tokens at snapshot
    pub snapshot_slot: u64,     // Slot when snapshot was taken
    pub has_voted: bool,
}

/// Create a snapshot of a voter's balance (must be before voting starts)
pub fn create_snapshot(ctx: Context<CreateSnapshot>) -> Result<()> {
    let proposal = &ctx.accounts.proposal;
    let snapshot = &mut ctx.accounts.vote_snapshot;

    // Snapshot must be from proposal creation time
    // This prevents flash loan attacks
    let token_account = &ctx.accounts.voter_token_account;

    snapshot.proposal_id = proposal.id;
    snapshot.voter = ctx.accounts.voter.key();
    snapshot.token_balance = token_account.amount;
    snapshot.delegated_balance = get_delegated_balance(
        &ctx.accounts.voter.key(),
        proposal.created_at,
    )?;
    snapshot.snapshot_slot = Clock::get()?.slot;
    snapshot.has_voted = false;

    Ok(())
}
```

### Delegation System

```rust
#[account]
pub struct Delegation {
    pub delegator: Pubkey,
    pub delegate: Pubkey,
    pub token_mint: Pubkey,
    pub amount: u64,
    pub created_at: i64,
}

/// Delegate voting power to another address
pub fn delegate(ctx: Context<Delegate>, amount: u64) -> Result<()> {
    let delegation = &mut ctx.accounts.delegation;
    let clock = Clock::get()?;

    // Verify delegator has sufficient tokens
    let delegator_balance = ctx.accounts.delegator_token_account.amount;
    let existing_delegations = get_total_delegated(&ctx.accounts.delegator.key())?;
    require!(
        delegator_balance >= existing_delegations + amount,
        GovError::InsufficientBalance
    );

    delegation.delegator = ctx.accounts.delegator.key();
    delegation.delegate = ctx.accounts.delegate.key();
    delegation.token_mint = ctx.accounts.token_mint.key();
    delegation.amount = amount;
    delegation.created_at = clock.unix_timestamp;

    Ok(())
}

/// Un-delegate (instant, no delay)
pub fn undelegate(ctx: Context<Undelegate>) -> Result<()> {
    // Close the delegation account, returning rent to delegator
    // Voting power returns to the delegator immediately

    Ok(())
}
```

### Timelock Execution

```rust
/// Execute a passed proposal after timelock expires
pub fn execute_proposal(ctx: Context<ExecuteProposal>) -> Result<()> {
    let proposal = &mut ctx.accounts.proposal;
    let config = &ctx.accounts.governance_config;
    let clock = Clock::get()?;

    // Verify proposal is ready to execute
    let status = proposal.status(clock.unix_timestamp, config);
    require!(
        status == ProposalStatus::ReadyToExecute,
        GovError::ProposalNotReady
    );

    // Execute the proposal's instruction
    let ix = Instruction {
        program_id: proposal.target_program,
        accounts: deserialize_accounts(&proposal.instruction_data)?,
        data: deserialize_ix_data(&proposal.instruction_data)?,
    };

    // Execute with governance PDA as signer
    let governance_seeds = &[
        b"governance",
        config.token_mint.as_ref(),
        &[config.bump],
    ];

    anchor_lang::solana_program::program::invoke_signed(
        &ix,
        &ctx.remaining_accounts.iter()
            .map(|a| a.to_account_info())
            .collect::<Vec<_>>(),
        &[governance_seeds],
    )?;

    proposal.executed = true;

    emit!(ProposalExecuted {
        proposal_id: proposal.id,
        executor: ctx.accounts.executor.key(),
        timestamp: clock.unix_timestamp,
    });

    Ok(())
}
```

---

## 11. References

1. **Compound Governor Bravo**: The most forked governance contract
2. **SPL Governance (Realms) documentation**: Solana-native governance framework
3. **"Moving Beyond Coin Voting Governance" — Vitalik Buterin (2021)**: Critique of token-weighted voting
4. **Snapshot documentation**: Off-chain voting platform
5. **"Notes on Blockchain Governance" — Vlad Zamfir (2017)**: Early governance thinking
6. **Beanstalk Governance Attack post-mortem**: Flash loan governance exploit
7. **Optimism Governance documentation**: RPGF and optimistic governance
8. **"Governance Minimization" — Fred Ehrsam (Paradigm, 2020)**: The case for minimal governance
9. **MetaDAO documentation**: Futarchy implementation on Solana
10. **Jupiter Governance docs**: ASM and community-governed tokenomics

---

*Next: [07 - Tokenomics at the Program Level](./07-tokenomics-program-level.md) — SPL tokens, mint/freeze authority, Token-2022 extensions, vesting contracts, staking programs, and building tokenomics infrastructure on Solana.*
