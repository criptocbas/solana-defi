use anchor_lang::prelude::*;

use crate::constants::*;
use crate::errors::KclmmError;
use crate::state::{Pool, TickArray, Tick};

#[derive(Accounts)]
#[instruction(start_tick_index: i32)]
pub struct InitTickArray<'info> {
    #[account(mut)]
    pub payer: Signer<'info>,

    pub pool: Account<'info, Pool>,

    #[account(
        init,
        payer = payer,
        space = TickArray::SPACE,
        seeds = [TICK_ARRAY_SEED, pool.key().as_ref(), &start_tick_index.to_le_bytes()],
        bump,
    )]
    pub tick_array: AccountLoader<'info, TickArray>,

    pub system_program: Program<'info, System>,
}

pub fn handle_init_tick_array(
    ctx: Context<InitTickArray>,
    start_tick_index: i32,
) -> Result<()> {
    let pool = &ctx.accounts.pool;

    // Validate alignment: start_tick_index must be aligned to tick_spacing * TICKS_PER_ARRAY
    let ticks_in_array = pool.tick_spacing as i32 * TICKS_PER_ARRAY as i32;
    require!(
        start_tick_index % ticks_in_array == 0,
        KclmmError::InvalidTickArrayStartIndex
    );

    // Validate bounds
    require!(
        start_tick_index >= MIN_TICK - ticks_in_array && start_tick_index <= MAX_TICK,
        KclmmError::TickOutOfBounds
    );

    let mut tick_array = ctx.accounts.tick_array.load_init()?;
    tick_array.pool = pool.key();
    tick_array.start_tick_index = start_tick_index;
    tick_array.initialized_bitmap = 0;
    tick_array.ticks = [Tick::default(); TICKS_PER_ARRAY];

    Ok(())
}
