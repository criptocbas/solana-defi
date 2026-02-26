use crate::constants::*;
use crate::errors::KlendError;
use anchor_lang::prelude::*;

/// Compute utilization rate: borrowed / (deposited + borrowed - fees), scaled by 1e18
pub fn utilization_rate(deposited: u64, borrowed: u64, fees: u64) -> Result<u128> {
    let total = (deposited as u128)
        .checked_add(borrowed as u128)
        .ok_or(KlendError::MathOverflow)?
        .checked_sub(fees as u128)
        .ok_or(KlendError::MathUnderflow)?;
    if total == 0 {
        return Ok(0);
    }
    let util = (borrowed as u128)
        .checked_mul(SCALE)
        .ok_or(KlendError::MathOverflow)?
        / total;
    Ok(util)
}

/// Compute borrow rate (annual, 1e18 scaled) using kinked rate model
pub fn borrow_rate(utilization: u128, r_base: u64, r_slope1: u64, r_slope2: u64, u_optimal: u64) -> Result<u128> {
    let u_opt = u_optimal as u128;
    let base = r_base as u128;
    let slope1 = r_slope1 as u128;
    let slope2 = r_slope2 as u128;

    if utilization <= u_opt {
        // base + (U / U_optimal) * slope1
        if u_opt == 0 {
            return Ok(base);
        }
        let variable = utilization
            .checked_mul(slope1)
            .ok_or(KlendError::MathOverflow)?
            / u_opt;
        Ok(base.checked_add(variable).ok_or(KlendError::MathOverflow)?)
    } else {
        // base + slope1 + ((U - U_optimal) / (1 - U_optimal)) * slope2
        let excess = utilization.checked_sub(u_opt).ok_or(KlendError::MathUnderflow)?;
        let denominator = SCALE.checked_sub(u_opt).ok_or(KlendError::MathUnderflow)?;
        if denominator == 0 {
            return Ok(base
                .checked_add(slope1)
                .ok_or(KlendError::MathOverflow)?
                .checked_add(slope2)
                .ok_or(KlendError::MathOverflow)?);
        }
        let steep = excess
            .checked_mul(slope2)
            .ok_or(KlendError::MathOverflow)?
            / denominator;
        Ok(base
            .checked_add(slope1)
            .ok_or(KlendError::MathOverflow)?
            .checked_add(steep)
            .ok_or(KlendError::MathOverflow)?)
    }
}

/// Compute supply rate = borrow_rate * utilization * (1 - reserve_factor) / SCALE
pub fn supply_rate(borrow_rate: u128, utilization: u128, reserve_factor_bps: u16) -> Result<u128> {
    let after_fee = (BPS_SCALE as u128)
        .checked_sub(reserve_factor_bps as u128)
        .ok_or(KlendError::MathUnderflow)?;
    let rate = borrow_rate
        .checked_mul(utilization)
        .ok_or(KlendError::MathOverflow)?
        / SCALE;
    let rate = rate
        .checked_mul(after_fee)
        .ok_or(KlendError::MathOverflow)?
        / (BPS_SCALE as u128);
    Ok(rate)
}

/// Accrue interest: returns (new_borrow_index, interest_accrued, protocol_fee)
pub fn accrue_interest(
    current_borrow_index: u128,
    borrowed_liquidity: u64,
    annual_borrow_rate: u128,
    elapsed_seconds: u64,
    reserve_factor_bps: u16,
) -> Result<(u128, u64, u64)> {
    if elapsed_seconds == 0 || borrowed_liquidity == 0 {
        return Ok((current_borrow_index, 0, 0));
    }

    // interest_factor = borrow_rate * elapsed / SECONDS_PER_YEAR
    let interest_factor = annual_borrow_rate
        .checked_mul(elapsed_seconds as u128)
        .ok_or(KlendError::MathOverflow)?
        / SECONDS_PER_YEAR;

    // new_index = old_index * (SCALE + interest_factor) / SCALE
    let new_index = current_borrow_index
        .checked_mul(
            SCALE
                .checked_add(interest_factor)
                .ok_or(KlendError::MathOverflow)?,
        )
        .ok_or(KlendError::MathOverflow)?
        / SCALE;

    // interest_accrued = borrowed * (new_index - old_index) / old_index
    let index_delta = new_index
        .checked_sub(current_borrow_index)
        .ok_or(KlendError::MathUnderflow)?;
    let interest_accrued = (borrowed_liquidity as u128)
        .checked_mul(index_delta)
        .ok_or(KlendError::MathOverflow)?
        / current_borrow_index;

    // protocol_fee = interest_accrued * reserve_factor / BPS_SCALE
    let protocol_fee = interest_accrued
        .checked_mul(reserve_factor_bps as u128)
        .ok_or(KlendError::MathOverflow)?
        / (BPS_SCALE as u128);

    Ok((new_index, interest_accrued as u64, protocol_fee as u64))
}

