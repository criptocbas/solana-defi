use anchor_lang::prelude::*;
use anchor_spl::token::{self, Burn, Mint, Token, TokenAccount, Transfer};

use crate::constants::*;
use crate::errors::KusdError;
use crate::instructions::common::accrue_vault_fees;
use crate::math;
use crate::state::{CdpPosition, CdpVault, MockOracle};

#[derive(Accounts)]
pub struct Liquidate<'info> {
    #[account(mut)]
    pub liquidator: Signer<'info>,

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
        seeds = [CDP_POSITION_SEED, vault.key().as_ref(), position_owner.key().as_ref()],
        bump = position.bump,
        has_one = vault,
    )]
    pub position: Account<'info, CdpPosition>,

    /// CHECK: owner of the position being liquidated
    pub position_owner: AccountInfo<'info>,

    #[account(
        constraint = oracle.key() == vault.oracle @ KusdError::InvalidOracle,
    )]
    pub oracle: Account<'info, MockOracle>,

    #[account(
        mut,
        seeds = [KUSD_MINT_SEED, vault.key().as_ref()],
        bump = vault.kusd_mint_bump,
    )]
    pub kusd_mint: Account<'info, Mint>,

    /// Liquidator's kUSD token account (burned)
    #[account(mut)]
    pub liquidator_kusd: Account<'info, TokenAccount>,

    /// Liquidator's collateral token account (receives seized collateral)
    #[account(mut)]
    pub liquidator_collateral: Account<'info, TokenAccount>,

    /// Vault's collateral token account
    #[account(
        mut,
        constraint = collateral_vault.key() == vault.collateral_token_account,
    )]
    pub collateral_vault: Account<'info, TokenAccount>,

    pub token_program: Program<'info, Token>,
}

pub fn handle_liquidate(ctx: Context<Liquidate>, repay_amount: u64) -> Result<()> {
    require!(repay_amount > 0, KusdError::ZeroLiquidation);

    let vault = &mut ctx.accounts.vault;

    // Accrue fees
    let clock = Clock::get()?;
    accrue_vault_fees(vault, clock.unix_timestamp)?;

    // Check oracle staleness
    let oracle = &ctx.accounts.oracle;
    let staleness = clock.unix_timestamp.saturating_sub(oracle.timestamp) as u64;
    require!(staleness <= vault.oracle_max_staleness, KusdError::OracleStale);

    // Compute current debt
    let position = &ctx.accounts.position;
    let current_debt = math::shares_to_debt(position.debt_shares, vault.cumulative_fee_index)?;

    // Compute collateral USD value
    let coll_usd = math::collateral_value_usd(
        position.collateral_amount,
        oracle.price,
        oracle.decimals,
    )?;

    // Health factor check: must be < SCALE (unhealthy)
    // debt_usd = current_debt (since kUSD = $1 at 1e6 scale)
    let hf = math::health_factor(coll_usd, current_debt as u128, vault.liquidation_threshold_bps)?;
    require!(hf < SCALE, KusdError::PositionHealthy);

    // Close factor: max 50% of debt per liquidation
    let max_repay = (current_debt as u128)
        .checked_mul(CLOSE_FACTOR_BPS as u128)
        .ok_or(KusdError::MathOverflow)?
        / (BPS_SCALE as u128);
    require!(repay_amount as u128 <= max_repay, KusdError::CloseFactorExceeded);

    let actual_repay = repay_amount.min(current_debt);

    // Compute collateral seized
    let collateral_seized = math::liquidation_collateral_seized(
        actual_repay,
        oracle.price,
        oracle.decimals,
        vault.liquidation_bonus_bps,
    )?;

    // Cap seized at position's collateral
    let collateral_seized = collateral_seized.min(position.collateral_amount);

    // 1. Burn kUSD from liquidator
    let cpi_accounts = Burn {
        mint: ctx.accounts.kusd_mint.to_account_info(),
        from: ctx.accounts.liquidator_kusd.to_account_info(),
        authority: ctx.accounts.liquidator.to_account_info(),
    };
    let cpi_ctx = CpiContext::new(ctx.accounts.token_program.to_account_info(), cpi_accounts);
    token::burn(cpi_ctx, actual_repay)?;

    // 2. Transfer seized collateral to liquidator via PDA signer
    let vault_key = vault.key();
    let authority_seeds: &[&[u8]] = &[
        CDP_VAULT_AUTHORITY_SEED,
        vault_key.as_ref(),
        &[vault.authority_bump],
    ];
    let signer_seeds = [authority_seeds];

    let cpi_accounts = Transfer {
        from: ctx.accounts.collateral_vault.to_account_info(),
        to: ctx.accounts.liquidator_collateral.to_account_info(),
        authority: ctx.accounts.vault_authority.to_account_info(),
    };
    let cpi_ctx = CpiContext::new_with_signer(
        ctx.accounts.token_program.to_account_info(),
        cpi_accounts,
        &signer_seeds,
    );
    token::transfer(cpi_ctx, collateral_seized)?;

    // Update position
    let position = &mut ctx.accounts.position;
    if actual_repay >= current_debt {
        let old_shares = position.debt_shares;
        position.debt_shares = 0;
        vault.total_debt_shares = vault.total_debt_shares.saturating_sub(old_shares);
    } else {
        let shares_to_remove = math::amount_to_shares(actual_repay, vault.cumulative_fee_index)?;
        position.debt_shares = position.debt_shares.saturating_sub(shares_to_remove);
        vault.total_debt_shares = vault.total_debt_shares.saturating_sub(shares_to_remove);
    }

    position.collateral_amount = position
        .collateral_amount
        .saturating_sub(collateral_seized);

    vault.total_collateral = vault.total_collateral.saturating_sub(collateral_seized);

    Ok(())
}
