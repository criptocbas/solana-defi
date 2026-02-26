# Tokenomics at the Program Level

> Written for experienced Solana developers entering token design.
> Last updated: February 2026

---

## Table of Contents

1. [SPL Token Fundamentals](#1-spl-token-fundamentals)
2. [Mint Authority and Supply Control](#2-mint-authority-and-supply-control)
3. [Token-2022 Extensions for Tokenomics](#3-token-2022-extensions-for-tokenomics)
   - [3.1 Transfer Fees](#31-transfer-fees)
   - [3.2 Interest-Bearing Tokens](#32-interest-bearing-tokens)
   - [3.3 Non-Transferable (Soulbound) Tokens](#33-non-transferable-soulbound-tokens)
   - [3.4 Permanent Delegate](#34-permanent-delegate)
   - [3.5 Confidential Transfers](#35-confidential-transfers)
   - [3.6 Transfer Hooks](#36-transfer-hooks)
4. [Building a Token Emission Program](#4-building-a-token-emission-program)
5. [Vesting Programs](#5-vesting-programs)
6. [Staking Programs](#6-staking-programs)
7. [Airdrop Distribution Programs](#7-airdrop-distribution-programs)
8. [Buyback and Burn Programs](#8-buyback-and-burn-programs)
9. [Governance Integration](#9-governance-integration)
10. [Security Considerations](#10-security-considerations)
11. [Testing Token Programs](#11-testing-token-programs)
12. [References](#12-references)

---

## 1. SPL Token Fundamentals

### The SPL Token Standard

On Solana, all fungible tokens use the SPL Token program (or Token-2022). Understanding the data model is essential for building tokenomics programs.

### Core Accounts

```
Mint Account (82 bytes)
├── mint_authority: Option<Pubkey>   // Who can create new tokens
├── supply: u64                      // Total tokens in existence
├── decimals: u8                     // Token decimal places (usually 6 or 9)
├── is_initialized: bool
└── freeze_authority: Option<Pubkey> // Who can freeze token accounts

Token Account (165 bytes)
├── mint: Pubkey           // Which mint this account holds
├── owner: Pubkey          // Who controls this account
├── amount: u64            // Balance
├── delegate: Option<Pubkey>
├── state: AccountState    // Initialized, Frozen, etc.
├── is_native: Option<u64>
├── delegated_amount: u64
└── close_authority: Option<Pubkey>
```

### Authority Model

The SPL Token program has two authority types that are critical for tokenomics:

**Mint Authority**: Controls who can create (mint) new tokens. This is the most important authority for supply mechanics.

```
Mint authority patterns:
  1. EOA (Externally Owned Account) → Team/admin can mint at will
  2. Multisig → Multiple signers required to mint
  3. PDA → Program controls minting logic
  4. None → Revoked, no one can ever mint again (fixed supply)
```

**Freeze Authority**: Controls who can freeze token accounts (prevent transfers).

```
Freeze authority use cases:
  - Regulatory compliance (freeze accounts for legal reasons)
  - Emergency response (freeze during exploit)
  - Token recovery (freeze stolen tokens)

  For most DeFi tokens: Freeze authority is revoked
  For regulated/compliance tokens: Freeze authority is retained
```

### Associated Token Accounts (ATAs)

Each wallet has one ATA per token mint, derived deterministically:

```
ATA address = PDA([wallet_pubkey, token_program_id, mint_pubkey])

This means:
  - Every wallet has a predictable token account address
  - No need to create accounts in advance (init_if_needed)
  - Programs can compute account addresses without on-chain lookup
```

---

## 2. Mint Authority and Supply Control

### Pattern: Emission-Controlled Mint

The most common tokenomics pattern — a program holds mint authority and enforces emission rules:

```rust
use anchor_lang::prelude::*;
use anchor_spl::token::{self, Mint, MintTo, Token};

#[account]
pub struct EmissionController {
    pub token_mint: Pubkey,
    pub authority_bump: u8,
    pub initial_rate_per_second: u64,  // Tokens/sec at epoch 0
    pub decay_rate_bps: u16,           // e.g., 1500 = 15% decay per epoch
    pub epoch_duration: i64,           // Seconds per epoch
    pub start_time: i64,
    pub last_emit_time: i64,
    pub total_emitted: u64,
    pub max_supply: u64,               // 0 = no cap
}

impl EmissionController {
    pub const SPACE: usize = 8 + 32 + 1 + 8 + 2 + 8 + 8 + 8 + 8 + 8;

    pub fn current_rate(&self, now: i64) -> u64 {
        let elapsed = (now - self.start_time).max(0);
        let epochs = (elapsed / self.epoch_duration) as u32;

        let mut rate = self.initial_rate_per_second as u128;
        let decay = 10000u128 - self.decay_rate_bps as u128;

        for _ in 0..epochs.min(100) {
            rate = rate * decay / 10000;
        }
        rate as u64
    }

    pub fn pending_tokens(&self, now: i64) -> u64 {
        let elapsed = (now - self.last_emit_time).max(0) as u128;
        let rate = self.current_rate(now) as u128;
        let pending = (rate * elapsed) as u64;

        if self.max_supply > 0 {
            let remaining = self.max_supply.saturating_sub(self.total_emitted);
            pending.min(remaining)
        } else {
            pending
        }
    }
}

#[derive(Accounts)]
pub struct EmitTokens<'info> {
    #[account(mut)]
    pub emission_controller: Account<'info, EmissionController>,

    #[account(
        mut,
        address = emission_controller.token_mint,
    )]
    pub token_mint: Account<'info, Mint>,

    /// CHECK: PDA that holds mint authority
    #[account(
        seeds = [b"emission_authority", token_mint.key().as_ref()],
        bump = emission_controller.authority_bump,
    )]
    pub emission_authority: UncheckedAccount<'info>,

    /// The recipient of emitted tokens
    #[account(mut)]
    pub recipient: Account<'info, anchor_spl::token::TokenAccount>,

    pub token_program: Program<'info, Token>,
}

pub fn emit_tokens(ctx: Context<EmitTokens>) -> Result<()> {
    let clock = Clock::get()?;
    let controller = &mut ctx.accounts.emission_controller;

    let amount = controller.pending_tokens(clock.unix_timestamp);
    require!(amount > 0, TokenError::NothingToEmit);

    controller.last_emit_time = clock.unix_timestamp;
    controller.total_emitted = controller
        .total_emitted
        .checked_add(amount)
        .ok_or(TokenError::MathOverflow)?;

    // Mint via PDA signer
    let mint_key = ctx.accounts.token_mint.key();
    let seeds = &[
        b"emission_authority",
        mint_key.as_ref(),
        &[controller.authority_bump],
    ];

    token::mint_to(
        CpiContext::new_with_signer(
            ctx.accounts.token_program.to_account_info(),
            MintTo {
                mint: ctx.accounts.token_mint.to_account_info(),
                to: ctx.accounts.recipient.to_account_info(),
                authority: ctx.accounts.emission_authority.to_account_info(),
            },
            &[seeds],
        ),
        amount,
    )?;

    Ok(())
}
```

### Pattern: Revoked Authority (Immutable Supply)

```rust
/// After initial mint, revoke authority permanently
pub fn revoke_mint_authority(ctx: Context<RevokeMintAuthority>) -> Result<()> {
    // Set mint authority to None
    token::set_authority(
        CpiContext::new(
            ctx.accounts.token_program.to_account_info(),
            token::SetAuthority {
                account_or_mint: ctx.accounts.token_mint.to_account_info(),
                current_authority: ctx.accounts.current_authority.to_account_info(),
            },
        ),
        token::spl_token::instruction::AuthorityType::MintTokens,
        None, // None = permanently revoked
    )?;

    // Now no one can ever mint more tokens
    // Supply is permanently fixed at current level
    Ok(())
}
```

---

## 3. Token-2022 Extensions for Tokenomics

Token-2022 (also called Token Extensions) is Solana's next-generation token program with built-in extensions that enable complex tokenomics without custom programs.

### 3.1 Transfer Fees

Automatically collect a fee on every token transfer. This is the most impactful extension for tokenomics.

```
Transfer Fee mechanics:
  User transfers 1000 tokens
  Transfer fee: 100 bps (1%)
  Recipient receives: 990 tokens
  Withheld (fee): 10 tokens

  Withheld tokens accumulate in each token account
  Protocol can harvest withheld tokens and burn/distribute them
```

**Use cases**:
- Automatic protocol revenue on every transfer
- Deflationary mechanics (harvest fees → burn)
- Revenue sharing (harvest fees → distribute to stakers)

```rust
// Creating a Token-2022 mint with transfer fee
use spl_token_2022::extension::transfer_fee::TransferFeeConfig;

// At mint initialization:
// transfer_fee_basis_points: 100  (1%)
// maximum_fee: u64::MAX           (no cap)

// Harvesting withheld fees (anyone can call):
// spl_token_2022::instruction::harvest_withheld_tokens_to_mint()
// This moves all withheld fees from token accounts to the mint

// Withdrawing from mint (authority only):
// spl_token_2022::instruction::withdraw_withheld_tokens_from_mint()
// Moves collected fees to the protocol's token account
```

**Real-world example**: A protocol token with 0.5% transfer fee. Every trade, transfer, or DeFi interaction automatically generates revenue. At $10M daily volume, that's $50K/day or $18.25M/year in protocol revenue — with zero additional infrastructure.

### 3.2 Interest-Bearing Tokens

Display a continuously growing balance without minting new tokens. The balance shown is a cosmetic multiplier — actual supply doesn't change.

```
Interest-bearing mechanics:
  Initial deposit: 100 tokens
  Rate: 5% APY
  After 1 year: UI shows 105 tokens

  But actual on-chain balance is still 100 tokens
  The display is computed: balance × (1 + rate × elapsed)
```

**Use case**: Staking receipt tokens that show accrued rewards without requiring claim transactions. Similar to how aUSDC works on Aave.

### 3.3 Non-Transferable (Soulbound) Tokens

Tokens that cannot be transferred between accounts. Once minted to an address, they stay there.

```
Soulbound token use cases:
  - Governance reputation (earned, not bought)
  - KYC verification (proof of identity, non-sellable)
  - Achievement badges (non-tradeable accomplishments)
  - Voting power (cannot be flash-loaned or borrowed)
```

**Tokenomics application**: Combine soulbound tokens with governance. Voting power is earned through protocol participation, not purchased on the market. This prevents plutocracy and flash loan governance attacks.

### 3.4 Permanent Delegate

A designated authority can transfer or burn tokens from ANY holder's account.

```
Permanent delegate use cases:
  - Regulatory clawback (freeze and seize tokens for compliance)
  - Auto-burn (protocol can burn tokens from fee accounts)
  - Subscription model (protocol can deduct tokens for services)
```

**Warning**: This is extremely powerful and centralized. Most DeFi tokens should NOT use permanent delegate, as it undermines the trust-minimization ethos. It is primarily for regulated/compliance tokens.

### 3.5 Confidential Transfers

Encrypt transfer amounts so they are not publicly visible on-chain.

```
Confidential transfer:
  Public info: sender address, recipient address
  Hidden info: transfer amount

  Uses zero-knowledge proofs (ZK proofs) to verify:
  - Sender has sufficient balance
  - No tokens are created or destroyed
  - All without revealing the actual amount
```

**Tokenomics application**: Privacy-preserving token distributions, salary payments, treasury operations without revealing amounts to competitors.

### 3.6 Transfer Hooks

Custom program logic that executes on every token transfer.

```
Transfer Hook flow:
  1. User initiates token transfer
  2. Token-2022 program calls the transfer hook program
  3. Hook program executes custom logic
  4. If hook succeeds → transfer completes
  5. If hook fails → transfer reverts

Use cases:
  - Custom transfer restrictions (KYC whitelist)
  - Tax-on-transfer (flexible fee logic beyond flat rate)
  - Transfer logging/analytics
  - Cooldown periods between transfers
  - Maximum transfer limits
```

```rust
// Transfer hook program entry point
pub fn execute_transfer_hook(
    ctx: Context<TransferHook>,
    amount: u64,
) -> Result<()> {
    // Example: Enforce 24-hour cooldown between transfers
    let last_transfer = ctx.accounts.transfer_state.last_transfer_time;
    let clock = Clock::get()?;
    let elapsed = clock.unix_timestamp - last_transfer;

    require!(elapsed >= 86400, HookError::CooldownNotElapsed);

    // Example: Maximum transfer of 1% of total supply per transaction
    let max_transfer = ctx.accounts.token_mint.supply / 100;
    require!(amount <= max_transfer, HookError::ExceedsMaxTransfer);

    // Update state
    ctx.accounts.transfer_state.last_transfer_time = clock.unix_timestamp;

    Ok(())
}
```

---

## 4. Building a Token Emission Program

### Complete Emission System Architecture

```
                    ┌───────────────┐
                    │ Emission       │
                    │ Controller PDA │
                    │ (config/state) │
                    └───────┬───────┘
                            │
              ┌─────────────┼─────────────┐
              │             │             │
         ┌────▼────┐  ┌────▼────┐  ┌────▼────┐
         │ Staking  │  │ LP      │  │ Treasury│
         │ Rewards  │  │ Rewards │  │ Grants  │
         │ (40%)    │  │ (40%)   │  │ (20%)   │
         └─────────┘  └─────────┘  └─────────┘
```

### Multi-Recipient Emission

```rust
#[account]
pub struct EmissionSchedule {
    pub token_mint: Pubkey,
    pub total_rate_per_second: u64,
    pub recipients: Vec<EmissionRecipient>,
    pub last_emit_time: i64,
    pub authority_bump: u8,
}

#[derive(AnchorSerialize, AnchorDeserialize, Clone)]
pub struct EmissionRecipient {
    pub recipient: Pubkey,
    pub share_bps: u16,        // Basis points of total emission
    pub total_received: u64,
}

pub fn distribute_emissions(ctx: Context<DistributeEmissions>) -> Result<()> {
    let clock = Clock::get()?;
    let schedule = &mut ctx.accounts.emission_schedule;

    let elapsed = (clock.unix_timestamp - schedule.last_emit_time) as u128;
    let total_amount = (schedule.total_rate_per_second as u128 * elapsed) as u64;

    require!(total_amount > 0, EmissionError::NothingToEmit);
    schedule.last_emit_time = clock.unix_timestamp;

    let mint_key = ctx.accounts.token_mint.key();
    let seeds = &[
        b"emission_authority",
        mint_key.as_ref(),
        &[schedule.authority_bump],
    ];

    // Distribute to each recipient based on their share
    for (i, recipient) in schedule.recipients.iter_mut().enumerate() {
        let share = (total_amount as u128 * recipient.share_bps as u128 / 10000) as u64;
        if share == 0 { continue; }

        recipient.total_received = recipient
            .total_received
            .checked_add(share)
            .ok_or(EmissionError::MathOverflow)?;

        // Mint to recipient's token account
        token::mint_to(
            CpiContext::new_with_signer(
                ctx.accounts.token_program.to_account_info(),
                MintTo {
                    mint: ctx.accounts.token_mint.to_account_info(),
                    to: ctx.remaining_accounts[i].to_account_info(),
                    authority: ctx.accounts.emission_authority.to_account_info(),
                },
                &[seeds],
            ),
            share,
        )?;
    }

    Ok(())
}
```

---

## 5. Vesting Programs

### Linear Vesting with Cliff

```rust
#[account]
pub struct VestingAccount {
    pub beneficiary: Pubkey,
    pub mint: Pubkey,
    pub total_amount: u64,
    pub withdrawn: u64,
    pub start_time: i64,
    pub cliff_time: i64,        // cliff_time = start_time + cliff_duration
    pub end_time: i64,          // end_time = start_time + total_vesting_duration
    pub revocable: bool,        // Can admin revoke unvested tokens?
    pub revoked: bool,
    pub vault_bump: u8,
}

impl VestingAccount {
    /// Amount vested at a given timestamp
    pub fn vested_at(&self, time: i64) -> u64 {
        if self.revoked {
            return self.withdrawn; // Only already-withdrawn amount
        }
        if time < self.cliff_time {
            return 0;
        }
        if time >= self.end_time {
            return self.total_amount;
        }

        // Linear vesting from start to end
        let elapsed = (time - self.start_time) as u128;
        let total_duration = (self.end_time - self.start_time) as u128;

        ((self.total_amount as u128) * elapsed / total_duration) as u64
    }

    /// Amount available to withdraw now
    pub fn withdrawable_at(&self, time: i64) -> u64 {
        self.vested_at(time).saturating_sub(self.withdrawn)
    }
}
```

### Revocation (for employee/advisor vesting)

```rust
/// Admin revokes unvested tokens (e.g., employee leaves)
pub fn revoke_vesting(ctx: Context<RevokeVesting>) -> Result<()> {
    let vesting = &mut ctx.accounts.vesting_account;
    require!(vesting.revocable, VestingError::NotRevocable);
    require!(!vesting.revoked, VestingError::AlreadyRevoked);

    let clock = Clock::get()?;
    let vested = vesting.vested_at(clock.unix_timestamp);
    let unvested = vesting.total_amount - vested;

    vesting.revoked = true;

    // Transfer unvested tokens back to admin/treasury
    if unvested > 0 {
        // ... token transfer CPI from vault to treasury ...
    }

    // Beneficiary can still claim their vested portion
    Ok(())
}
```

---

## 6. Staking Programs

### Staking Pool with Reward Distribution

```rust
/// Synthetix-style reward distribution
/// reward_per_token = cumulative rewards per staked token (scaled 1e18)
#[account]
pub struct StakingPool {
    pub stake_mint: Pubkey,
    pub reward_mint: Pubkey,
    pub total_staked: u64,
    pub reward_per_token_stored: u128,   // Scaled by PRECISION
    pub last_update_time: i64,
    pub reward_rate: u64,                // Rewards per second
    pub reward_end_time: i64,
    pub pool_authority_bump: u8,
}

const PRECISION: u128 = 1_000_000_000_000_000_000; // 1e18

#[account]
pub struct StakerInfo {
    pub owner: Pubkey,
    pub staked_amount: u64,
    pub reward_per_token_paid: u128,
    pub pending_rewards: u64,
}

impl StakingPool {
    /// Update reward_per_token to current time
    pub fn update_reward_per_token(&mut self, now: i64) {
        if self.total_staked == 0 {
            self.last_update_time = now;
            return;
        }

        let applicable_time = now.min(self.reward_end_time);
        let elapsed = (applicable_time - self.last_update_time).max(0) as u128;

        self.reward_per_token_stored += elapsed
            * (self.reward_rate as u128)
            * PRECISION
            / (self.total_staked as u128);

        self.last_update_time = now;
    }
}

impl StakerInfo {
    /// Calculate pending rewards for this staker
    pub fn earned(&self, pool: &StakingPool) -> u64 {
        let rpt_delta = pool.reward_per_token_stored - self.reward_per_token_paid;
        let earned = (self.staked_amount as u128) * rpt_delta / PRECISION;
        self.pending_rewards + (earned as u64)
    }

    /// Update staker's checkpoint to current pool state
    pub fn update_rewards(&mut self, pool: &StakingPool) {
        self.pending_rewards = self.earned(pool);
        self.reward_per_token_paid = pool.reward_per_token_stored;
    }
}

pub fn stake(ctx: Context<Stake>, amount: u64) -> Result<()> {
    let clock = Clock::get()?;
    let pool = &mut ctx.accounts.staking_pool;
    let staker = &mut ctx.accounts.staker_info;

    // Update global reward state
    pool.update_reward_per_token(clock.unix_timestamp);

    // Checkpoint staker's rewards before changing their stake
    staker.update_rewards(pool);

    // Transfer tokens to pool
    token::transfer(
        CpiContext::new(
            ctx.accounts.token_program.to_account_info(),
            token::Transfer {
                from: ctx.accounts.staker_token_account.to_account_info(),
                to: ctx.accounts.pool_vault.to_account_info(),
                authority: ctx.accounts.staker.to_account_info(),
            },
        ),
        amount,
    )?;

    // Update state
    staker.staked_amount += amount;
    pool.total_staked += amount;

    Ok(())
}

pub fn claim_rewards(ctx: Context<ClaimRewards>) -> Result<()> {
    let clock = Clock::get()?;
    let pool = &mut ctx.accounts.staking_pool;
    let staker = &mut ctx.accounts.staker_info;

    pool.update_reward_per_token(clock.unix_timestamp);
    staker.update_rewards(pool);

    let rewards = staker.pending_rewards;
    require!(rewards > 0, StakingError::NoRewardsToClaim);

    staker.pending_rewards = 0;

    // Transfer rewards from pool's reward vault to staker
    // ... CPI transfer with PDA signer ...

    Ok(())
}
```

---

## 7. Airdrop Distribution Programs

### Merkle Distributor (Production-Ready)

For large airdrops (100K+ addresses), a Merkle tree is the standard approach:

```rust
use anchor_lang::solana_program::keccak;

#[account]
pub struct MerkleDistributor {
    pub authority: Pubkey,
    pub token_mint: Pubkey,
    pub merkle_root: [u8; 32],
    pub max_total_claim: u64,
    pub total_amount_claimed: u64,
    pub num_claims: u64,
    pub vault_bump: u8,
}

#[account]
pub struct ClaimStatus {
    pub is_claimed: bool,
    pub claimant: Pubkey,
    pub claimed_at: i64,
    pub amount: u64,
}

pub fn claim(
    ctx: Context<Claim>,
    index: u64,
    amount: u64,
    proof: Vec<[u8; 32]>,
) -> Result<()> {
    let distributor = &mut ctx.accounts.distributor;
    let claim_status = &mut ctx.accounts.claim_status;

    require!(!claim_status.is_claimed, AirdropError::AlreadyClaimed);

    // Construct leaf: hash(index, claimant, amount)
    let leaf = keccak::hashv(&[
        &index.to_le_bytes(),
        &ctx.accounts.claimant.key().to_bytes(),
        &amount.to_le_bytes(),
    ]);

    // Verify Merkle proof
    let mut current = leaf.0;
    for sibling in proof.iter() {
        if current <= *sibling {
            current = keccak::hashv(&[&current, sibling]).0;
        } else {
            current = keccak::hashv(&[sibling, &current]).0;
        }
    }

    require!(
        current == distributor.merkle_root,
        AirdropError::InvalidMerkleProof
    );

    // Record claim
    claim_status.is_claimed = true;
    claim_status.claimant = ctx.accounts.claimant.key();
    claim_status.claimed_at = Clock::get()?.unix_timestamp;
    claim_status.amount = amount;

    distributor.total_amount_claimed += amount;
    distributor.num_claims += 1;

    // Transfer tokens from vault to claimant
    let mint_key = distributor.token_mint;
    let seeds = &[
        b"vault",
        mint_key.as_ref(),
        &[distributor.vault_bump],
    ];

    token::transfer(
        CpiContext::new_with_signer(
            ctx.accounts.token_program.to_account_info(),
            token::Transfer {
                from: ctx.accounts.vault.to_account_info(),
                to: ctx.accounts.claimant_token_account.to_account_info(),
                authority: ctx.accounts.vault_authority.to_account_info(),
            },
            &[seeds],
        ),
        amount,
    )?;

    Ok(())
}
```

### Building the Merkle Tree (Off-Chain)

```typescript
import { keccak_256 } from "@noble/hashes/sha3";
import { MerkleTree } from "merkletreejs";

// Airdrop recipients
const claims = [
  { address: "Abc123...", amount: 1000_000000n }, // 1000 tokens (6 decimals)
  { address: "Def456...", amount: 5000_000000n },
  // ... 100,000+ entries
];

// Build leaves
const leaves = claims.map((claim, index) => {
  const data = Buffer.concat([
    Buffer.from(new BigUint64Array([BigInt(index)]).buffer),
    Buffer.from(bs58.decode(claim.address)),
    Buffer.from(new BigUint64Array([claim.amount]).buffer),
  ]);
  return keccak_256(data);
});

// Build tree
const tree = new MerkleTree(leaves, keccak_256, { sort: true });
const root = tree.getRoot();

// Generate proof for a specific claim
const proof = tree.getProof(leaves[0]).map((p) => p.data);
```

---

## 8. Buyback and Burn Programs

### Automated Buyback via Jupiter DCA

```rust
/// Protocol buyback program that uses accumulated USDC revenue
/// to buy and burn the protocol token
#[account]
pub struct BuybackConfig {
    pub protocol_token_mint: Pubkey,
    pub revenue_mint: Pubkey,          // USDC
    pub revenue_vault: Pubkey,
    pub burn_vault: Pubkey,            // Intermediate vault before burn
    pub total_revenue_spent: u64,
    pub total_tokens_burned: u64,
    pub admin: Pubkey,
    pub authority_bump: u8,
}

pub fn execute_buyback(
    ctx: Context<ExecuteBuyback>,
    revenue_amount: u64,
    min_tokens_out: u64,
) -> Result<()> {
    let config = &mut ctx.accounts.buyback_config;

    require!(revenue_amount > 0, BuybackError::ZeroAmount);
    require!(
        ctx.accounts.revenue_vault.amount >= revenue_amount,
        BuybackError::InsufficientRevenue
    );

    // Step 1: Swap revenue (USDC) for protocol tokens via DEX
    // In production, this would be a CPI to Jupiter or Raydium
    // For simplicity, showing the post-swap logic:

    let tokens_received = execute_swap(
        &ctx.accounts.revenue_vault,
        &ctx.accounts.burn_vault,
        revenue_amount,
        min_tokens_out,
    )?;

    // Step 2: Burn the received tokens
    let mint_key = config.protocol_token_mint;
    let seeds = &[
        b"buyback_authority",
        mint_key.as_ref(),
        &[config.authority_bump],
    ];

    token::burn(
        CpiContext::new_with_signer(
            ctx.accounts.token_program.to_account_info(),
            token::Burn {
                mint: ctx.accounts.protocol_token_mint.to_account_info(),
                from: ctx.accounts.burn_vault.to_account_info(),
                authority: ctx.accounts.buyback_authority.to_account_info(),
            },
            &[seeds],
        ),
        tokens_received,
    )?;

    // Update stats
    config.total_revenue_spent += revenue_amount;
    config.total_tokens_burned += tokens_received;

    emit!(BuybackExecuted {
        revenue_spent: revenue_amount,
        tokens_burned: tokens_received,
        timestamp: Clock::get()?.unix_timestamp,
    });

    Ok(())
}
```

---

## 9. Governance Integration

### Program Upgrade via Governance

```rust
/// The governance PDA is the upgrade authority of the program
/// Proposals can trigger program upgrades

// In program deployment:
// solana program deploy --upgrade-authority <governance_pda>

// To upgrade, governance must invoke:
// BPFLoaderUpgradeable::upgrade(program_id, buffer_id, governance_pda)
```

### Parameter Changes via Governance

```rust
/// Governance-controlled parameters
#[account]
pub struct ProtocolConfig {
    pub governance: Pubkey,     // Only governance PDA can modify
    pub fee_rate_bps: u16,
    pub max_supply: u64,
    pub emission_rate: u64,
    pub paused: bool,
}

/// Only callable by governance PDA
pub fn update_config(
    ctx: Context<UpdateConfig>,
    new_fee_rate: Option<u16>,
    new_emission_rate: Option<u64>,
) -> Result<()> {
    // Verify caller is governance
    require!(
        ctx.accounts.authority.key() == ctx.accounts.config.governance,
        ConfigError::Unauthorized
    );

    let config = &mut ctx.accounts.config;

    if let Some(fee) = new_fee_rate {
        require!(fee <= 10000, ConfigError::InvalidFee);
        config.fee_rate_bps = fee;
    }

    if let Some(rate) = new_emission_rate {
        config.emission_rate = rate;
    }

    Ok(())
}
```

---

## 10. Security Considerations

### Common Token Program Vulnerabilities

| Vulnerability | Description | Defense |
|---|---|---|
| **Unauthorized minting** | Someone other than intended authority mints tokens | Verify mint authority is PDA, check constraints |
| **Missing authority checks** | Instruction doesn't verify signer is authorized | `has_one = admin` or explicit checks |
| **Integer overflow** | Large amounts overflow u64 | Use `checked_add`, `checked_mul` |
| **Reentrancy via CPI** | Callback during CPI re-enters program | Check state before CPI, use reentrancy guards |
| **Account substitution** | Wrong account passed for a token operation | Verify mint, owner, and address constraints |
| **Stale oracle** | Using outdated price for token operations | Check oracle freshness timestamps |
| **Flash loan manipulation** | Borrow tokens, manipulate state, repay | Snapshot-based voting, time delays |

### Anchor Security Checklist for Token Programs

```rust
// ✅ GOOD: Verify mint authority is the expected PDA
#[account(
    mut,
    mint::authority = emission_authority,
)]
pub token_mint: Account<'info, Mint>,

// ❌ BAD: No verification of mint authority
#[account(mut)]
pub token_mint: Account<'info, Mint>,

// ✅ GOOD: Verify token account mint matches
#[account(
    mut,
    token::mint = expected_mint,
    token::authority = expected_owner,
)]
pub token_account: Account<'info, TokenAccount>,

// ✅ GOOD: Use checked arithmetic
let new_supply = current_supply
    .checked_add(amount)
    .ok_or(TokenError::MathOverflow)?;

// ❌ BAD: Unchecked arithmetic
let new_supply = current_supply + amount; // Can overflow!
```

---

## 11. Testing Token Programs

### LiteSVM Test Pattern

```rust
#[cfg(test)]
mod tests {
    use litesvm::LiteSVM;
    use solana_sdk::{
        signature::{Keypair, Signer},
        transaction::Transaction,
    };

    #[test]
    fn test_emission_schedule() {
        let mut svm = LiteSVM::new();

        // Deploy token program and emission program
        svm.add_program_from_file(spl_token::id(), "spl_token.so");
        svm.add_program_from_file(emission_program::id(), "emission.so");

        let admin = Keypair::new();
        svm.airdrop(&admin.pubkey(), 10_000_000_000).unwrap();

        // Create mint
        let mint = Keypair::new();
        // ... create mint transaction ...

        // Initialize emission controller
        // ... init emission transaction ...

        // Warp time forward
        let clock = svm.get_sysvar::<Clock>();
        svm.warp_to_slot(clock.slot + 100); // ~50 seconds

        // Emit tokens
        // ... emit transaction ...

        // Verify correct amount emitted
        let account = svm.get_account(&recipient_ata).unwrap();
        let token_account = TokenAccount::unpack(&account.data).unwrap();

        let expected = emission_rate * elapsed_seconds;
        assert_eq!(token_account.amount, expected);
    }

    #[test]
    fn test_vesting_schedule() {
        // Test cliff behavior: no tokens before cliff
        // Test linear vesting: correct amounts after cliff
        // Test full vesting: all tokens available after end_time
        // Test revocation: only vested amount remains
    }

    #[test]
    fn test_staking_rewards() {
        // Test: stake → warp → claim → verify rewards
        // Test: multiple stakers get proportional rewards
        // Test: unstake partially → rewards still accrue on remainder
    }

    #[test]
    fn test_merkle_airdrop() {
        // Build Merkle tree off-chain
        // Claim with valid proof → succeeds
        // Claim with invalid proof → fails
        // Double claim → fails
        // Claim wrong amount → fails
    }
}
```

---

## 12. References

1. **SPL Token program source**: [github.com/solana-labs/solana-program-library](https://github.com/solana-labs/solana-program-library)
2. **Token-2022 specification**: Transfer fees, hooks, extensions
3. **Anchor framework**: Account constraints, PDA signers, CPI
4. **Jito Merkle Distributor**: Production airdrop implementation on Solana
5. **Synthetix StakingRewards**: The original reward distribution algorithm
6. **Realms/SPL Governance**: Solana-native governance framework
7. **LiteSVM documentation**: Testing Solana programs
8. **"Common Solana Program Vulnerabilities" — Neodyme**: Security audit patterns

---

*Next: [08 - Tokenomics Case Studies](./08-tokenomics-case-studies.md) — Deep analysis of successful and failed tokenomics models: ETH, SOL, BTC, UNI, CRV, AAVE, MKR, JUP, JTO, LINK, and cautionary tales.*
