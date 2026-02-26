use anchor_lang::prelude::*;

use crate::constants::*;
use crate::errors::KclmmError;
use crate::math;
use crate::state::{Pool, Position};

#[derive(Accounts)]
#[instruction(tick_lower: i32, tick_upper: i32)]
pub struct OpenPosition<'info> {
    #[account(mut)]
    pub payer: Signer<'info>,

    pub owner: Signer<'info>,

    pub pool: Account<'info, Pool>,

    #[account(
        init,
        payer = payer,
        space = Position::SPACE,
        seeds = [
            POSITION_SEED,
            pool.key().as_ref(),
            owner.key().as_ref(),
            &tick_lower.to_le_bytes(),
            &tick_upper.to_le_bytes(),
        ],
        bump,
    )]
    pub position: Account<'info, Position>,

    pub system_program: Program<'info, System>,
}

pub fn handle_open_position(
    ctx: Context<OpenPosition>,
    tick_lower: i32,
    tick_upper: i32,
) -> Result<()> {
    let pool = &ctx.accounts.pool;

    // Validate tick range
    require!(tick_lower < tick_upper, KclmmError::InvalidTickRange);
    require!(tick_lower >= MIN_TICK, KclmmError::TickOutOfBounds);
    require!(tick_upper <= MAX_TICK, KclmmError::TickOutOfBounds);

    // Validate alignment
    require!(
        math::is_tick_aligned(tick_lower, pool.tick_spacing),
        KclmmError::TickNotAligned
    );
    require!(
        math::is_tick_aligned(tick_upper, pool.tick_spacing),
        KclmmError::TickNotAligned
    );

    let position = &mut ctx.accounts.position;
    position.pool = pool.key();
    position.owner = ctx.accounts.owner.key();
    position.tick_lower = tick_lower;
    position.tick_upper = tick_upper;
    position.liquidity = 0;
    position.fee_growth_inside_last_a = 0;
    position.fee_growth_inside_last_b = 0;
    position.tokens_owed_a = 0;
    position.tokens_owed_b = 0;
    position.bump = ctx.bumps.position;

    Ok(())
}
