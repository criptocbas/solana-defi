use anchor_lang::prelude::*;

use crate::constants::*;
use crate::errors::KclmmError;
use crate::state::{Pool, Position};

#[derive(Accounts)]
pub struct ClosePosition<'info> {
    #[account(mut)]
    pub owner: Signer<'info>,

    pub pool: Account<'info, Pool>,

    #[account(
        mut,
        close = owner,
        has_one = pool @ KclmmError::PositionPoolMismatch,
        has_one = owner,
        seeds = [
            POSITION_SEED,
            pool.key().as_ref(),
            owner.key().as_ref(),
            &position.tick_lower.to_le_bytes(),
            &position.tick_upper.to_le_bytes(),
        ],
        bump = position.bump,
    )]
    pub position: Account<'info, Position>,

    pub system_program: Program<'info, System>,
}

pub fn handle_close_position(ctx: Context<ClosePosition>) -> Result<()> {
    let position = &ctx.accounts.position;

    // Ensure position is empty
    require!(position.liquidity == 0, KclmmError::PositionNotEmpty);
    require!(position.tokens_owed_a == 0, KclmmError::PositionNotEmpty);
    require!(position.tokens_owed_b == 0, KclmmError::PositionNotEmpty);

    // Account is closed via `close = owner` in the Accounts derive

    Ok(())
}
