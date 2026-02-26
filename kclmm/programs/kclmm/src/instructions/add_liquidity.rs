use anchor_lang::prelude::*;
use anchor_spl::token::{self, Token, TokenAccount, Transfer};

use crate::errors::KclmmError;
use crate::math;
use crate::state::{Pool, Position, TickArray};

#[derive(Accounts)]
pub struct AddLiquidity<'info> {
    pub owner: Signer<'info>,

    #[account(mut)]
    pub pool: Account<'info, Pool>,

    #[account(
        mut,
        has_one = pool @ KclmmError::PositionPoolMismatch,
        has_one = owner,
    )]
    pub position: Account<'info, Position>,

    #[account(mut)]
    pub tick_array_lower: AccountLoader<'info, TickArray>,

    #[account(mut)]
    pub tick_array_upper: AccountLoader<'info, TickArray>,

    #[account(
        mut,
        constraint = vault_a.key() == pool.vault_a,
    )]
    pub vault_a: Account<'info, TokenAccount>,

    #[account(
        mut,
        constraint = vault_b.key() == pool.vault_b,
    )]
    pub vault_b: Account<'info, TokenAccount>,

    #[account(mut)]
    pub user_token_a: Account<'info, TokenAccount>,

    #[account(mut)]
    pub user_token_b: Account<'info, TokenAccount>,

    pub token_program: Program<'info, Token>,
}

