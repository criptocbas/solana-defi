use anchor_lang::prelude::*;

pub mod cpi_helpers;
pub mod errors;
pub mod instructions;
pub mod types;

use instructions::*;
use types::*;

declare_id!("hJ69REU7iZLsWzT1Bvw5w8Pe8Yz5kBR6dA42AczRj9Y");

#[program]
pub mod krouter {
    use super::*;

    /// Direct swap through kpool (constant product AMM)
    pub fn swap_kpool(
        ctx: Context<SwapKpool>,
        amount_in: u64,
        minimum_amount_out: u64,
    ) -> Result<()> {
        instructions::swap_kpool::handle_swap_kpool(ctx, amount_in, minimum_amount_out)
    }

    /// Direct swap through kclmm (concentrated liquidity AMM)
    pub fn swap_kclmm<'info>(
        ctx: Context<'_, '_, 'info, 'info, SwapKclmm<'info>>,
        amount_in: u64,
        sqrt_price_limit: u128,
        minimum_amount_out: u64,
    ) -> Result<()> {
        instructions::swap_kclmm::handle_swap_kclmm(ctx, amount_in, sqrt_price_limit, minimum_amount_out)
    }

    /// Two-hop route: A -> B -> C through two pools (any combination of kpool/kclmm)
    pub fn route_two_hop<'info>(
        ctx: Context<'_, '_, 'info, 'info, RouteTwoHop<'info>>,
        amount_in: u64,
        minimum_amount_out: u64,
        leg1: LegDescriptor,
        leg2: LegDescriptor,
    ) -> Result<()> {
        instructions::route_two_hop::handle_route_two_hop(ctx, amount_in, minimum_amount_out, leg1, leg2)
    }

    /// Split route: divide input across two pools for the same pair
    pub fn route_split<'info>(
        ctx: Context<'_, '_, 'info, 'info, RouteSplit<'info>>,
        total_amount_in: u64,
        minimum_amount_out: u64,
        leg1: SplitLegDescriptor,
        leg2: SplitLegDescriptor,
    ) -> Result<()> {
        instructions::route_split::handle_route_split(ctx, total_amount_in, minimum_amount_out, leg1, leg2)
    }
}
