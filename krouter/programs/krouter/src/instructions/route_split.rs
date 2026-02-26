use anchor_lang::prelude::*;
use anchor_spl::token::{Token, TokenAccount};

use crate::cpi_helpers::execute_leg;
use crate::errors::KrouterError;
use crate::types::SplitLegDescriptor;

#[derive(Accounts)]
pub struct RouteSplit<'info> {
    pub user: Signer<'info>,

    #[account(mut)]
    pub user_token_in: Account<'info, TokenAccount>,

    #[account(mut)]
    pub user_token_out: Account<'info, TokenAccount>,

    pub token_program: Program<'info, Token>,

    // remaining_accounts: [pool1 accounts..., pool2 accounts...]
}

pub fn handle_route_split<'info>(
    ctx: Context<'_, '_, 'info, 'info, RouteSplit<'info>>,
    total_amount_in: u64,
    minimum_amount_out: u64,
    leg1: SplitLegDescriptor,
    leg2: SplitLegDescriptor,
) -> Result<()> {
    require!(total_amount_in > 0, KrouterError::ZeroSwapAmount);
    require!(
        leg1.amount_in.checked_add(leg2.amount_in).ok_or(KrouterError::MathOverflow)? == total_amount_in,
        KrouterError::SplitAmountMismatch,
    );

    let remaining = ctx.remaining_accounts;
    let leg1_end = leg1.num_accounts as usize;
    require!(remaining.len() >= leg1_end + leg2.num_accounts as usize, KrouterError::InsufficientAccounts);

    let leg1_accounts = &remaining[..leg1_end];
    let leg2_accounts = &remaining[leg1_end..leg1_end + leg2.num_accounts as usize];

    let out_balance_before = ctx.accounts.user_token_out.amount;

    // === Leg 1 ===
    execute_leg(
        leg1.pool_type,
        leg1_accounts,
        &ctx.accounts.user.to_account_info(),
        &ctx.accounts.user_token_in.to_account_info(),
        &ctx.accounts.user_token_out.to_account_info(),
        &ctx.accounts.token_program.to_account_info(),
        leg1.amount_in,
        leg1.sqrt_price_limit,
    )?;

    // === Leg 2 ===
    execute_leg(
        leg2.pool_type,
        leg2_accounts,
        &ctx.accounts.user.to_account_info(),
        &ctx.accounts.user_token_in.to_account_info(),
        &ctx.accounts.user_token_out.to_account_info(),
        &ctx.accounts.token_program.to_account_info(),
        leg2.amount_in,
        leg2.sqrt_price_limit,
    )?;

    // Reload and enforce combined slippage
    ctx.accounts.user_token_out.reload()?;
    let actual_out = ctx.accounts.user_token_out.amount
        .checked_sub(out_balance_before)
        .ok_or(KrouterError::MathOverflow)?;

    require!(actual_out >= minimum_amount_out, KrouterError::SlippageExceeded);

    Ok(())
}
