use anchor_lang::prelude::*;

pub mod constants;
pub mod errors;
pub mod instructions;
pub mod math;
pub mod state;

use instructions::*;

declare_id!("7g3bAmnUmaoXZcDxffmxsZk7hmhMNDcQw7pT2aNC5tYW");

#[program]
pub mod kclmm {
    use super::*;

    pub fn init_pool(
        ctx: Context<InitPool>,
        fee_rate: u32,
        initial_sqrt_price: u128,
    ) -> Result<()> {
        instructions::init_pool::handle_init_pool(ctx, fee_rate, initial_sqrt_price)
    }

    pub fn init_tick_array(
        ctx: Context<InitTickArray>,
        start_tick_index: i32,
    ) -> Result<()> {
        instructions::init_tick_array::handle_init_tick_array(ctx, start_tick_index)
    }

    pub fn open_position(
        ctx: Context<OpenPosition>,
        tick_lower: i32,
        tick_upper: i32,
    ) -> Result<()> {
        instructions::open_position::handle_open_position(ctx, tick_lower, tick_upper)
    }

    pub fn add_liquidity(
        ctx: Context<AddLiquidity>,
        liquidity_delta: u128,
        amount_a_max: u64,
        amount_b_max: u64,
    ) -> Result<()> {
        instructions::add_liquidity::handle_add_liquidity(ctx, liquidity_delta, amount_a_max, amount_b_max)
    }

    pub fn remove_liquidity(
        ctx: Context<RemoveLiquidity>,
        liquidity_delta: u128,
        amount_a_min: u64,
        amount_b_min: u64,
    ) -> Result<()> {
        instructions::remove_liquidity::handle_remove_liquidity(ctx, liquidity_delta, amount_a_min, amount_b_min)
    }

    pub fn collect_fees(ctx: Context<CollectFees>) -> Result<()> {
        instructions::collect_fees::handle_collect_fees(ctx)
    }

    pub fn swap(
        ctx: Context<Swap>,
        amount_in: u64,
        sqrt_price_limit: u128,
        minimum_amount_out: u64,
    ) -> Result<()> {
        instructions::swap::handle_swap(ctx, amount_in, sqrt_price_limit, minimum_amount_out)
    }

    pub fn close_position(ctx: Context<ClosePosition>) -> Result<()> {
        instructions::close_position::handle_close_position(ctx)
    }
}
