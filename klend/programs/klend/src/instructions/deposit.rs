use anchor_lang::prelude::*;
use anchor_spl::token::{self, Token, TokenAccount, Transfer};

use crate::constants::*;
use crate::errors::KlendError;
use crate::math;
use crate::state::{LendingMarket, Obligation, ObligationDeposit, Reserve};

#[derive(Accounts)]
pub struct Deposit<'info> {
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
        init_if_needed,
        payer = user,
        space = 8 + Obligation::SPACE,
        seeds = [OBLIGATION_SEED, lending_market.key().as_ref(), user.key().as_ref()],
        bump,
    )]
    pub obligation: Account<'info, Obligation>,

    #[account(mut)]
    pub user_token_account: Account<'info, TokenAccount>,

    #[account(mut)]
    pub token_vault: Account<'info, TokenAccount>,

    pub token_program: Program<'info, Token>,
    pub system_program: Program<'info, System>,
}

impl Obligation {
    pub const SPACE: usize = 32  // lending_market
        + 32  // owner
        + 4 + (MAX_DEPOSITS * (32 + 8))   // Vec<ObligationDeposit>
        + 4 + (MAX_BORROWS * (32 + 16))   // Vec<ObligationBorrow>
        + 1;  // bump
}

pub fn handle_deposit(ctx: Context<Deposit>, amount: u64) -> Result<()> {
    require!(amount > 0, KlendError::ZeroDeposit);

    let reserve = &ctx.accounts.reserve;

    // Check reserve freshness (must be refreshed in same slot/recent)
    let clock = Clock::get()?;
    require!(
        clock.unix_timestamp.saturating_sub(reserve.last_update_timestamp) <= 2,
        KlendError::ReserveStale
    );

    // Check supply cap
    let new_deposited = reserve
        .deposited_liquidity
        .checked_add(amount)
        .ok_or(KlendError::MathOverflow)?;
    if reserve.config.supply_cap > 0 {
        require!(
            new_deposited <= reserve.config.supply_cap,
            KlendError::SupplyCapExceeded
        );
    }

    // Compute shares
    let shares = math::underlying_to_shares(
        amount,
        reserve.total_shares,
        reserve.total_assets(),
    )?;

    // Transfer tokens to vault
    let cpi_accounts = Transfer {
        from: ctx.accounts.user_token_account.to_account_info(),
        to: ctx.accounts.token_vault.to_account_info(),
        authority: ctx.accounts.user.to_account_info(),
    };
    let cpi_ctx = CpiContext::new(ctx.accounts.token_program.to_account_info(), cpi_accounts);
    token::transfer(cpi_ctx, amount)?;

    // Update reserve
    let reserve = &mut ctx.accounts.reserve;
    reserve.deposited_liquidity = new_deposited;
    reserve.total_shares = reserve
        .total_shares
        .checked_add(shares)
        .ok_or(KlendError::MathOverflow)?;

    // Update obligation
    let obligation = &mut ctx.accounts.obligation;
    if obligation.owner == Pubkey::default() {
        // First time init
        obligation.lending_market = ctx.accounts.lending_market.key();
        obligation.owner = ctx.accounts.user.key();
        obligation.deposits = Vec::new();
        obligation.borrows = Vec::new();
        obligation.bump = ctx.bumps.obligation;
    }

    // Find or create deposit entry
    let reserve_key = reserve.key();
    if let Some(dep) = obligation.deposits.iter_mut().find(|d| d.reserve == reserve_key) {
        dep.shares = dep
            .shares
            .checked_add(shares)
            .ok_or(KlendError::MathOverflow)?;
    } else {
        require!(
            obligation.deposits.len() < MAX_DEPOSITS,
            KlendError::MaxEntriesReached
        );
        obligation.deposits.push(ObligationDeposit {
            reserve: reserve_key,
            shares,
        });
    }

    Ok(())
}
