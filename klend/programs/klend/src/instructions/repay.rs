use anchor_lang::prelude::*;
use anchor_spl::token::{self, Token, TokenAccount, Transfer};

use crate::constants::*;
use crate::errors::KlendError;
use crate::state::{LendingMarket, Obligation, Reserve};

#[derive(Accounts)]
pub struct Repay<'info> {
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

    #[account(
        mut,
        seeds = [OBLIGATION_SEED, lending_market.key().as_ref(), obligation_owner.key().as_ref()],
        bump = obligation.bump,
    )]
    pub obligation: Account<'info, Obligation>,

    /// CHECK: The owner of the obligation (can repay on behalf of someone else)
    pub obligation_owner: AccountInfo<'info>,

    #[account(mut)]
    pub user_token_account: Account<'info, TokenAccount>,

    #[account(mut)]
    pub token_vault: Account<'info, TokenAccount>,

    pub token_program: Program<'info, Token>,
}

pub fn handle_repay(ctx: Context<Repay>, amount: u64) -> Result<()> {
    require!(amount > 0, KlendError::ZeroRepay);

    let reserve = &ctx.accounts.reserve;

    // Check reserve freshness
    let clock = Clock::get()?;
    require!(
        clock.unix_timestamp.saturating_sub(reserve.last_update_timestamp)
            <= RESERVE_FRESHNESS_SECONDS,
        KlendError::ReserveStale
    );

    let obligation = &mut ctx.accounts.obligation;
    let reserve_key = reserve.key();
    let borrow_index = reserve.cumulative_borrow_index;

    // Find borrow entry
    let borrow = obligation
        .borrows
        .iter_mut()
        .find(|b| b.reserve == reserve_key)
        .ok_or(KlendError::NoBorrowFound)?;

    // Compute current debt
    let current_debt = borrow.current_debt(borrow_index)?;

    // Cap repay at current debt
    let repay_amount = amount.min(current_debt);

    // Transfer tokens to vault
    let cpi_accounts = Transfer {
        from: ctx.accounts.user_token_account.to_account_info(),
        to: ctx.accounts.token_vault.to_account_info(),
        authority: ctx.accounts.user.to_account_info(),
    };
    let cpi_ctx = CpiContext::new(ctx.accounts.token_program.to_account_info(), cpi_accounts);
    token::transfer(cpi_ctx, repay_amount)?;

    // Update obligation borrow entry
    // scaled_repay = repay_amount * SCALE / borrow_index
    let scaled_repay = (repay_amount as u128)
        .checked_mul(SCALE)
        .ok_or(KlendError::MathOverflow)?
        / borrow_index;

    if repay_amount >= current_debt {
        // Full repay - remove entry
        obligation.borrows.retain(|b| b.reserve != reserve_key);
    } else {
        borrow.borrowed_amount_scaled = borrow
            .borrowed_amount_scaled
            .saturating_sub(scaled_repay);
    }

    // Update reserve
    let reserve = &mut ctx.accounts.reserve;
    reserve.borrowed_liquidity = reserve
        .borrowed_liquidity
        .saturating_sub(repay_amount);
    reserve.deposited_liquidity = reserve
        .deposited_liquidity
        .checked_add(repay_amount)
        .ok_or(KlendError::MathOverflow)?;

    Ok(())
}
