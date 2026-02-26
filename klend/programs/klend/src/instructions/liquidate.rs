use anchor_lang::prelude::*;
use anchor_spl::token::{self, Token, TokenAccount, Transfer};

use crate::constants::*;
use crate::errors::KlendError;
use crate::math;
use crate::state::{LendingMarket, MockOracle, Obligation, Reserve};

#[derive(Accounts)]
pub struct Liquidate<'info> {
    #[account(mut)]
    pub liquidator: Signer<'info>,

    #[account(
        seeds = [LENDING_MARKET_SEED, lending_market.admin.as_ref()],
        bump = lending_market.bump,
    )]
    pub lending_market: Account<'info, LendingMarket>,

    /// The reserve whose tokens the liquidator is repaying (debt reserve)
    #[account(
        mut,
        seeds = [RESERVE_SEED, lending_market.key().as_ref(), debt_reserve.token_mint.as_ref()],
        bump = debt_reserve.bump,
        has_one = lending_market,
    )]
    pub debt_reserve: Account<'info, Reserve>,

    /// The reserve from which collateral is seized
    #[account(
        mut,
        seeds = [RESERVE_SEED, lending_market.key().as_ref(), collateral_reserve.token_mint.as_ref()],
        bump = collateral_reserve.bump,
        has_one = lending_market,
    )]
    pub collateral_reserve: Account<'info, Reserve>,

    /// CHECK: PDA signing authority for collateral vault
    #[account(
        seeds = [RESERVE_AUTHORITY_SEED, collateral_reserve.key().as_ref()],
        bump = collateral_reserve.authority_bump,
    )]
    pub collateral_reserve_authority: UncheckedAccount<'info>,

    /// The obligation being liquidated
    #[account(
        mut,
        seeds = [OBLIGATION_SEED, lending_market.key().as_ref(), obligation_owner.key().as_ref()],
        bump = obligation.bump,
    )]
    pub obligation: Account<'info, Obligation>,

    /// CHECK: owner of the obligation being liquidated
    pub obligation_owner: AccountInfo<'info>,

    pub debt_oracle: Account<'info, MockOracle>,
    pub collateral_oracle: Account<'info, MockOracle>,

    /// Liquidator's debt token account (pays debt tokens in)
    #[account(mut)]
    pub liquidator_debt_token: Account<'info, TokenAccount>,

    /// Debt reserve vault (receives debt tokens)
    #[account(mut, constraint = debt_vault.key() == debt_reserve.token_vault)]
    pub debt_vault: Account<'info, TokenAccount>,

    /// Liquidator's collateral token account (receives seized collateral)
    #[account(mut)]
    pub liquidator_collateral_token: Account<'info, TokenAccount>,

    /// Collateral reserve vault (collateral seized from here)
    #[account(mut, constraint = collateral_vault.key() == collateral_reserve.token_vault)]
    pub collateral_vault: Account<'info, TokenAccount>,

    pub token_program: Program<'info, Token>,
}

