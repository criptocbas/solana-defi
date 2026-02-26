use anchor_lang::prelude::*;
use anchor_spl::token::{Token, TokenAccount};

use crate::cpi_helpers::execute_leg;
use crate::errors::KrouterError;
use crate::types::LegDescriptor;

#[derive(Accounts)]
pub struct RouteTwoHop<'info> {
    pub user: Signer<'info>,

    /// User's source token account (token A)
    #[account(mut)]
    pub user_token_source: Account<'info, TokenAccount>,

    /// User's intermediate token account (token B) — output of leg 1, input of leg 2
    #[account(mut)]
    pub user_token_intermediate: Account<'info, TokenAccount>,

    /// User's destination token account (token C)
    #[account(mut)]
    pub user_token_destination: Account<'info, TokenAccount>,

    pub token_program: Program<'info, Token>,

    // remaining_accounts: [leg1 accounts..., leg2 accounts...]
}

pub fn handle_route_two_hop<'info>(
    ctx: Context<'_, '_, 'info, 'info, RouteTwoHop<'info>>,
    amount_in: u64,
    minimum_amount_out: u64,
    leg1: LegDescriptor,
    leg2: LegDescriptor,
) -> Result<()> {
    require!(amount_in > 0, KrouterError::ZeroSwapAmount);

    let remaining = ctx.remaining_accounts;
    let leg1_end = leg1.num_accounts as usize;
    require!(remaining.len() >= leg1_end + leg2.num_accounts as usize, KrouterError::InsufficientAccounts);

    let leg1_accounts = &remaining[..leg1_end];
    let leg2_accounts = &remaining[leg1_end..leg1_end + leg2.num_accounts as usize];

    // Record intermediate balance before leg 1
    let intermediate_before = ctx.accounts.user_token_intermediate.amount;

    // === Leg 1: source -> intermediate ===
    execute_leg(
        leg1.pool_type,
        leg1_accounts,
        &ctx.accounts.user.to_account_info(),
        &ctx.accounts.user_token_source.to_account_info(),
        &ctx.accounts.user_token_intermediate.to_account_info(),
        &ctx.accounts.token_program.to_account_info(),
        amount_in,
        leg1.sqrt_price_limit,
    )?;

    // Reload intermediate to measure leg 1 output
    ctx.accounts.user_token_intermediate.reload()?;
    let intermediate_amount = ctx.accounts.user_token_intermediate.amount
        .checked_sub(intermediate_before)
        .ok_or(KrouterError::MathOverflow)?;

    // === Leg 2: intermediate -> destination ===
    let dest_balance_before = ctx.accounts.user_token_destination.amount;

    execute_leg(
        leg2.pool_type,
        leg2_accounts,
        &ctx.accounts.user.to_account_info(),
        &ctx.accounts.user_token_intermediate.to_account_info(),
        &ctx.accounts.user_token_destination.to_account_info(),
        &ctx.accounts.token_program.to_account_info(),
        intermediate_amount,
        leg2.sqrt_price_limit,
    )?;

    // Reload and enforce end-to-end slippage
    ctx.accounts.user_token_destination.reload()?;
    let actual_out = ctx.accounts.user_token_destination.amount
        .checked_sub(dest_balance_before)
        .ok_or(KrouterError::MathOverflow)?;

    require!(actual_out >= minimum_amount_out, KrouterError::SlippageExceeded);

    Ok(())
}
