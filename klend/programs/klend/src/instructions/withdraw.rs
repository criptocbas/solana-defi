use anchor_lang::prelude::*;
use anchor_spl::token::{self, Token, TokenAccount, Transfer};

use crate::constants::*;
use crate::errors::KlendError;
use crate::instructions::health::{self, WeightMode};
use crate::math;
use crate::state::{LendingMarket, MockOracle, Obligation, Reserve};

#[derive(Accounts)]
pub struct Withdraw<'info> {
    #[account(mut)]
    pub user: Signer<'info>,

    #[account(
        seeds = [LENDING_MARKET_SEED, lending_market.admin.as_ref()],
        bump = lending_market.bump,
    )]
    pub lending_market: Account<'info, LendingMarket>,

    #[account(
        mut,
        seeds = [RESERVE_SEED, lending_market.key().as_ref(), reserve.token_mint.as_ref()],
        bump = reserve.bump,
        has_one = lending_market,
        has_one = token_vault,
    )]
    pub reserve: Account<'info, Reserve>,

    /// CHECK: PDA signing authority for vault
    #[account(
        seeds = [RESERVE_AUTHORITY_SEED, reserve.key().as_ref()],
        bump = reserve.authority_bump,
    )]
    pub reserve_authority: UncheckedAccount<'info>,

    #[account(
        mut,
        seeds = [OBLIGATION_SEED, lending_market.key().as_ref(), user.key().as_ref()],
        bump = obligation.bump,
        has_one = owner @ KlendError::NoCollateralDeposit,
    )]
    pub obligation: Account<'info, Obligation>,

    /// CHECK: validated by has_one
    pub owner: AccountInfo<'info>,

    #[account(mut)]
    pub user_token_account: Account<'info, TokenAccount>,

    #[account(mut)]
    pub token_vault: Account<'info, TokenAccount>,

    // Oracle for this reserve (needed for health check if user has borrows)
    pub oracle: Account<'info, MockOracle>,

    pub token_program: Program<'info, Token>,
}

pub fn handle_withdraw(ctx: Context<Withdraw>, shares: u64) -> Result<()> {
    require!(shares > 0, KlendError::ZeroWithdraw);

    let reserve = &ctx.accounts.reserve;

    // Check reserve freshness
    let clock = Clock::get()?;
    require!(
        clock.unix_timestamp.saturating_sub(reserve.last_update_timestamp)
            <= RESERVE_FRESHNESS_SECONDS,
        KlendError::ReserveStale
    );

    // Convert shares to underlying
    let underlying = math::shares_to_underlying(
        shares,
        reserve.total_shares,
        reserve.total_assets(),
    )?;

    // Check against actual vault balance (more accurate than available_liquidity() after interest/repay)
    require!(
        underlying <= ctx.accounts.token_vault.amount,
        KlendError::InsufficientLiquidity
    );

    // Update obligation -- remove shares
    let obligation = &mut ctx.accounts.obligation;
    let reserve_key = reserve.key();
    let dep = obligation
        .deposits
        .iter_mut()
        .find(|d| d.reserve == reserve_key)
        .ok_or(KlendError::NoCollateralDeposit)?;

    dep.shares = dep
        .shares
        .checked_sub(shares)
        .ok_or(KlendError::MathOverflow)?;

    // Remove entry if fully withdrawn
    if dep.shares == 0 {
        obligation.deposits.retain(|d| d.reserve != reserve_key);
    }

    // If user has borrows, check health factor after withdrawal
    // Obligation shares are already decremented above, so HF reflects post-withdrawal state.
    // If HF < 1.0, entire tx reverts (Anchor rolls back all account changes).
    if !obligation.borrows.is_empty() {
        let lending_market_key = ctx.accounts.lending_market.key();
        let (_, _, hf) = health::compute_obligation_health(
            obligation,
            ctx.remaining_accounts,
            &lending_market_key,
            &clock,
            WeightMode::LiquidationThreshold,
        )?;
        require!(hf >= SCALE, KlendError::HealthFactorTooLow);
    }

    // Transfer tokens from vault to user
    let reserve_key_bytes = reserve.key();
    let authority_seeds: &[&[u8]] = &[
        RESERVE_AUTHORITY_SEED,
        reserve_key_bytes.as_ref(),
        &[reserve.authority_bump],
    ];

    let cpi_accounts = Transfer {
        from: ctx.accounts.token_vault.to_account_info(),
        to: ctx.accounts.user_token_account.to_account_info(),
        authority: ctx.accounts.reserve_authority.to_account_info(),
    };
    let signer_seeds = [authority_seeds];
    let cpi_ctx = CpiContext::new_with_signer(
        ctx.accounts.token_program.to_account_info(),
        cpi_accounts,
        &signer_seeds,
    );
    token::transfer(cpi_ctx, underlying)?;

    // Update reserve
    let reserve = &mut ctx.accounts.reserve;
    reserve.deposited_liquidity = reserve
        .deposited_liquidity
        .checked_sub(underlying)
        .ok_or(KlendError::MathOverflow)?;
    reserve.total_shares = reserve
        .total_shares
        .checked_sub(shares)
        .ok_or(KlendError::MathOverflow)?;

    Ok(())
}
