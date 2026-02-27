use anchor_lang::prelude::*;
use anchor_spl::token::{self, Mint, MintTo, Token, TokenAccount};

use crate::constants::*;
use crate::errors::KusdError;
use crate::instructions::common::accrue_vault_fees;
use crate::math;
use crate::state::{CdpPosition, CdpVault, MockOracle};

#[derive(Accounts)]
pub struct MintKusd<'info> {
    #[account(mut)]
    pub borrower: Signer<'info>,

    #[account(
        mut,
        seeds = [CDP_VAULT_SEED, vault.collateral_mint.as_ref()],
        bump = vault.vault_bump,
    )]
    pub vault: Account<'info, CdpVault>,

    /// CHECK: PDA signing authority
    #[account(
        seeds = [CDP_VAULT_AUTHORITY_SEED, vault.key().as_ref()],
        bump = vault.authority_bump,
    )]
    pub vault_authority: UncheckedAccount<'info>,

    #[account(
        mut,
        seeds = [CDP_POSITION_SEED, vault.key().as_ref(), borrower.key().as_ref()],
        bump = position.bump,
        has_one = vault,
        has_one = owner,
    )]
    pub position: Account<'info, CdpPosition>,

    /// CHECK: position owner, must match borrower
    #[account(constraint = owner.key() == borrower.key())]
    pub owner: AccountInfo<'info>,

    #[account(
        mut,
        seeds = [KUSD_MINT_SEED, vault.key().as_ref()],
        bump = vault.kusd_mint_bump,
    )]
    pub kusd_mint: Account<'info, Mint>,

    /// Borrower's kUSD token account (receives minted kUSD)
    #[account(mut)]
    pub borrower_kusd: Account<'info, TokenAccount>,

    #[account(
        constraint = oracle.key() == vault.oracle @ KusdError::InvalidOracle,
    )]
    pub oracle: Account<'info, MockOracle>,

    pub token_program: Program<'info, Token>,
}

pub fn handle_mint_kusd(ctx: Context<MintKusd>, amount: u64) -> Result<()> {
    require!(amount > 0, KusdError::ZeroMint);

    let vault = &mut ctx.accounts.vault;
    require!(!vault.halted, KusdError::VaultHalted);

    // Accrue fees
    let clock = Clock::get()?;
    accrue_vault_fees(vault, clock.unix_timestamp)?;

    // Check oracle staleness
    let oracle = &ctx.accounts.oracle;
    let staleness = clock.unix_timestamp.saturating_sub(oracle.timestamp) as u64;
    require!(staleness <= vault.oracle_max_staleness, KusdError::OracleStale);

    // Compute current debt from shares
    let position = &ctx.accounts.position;
    let current_debt = if position.debt_shares > 0 {
        math::shares_to_debt(position.debt_shares, vault.cumulative_fee_index)?
    } else {
        0
    };

    // Compute collateral USD value
    let coll_usd = math::collateral_value_usd(
        position.collateral_amount,
        oracle.price,
        oracle.decimals,
    )?;

    // Since kUSD = $1 with 6 decimals and PRICE_SCALE = 1e6: debt_usd = debt_amount
    let new_debt = (current_debt as u128)
        .checked_add(amount as u128)
        .ok_or(KusdError::MathOverflow)?;

    // LTV check: new_debt <= coll_usd * max_ltv / BPS
    let max_debt = coll_usd
        .checked_mul(vault.max_ltv_bps as u128)
        .ok_or(KusdError::MathOverflow)?
        / (BPS_SCALE as u128);
    require!(new_debt <= max_debt, KusdError::ExceedsMaxLtv);

    // Debt ceiling check
    if vault.debt_ceiling > 0 {
        // Compute total outstanding debt from all positions
        let total_debt = math::shares_to_debt(vault.total_debt_shares, vault.cumulative_fee_index)?;
        let new_total = (total_debt as u128)
            .checked_add(amount as u128)
            .ok_or(KusdError::MathOverflow)?;
        require!(new_total <= vault.debt_ceiling as u128, KusdError::DebtCeilingExceeded);
    }

    // Compute new debt shares for this mint
    let new_shares = math::amount_to_shares(amount, vault.cumulative_fee_index)?;

    // Mint kUSD to borrower
    let vault_key = vault.key();
    let authority_seeds: &[&[u8]] = &[
        CDP_VAULT_AUTHORITY_SEED,
        vault_key.as_ref(),
        &[vault.authority_bump],
    ];
    let signer_seeds = [authority_seeds];

    let cpi_accounts = MintTo {
        mint: ctx.accounts.kusd_mint.to_account_info(),
        to: ctx.accounts.borrower_kusd.to_account_info(),
        authority: ctx.accounts.vault_authority.to_account_info(),
    };
    let cpi_ctx = CpiContext::new_with_signer(
        ctx.accounts.token_program.to_account_info(),
        cpi_accounts,
        &signer_seeds,
    );
    token::mint_to(cpi_ctx, amount)?;

    // Update position debt shares
    let position = &mut ctx.accounts.position;
    position.debt_shares = position
        .debt_shares
        .checked_add(new_shares)
        .ok_or(KusdError::MathOverflow)?;

    // Update vault total debt shares
    let vault = &mut ctx.accounts.vault;
    vault.total_debt_shares = vault
        .total_debt_shares
        .checked_add(new_shares)
        .ok_or(KusdError::MathOverflow)?;

    Ok(())
}
