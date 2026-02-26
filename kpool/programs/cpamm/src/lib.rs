use anchor_lang::prelude::*;

pub mod constants;
pub mod errors;
pub mod instructions;
pub mod state;

use instructions::*;

declare_id!("8EpEqMJTjJwFPWbbaSsJi4bDM8z5eZp3aULqdaWppyr9");

#[program]
pub mod cpamm {
    use super::*;

    pub fn initialize_pool(ctx: Context<InitializePool>) -> Result<()> {
        instructions::initialize::handle_initialize_pool(ctx)
    }

    pub fn add_liquidity(
        ctx: Context<AddLiquidity>,
        amount_a_desired: u64,
        amount_b_desired: u64,
        minimum_lp_tokens: u64,
    ) -> Result<()> {
        instructions::add_liquidity::handle_add_liquidity(ctx, amount_a_desired, amount_b_desired, minimum_lp_tokens)
    }

    pub fn remove_liquidity(
        ctx: Context<RemoveLiquidity>,
        lp_burn: u64,
        min_amount_a: u64,
        min_amount_b: u64,
    ) -> Result<()> {
        instructions::remove_liquidity::handle_remove_liquidity(ctx, lp_burn, min_amount_a, min_amount_b)
    }

    pub fn swap(ctx: Context<Swap>, amount_in: u64, minimum_amount_out: u64) -> Result<()> {
        instructions::swap::handle_swap(ctx, amount_in, minimum_amount_out)
    }
}