pub fn handle_liquidate(ctx: Context<Liquidate>, amount: u64) -> Result<()> {
    require!(amount > 0, KlendError::ZeroLiquidation);

    let debt_reserve = &ctx.accounts.debt_reserve;
    let collateral_reserve = &ctx.accounts.collateral_reserve;
    let clock = Clock::get()?;

    // Check reserves freshness
    require!(
        clock.unix_timestamp.saturating_sub(debt_reserve.last_update_timestamp) <= 2,
        KlendError::ReserveStale
    );
    require!(
        clock.unix_timestamp.saturating_sub(collateral_reserve.last_update_timestamp) <= 2,
        KlendError::ReserveStale
    );

    let obligation = &ctx.accounts.obligation;
    let collateral_oracle = &ctx.accounts.collateral_oracle;
    let debt_oracle = &ctx.accounts.debt_oracle;

    // Compute collateral value
    let collateral_reserve_key = collateral_reserve.key();
    let collateral_deposit = obligation
        .deposits
        .iter()
        .find(|d| d.reserve == collateral_reserve_key)
        .ok_or(KlendError::NoCollateralDeposit)?;

    let collateral_underlying = math::shares_to_underlying(
        collateral_deposit.shares,
        collateral_reserve.total_shares,
        collateral_reserve.total_assets(),
    )?;

    let collateral_value = math::collateral_value_usd(
        collateral_underlying,
        collateral_oracle.price,
        collateral_oracle.decimals,
    )?;
    let weighted_collateral = math::weighted_collateral_value(
        collateral_value,
        collateral_reserve.config.liquidation_threshold,
    )?;

    // Compute total debt value
    let debt_reserve_key = debt_reserve.key();
    let borrow_entry = obligation
        .borrows
        .iter()
        .find(|b| b.reserve == debt_reserve_key)
        .ok_or(KlendError::NoBorrowFound)?;

    let current_debt = borrow_entry.current_debt(debt_reserve.cumulative_borrow_index);
    let total_debt_value = math::collateral_value_usd(
        current_debt,
        debt_oracle.price,
        debt_oracle.decimals,
    )?;

    // Verify position is unhealthy (HF < 1.0)
    let hf = math::health_factor(weighted_collateral, total_debt_value)?;
    require!(hf < SCALE, KlendError::PositionHealthy);

    // Enforce close factor: max repay = current_debt * CLOSE_FACTOR_BPS / BPS_SCALE
    let max_repay = (current_debt as u128)
        .checked_mul(CLOSE_FACTOR_BPS as u128)
        .ok_or(KlendError::MathOverflow)?
        / (BPS_SCALE as u128);
    let repay_amount = amount.min(max_repay as u64).min(current_debt);
    require!(amount <= max_repay as u64, KlendError::CloseFactorExceeded);

    // Compute collateral to seize
    let collateral_seized = math::liquidation_collateral_seized(
        repay_amount,
        debt_oracle.price,
        debt_oracle.decimals,
        collateral_oracle.price,
        collateral_oracle.decimals,
        collateral_reserve.config.liquidation_bonus,
    )?;

    // Convert seized collateral to shares
    let seized_shares = math::underlying_to_shares(
        collateral_seized,
        collateral_reserve.total_shares,
        collateral_reserve.total_assets(),
    )?;

    // 1. Transfer debt tokens from liquidator to debt vault
    let cpi_accounts = Transfer {
        from: ctx.accounts.liquidator_debt_token.to_account_info(),
        to: ctx.accounts.debt_vault.to_account_info(),
        authority: ctx.accounts.liquidator.to_account_info(),
    };
    let cpi_ctx = CpiContext::new(ctx.accounts.token_program.to_account_info(), cpi_accounts);
    token::transfer(cpi_ctx, repay_amount)?;

    // 2. Transfer collateral from collateral vault to liquidator
    let collateral_reserve_key_bytes = collateral_reserve.key();
    let authority_seeds: &[&[u8]] = &[
        RESERVE_AUTHORITY_SEED,
        collateral_reserve_key_bytes.as_ref(),
        &[collateral_reserve.authority_bump],
    ];

    let cpi_accounts = Transfer {
        from: ctx.accounts.collateral_vault.to_account_info(),
        to: ctx.accounts.liquidator_collateral_token.to_account_info(),
        authority: ctx.accounts.collateral_reserve_authority.to_account_info(),
    };
    let signer_seeds = [authority_seeds];
    let cpi_ctx = CpiContext::new_with_signer(
        ctx.accounts.token_program.to_account_info(),
        cpi_accounts,
        &signer_seeds,
    );
    token::transfer(cpi_ctx, collateral_seized)?;

    // Update debt reserve
    let debt_reserve = &mut ctx.accounts.debt_reserve;
    debt_reserve.borrowed_liquidity = debt_reserve
        .borrowed_liquidity
        .saturating_sub(repay_amount);

    // Update collateral reserve
    let collateral_reserve = &mut ctx.accounts.collateral_reserve;
    collateral_reserve.deposited_liquidity = collateral_reserve
        .deposited_liquidity
        .saturating_sub(collateral_seized);
    collateral_reserve.total_shares = collateral_reserve
        .total_shares
        .saturating_sub(seized_shares);

    // Update obligation
    let obligation = &mut ctx.accounts.obligation;
    let borrow_index = debt_reserve.cumulative_borrow_index;

    // Update borrow entry
    let debt_reserve_key = debt_reserve.key();
    let scaled_repay = (repay_amount as u128)
        .checked_mul(SCALE)
        .ok_or(KlendError::MathOverflow)?
        / borrow_index;

    if repay_amount >= current_debt {
        obligation.borrows.retain(|b| b.reserve != debt_reserve_key);
    } else if let Some(borrow) = obligation
        .borrows
        .iter_mut()
        .find(|b| b.reserve == debt_reserve_key)
    {
        borrow.borrowed_amount_scaled = borrow
            .borrowed_amount_scaled
            .saturating_sub(scaled_repay);
    }

    // Update deposit entry
    let collateral_reserve_key = collateral_reserve.key();
    if let Some(dep) = obligation
        .deposits
        .iter_mut()
        .find(|d| d.reserve == collateral_reserve_key)
    {
        dep.shares = dep.shares.saturating_sub(seized_shares);
        if dep.shares == 0 {
            obligation
                .deposits
                .retain(|d| d.reserve != collateral_reserve_key);
        }
    }

    Ok(())
}