/// Convert underlying tokens to shares (rounds down -- favor protocol)
pub fn underlying_to_shares(amount: u64, total_shares: u64, total_assets: u64) -> Result<u64> {
    let effective_shares = (total_shares as u128)
        .checked_add(VIRTUAL_SHARES)
        .ok_or(KlendError::MathOverflow)?;
    let effective_assets = (total_assets as u128)
        .checked_add(VIRTUAL_ASSETS)
        .ok_or(KlendError::MathOverflow)?;

    let shares = (amount as u128)
        .checked_mul(effective_shares)
        .ok_or(KlendError::MathOverflow)?
        / effective_assets;

    Ok(shares as u64)
}

/// Convert shares to underlying tokens (rounds down -- favor protocol)
pub fn shares_to_underlying(shares: u64, total_shares: u64, total_assets: u64) -> Result<u64> {
    let effective_shares = (total_shares as u128)
        .checked_add(VIRTUAL_SHARES)
        .ok_or(KlendError::MathOverflow)?;
    let effective_assets = (total_assets as u128)
        .checked_add(VIRTUAL_ASSETS)
        .ok_or(KlendError::MathOverflow)?;

    let underlying = (shares as u128)
        .checked_mul(effective_assets)
        .ok_or(KlendError::MathOverflow)?
        / effective_shares;

    Ok(underlying as u64)
}

/// Compute health factor (scaled by 1e18).
/// weighted_collateral_value / total_debt_value
/// Returns u128 scaled by SCALE. HF >= SCALE means healthy.
pub fn health_factor(
    weighted_collateral_value: u128,
    total_debt_value: u128,
) -> Result<u128> {
    if total_debt_value == 0 {
        return Ok(u128::MAX); // no debt = infinitely healthy
    }
    let hf = weighted_collateral_value
        .checked_mul(SCALE)
        .ok_or(KlendError::MathOverflow)?
        / total_debt_value;
    Ok(hf)
}

/// Compute collateral value in USD (scaled by 1e6, matching oracle price scale)
/// value = shares_to_underlying * price / 10^decimals
pub fn collateral_value_usd(
    underlying_amount: u64,
    price: u64,      // USD * 1e6
    decimals: u8,
) -> Result<u128> {
    let value = (underlying_amount as u128)
        .checked_mul(price as u128)
        .ok_or(KlendError::MathOverflow)?
        / 10u128.pow(decimals as u32);
    Ok(value)
}

/// Compute weighted collateral value (apply liquidation_threshold)
pub fn weighted_collateral_value(value_usd: u128, liquidation_threshold_bps: u16) -> Result<u128> {
    let weighted = value_usd
        .checked_mul(liquidation_threshold_bps as u128)
        .ok_or(KlendError::MathOverflow)?
        / (BPS_SCALE as u128);
    Ok(weighted)
}

/// Compute collateral to seize during liquidation
/// collateral_seized = (debt_repaid * debt_price * (BPS + liq_bonus)) / (BPS * collateral_price)
/// All prices in same scale (1e6). Result in collateral token units (adjusted for decimals).
pub fn liquidation_collateral_seized(
    debt_repaid: u64,
    debt_price: u64,
    debt_decimals: u8,
    collateral_price: u64,
    collateral_decimals: u8,
    liquidation_bonus_bps: u16,
) -> Result<u64> {
    let bonus_factor = (BPS_SCALE as u128)
        .checked_add(liquidation_bonus_bps as u128)
        .ok_or(KlendError::MathOverflow)?;

    // debt_value_with_bonus = debt_repaid * debt_price * bonus_factor
    let numerator = (debt_repaid as u128)
        .checked_mul(debt_price as u128)
        .ok_or(KlendError::MathOverflow)?
        .checked_mul(bonus_factor)
        .ok_or(KlendError::MathOverflow)?
        .checked_mul(10u128.pow(collateral_decimals as u32))
        .ok_or(KlendError::MathOverflow)?;

    let denominator = (BPS_SCALE as u128)
        .checked_mul(collateral_price as u128)
        .ok_or(KlendError::DivisionByZero)?
        .checked_mul(10u128.pow(debt_decimals as u32))
        .ok_or(KlendError::MathOverflow)?;

    if denominator == 0 {
        return err!(KlendError::DivisionByZero);
    }

    let seized = numerator / denominator;
    Ok(seized as u64)
}
