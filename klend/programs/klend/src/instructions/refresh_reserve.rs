use anchor_lang::prelude::*;

use crate::errors::KlendError;
use crate::math;
use crate::state::{MockOracle, Reserve};

#[derive(Accounts)]
pub struct RefreshReserve<'info> {
    #[account(mut, has_one = oracle)]
    pub reserve: Account<'info, Reserve>,

    pub oracle: Account<'info, MockOracle>,
}

pub fn handle_refresh_reserve(ctx: Context<RefreshReserve>) -> Result<()> {
    let reserve = &mut ctx.accounts.reserve;
    let oracle = &ctx.accounts.oracle;
    let clock = Clock::get()?;

    // Check oracle freshness
    let staleness = clock
        .unix_timestamp
        .saturating_sub(oracle.timestamp) as u64;
    require!(
        staleness <= reserve.config.oracle_max_staleness,
        KlendError::OracleStale
    );

    // Compute elapsed time
    let elapsed = clock
        .unix_timestamp
        .saturating_sub(reserve.last_update_timestamp) as u64;

    if elapsed == 0 || reserve.borrowed_liquidity == 0 {
        reserve.last_update_timestamp = clock.unix_timestamp;
        return Ok(());
    }

    // Compute utilization and borrow rate
    let util = math::utilization_rate(
        reserve.deposited_liquidity,
        reserve.borrowed_liquidity,
        reserve.accumulated_protocol_fees,
    )?;

    let annual_rate = math::borrow_rate(
        util,
        reserve.config.r_base,
        reserve.config.r_slope1,
        reserve.config.r_slope2,
        reserve.config.u_optimal,
    )?;

    // Accrue interest
    let (new_index, interest_accrued, protocol_fee) = math::accrue_interest(
        reserve.cumulative_borrow_index,
        reserve.borrowed_liquidity,
        annual_rate,
        elapsed,
        reserve.config.reserve_factor,
    )?;

    reserve.cumulative_borrow_index = new_index;
    reserve.borrowed_liquidity = reserve
        .borrowed_liquidity
        .checked_add(interest_accrued)
        .ok_or(KlendError::MathOverflow)?;
    reserve.accumulated_protocol_fees = reserve
        .accumulated_protocol_fees
        .checked_add(protocol_fee)
        .ok_or(KlendError::MathOverflow)?;
    reserve.last_update_timestamp = clock.unix_timestamp;

    Ok(())
}