pub fn handle_add_liquidity(
    ctx: Context<AddLiquidity>,
    liquidity_delta: u128,
    amount_a_max: u64,
    amount_b_max: u64,
) -> Result<()> {
    require!(liquidity_delta > 0, KclmmError::ZeroLiquidityDelta);

    let pool = &ctx.accounts.pool;
    let position = &ctx.accounts.position;

    let tick_lower = position.tick_lower;
    let tick_upper = position.tick_upper;

    let sqrt_lower = math::tick_to_sqrt_price(tick_lower)?;
    let sqrt_upper = math::tick_to_sqrt_price(tick_upper)?;

    // --- Compute tick indices ---
    let ta_lower = ctx.accounts.tick_array_lower.load()?;
    require!(ta_lower.pool == pool.key(), KclmmError::TickArrayPoolMismatch);
    let lower_idx = math::tick_index_in_array(
        tick_lower,
        ta_lower.start_tick_index,
        pool.tick_spacing,
    ).ok_or(error!(KclmmError::TickNotInArray))?;
    let lower_tick = ta_lower.ticks[lower_idx];
    drop(ta_lower);

    let ta_upper = ctx.accounts.tick_array_upper.load()?;
    require!(ta_upper.pool == pool.key(), KclmmError::TickArrayPoolMismatch);
    let upper_idx = math::tick_index_in_array(
        tick_upper,
        ta_upper.start_tick_index,
        pool.tick_spacing,
    ).ok_or(error!(KclmmError::TickNotInArray))?;
    let upper_tick = ta_upper.ticks[upper_idx];
    drop(ta_upper);

    // --- Accrue pending fees to position ---
    let (fee_inside_a, fee_inside_b) = math::fee_growth_inside(
        &lower_tick,
        &upper_tick,
        tick_lower,
        tick_upper,
        pool.tick_current,
        pool.fee_growth_global_a,
        pool.fee_growth_global_b,
    );

    // Update position fee accounting
    let position = &mut ctx.accounts.position;
    if position.liquidity > 0 {
        let (owed_a, owed_b) = math::compute_fees_owed(
            fee_inside_a,
            fee_inside_b,
            position.fee_growth_inside_last_a,
            position.fee_growth_inside_last_b,
            position.liquidity,
        );
        position.tokens_owed_a = position.tokens_owed_a.checked_add(owed_a)
            .ok_or(error!(KclmmError::MathOverflow))?;
        position.tokens_owed_b = position.tokens_owed_b.checked_add(owed_b)
            .ok_or(error!(KclmmError::MathOverflow))?;
    }
    position.fee_growth_inside_last_a = fee_inside_a;
    position.fee_growth_inside_last_b = fee_inside_b;

    // --- Update tick states ---
    {
        let mut ta = ctx.accounts.tick_array_lower.load_mut()?;
        let tick_data = &mut ta.ticks[lower_idx];
        let was_empty = tick_data.liquidity_gross == 0;
        tick_data.liquidity_gross = tick_data.liquidity_gross
            .checked_add(liquidity_delta)
            .ok_or(error!(KclmmError::MathOverflow))?;
        tick_data.liquidity_net = tick_data.liquidity_net
            .checked_add(liquidity_delta as i128)
            .ok_or(error!(KclmmError::MathOverflow))?;
        if was_empty {
            let pool = &ctx.accounts.pool;
            if pool.tick_current >= tick_lower {
                tick_data.fee_growth_outside_a = pool.fee_growth_global_a;
                tick_data.fee_growth_outside_b = pool.fee_growth_global_b;
            }
            math::set_bit(&mut ta.initialized_bitmap, lower_idx);
        }
    }

    {
        let mut ta = ctx.accounts.tick_array_upper.load_mut()?;
        let tick_data = &mut ta.ticks[upper_idx];
        let was_empty = tick_data.liquidity_gross == 0;
        tick_data.liquidity_gross = tick_data.liquidity_gross
            .checked_add(liquidity_delta)
            .ok_or(error!(KclmmError::MathOverflow))?;
        tick_data.liquidity_net = tick_data.liquidity_net
            .checked_sub(liquidity_delta as i128)
            .ok_or(error!(KclmmError::MathOverflow))?;
        if was_empty {
            let pool = &ctx.accounts.pool;
            if pool.tick_current >= tick_upper {
                tick_data.fee_growth_outside_a = pool.fee_growth_global_a;
                tick_data.fee_growth_outside_b = pool.fee_growth_global_b;
            }
            math::set_bit(&mut ta.initialized_bitmap, upper_idx);
        }
    }

    // --- Update pool active liquidity ---
    let pool = &mut ctx.accounts.pool;
    if pool.tick_current >= tick_lower && pool.tick_current < tick_upper {
        pool.liquidity = pool.liquidity
            .checked_add(liquidity_delta)
            .ok_or(error!(KclmmError::MathOverflow))?;
    }

    // --- Update position liquidity ---
    let position = &mut ctx.accounts.position;
    position.liquidity = position.liquidity
        .checked_add(liquidity_delta)
        .ok_or(error!(KclmmError::MathOverflow))?;

    // --- Compute token amounts ---
    let (amount_a, amount_b) = math::get_amounts_for_liquidity(
        pool.sqrt_price,
        sqrt_lower,
        sqrt_upper,
        liquidity_delta,
        true, // round up for deposits
    )?;

    // --- Slippage check ---
    require!(amount_a <= amount_a_max, KclmmError::AmountAExceedsMax);
    require!(amount_b <= amount_b_max, KclmmError::AmountBExceedsMax);

    // --- Transfer tokens ---
    if amount_a > 0 {
        token::transfer(
            CpiContext::new(
                ctx.accounts.token_program.to_account_info(),
                Transfer {
                    from: ctx.accounts.user_token_a.to_account_info(),
                    to: ctx.accounts.vault_a.to_account_info(),
                    authority: ctx.accounts.owner.to_account_info(),
                },
            ),
            amount_a,
        )?;
    }

    if amount_b > 0 {
        token::transfer(
            CpiContext::new(
                ctx.accounts.token_program.to_account_info(),
                Transfer {
                    from: ctx.accounts.user_token_b.to_account_info(),
                    to: ctx.accounts.vault_b.to_account_info(),
                    authority: ctx.accounts.owner.to_account_info(),
                },
            ),
            amount_b,
        )?;
    }

    Ok(())
}
