use anchor_lang::prelude::*;

use crate::constants::*;
use crate::errors::KlevError;

/// Convert underlying amount to share tokens (rounds down -- favor protocol on deposit)
/// shares = amount * (supply + VIRTUAL_SHARES) / (total_assets + VIRTUAL_ASSETS)
pub fn amount_to_shares(amount: u64, supply: u64, total_assets: u64) -> Result<u64> {
    let effective_supply = (supply as u128)
        .checked_add(VIRTUAL_SHARES)
        .ok_or(KlevError::MathOverflow)?;
    let effective_assets = (total_assets as u128)
        .checked_add(VIRTUAL_ASSETS)
        .ok_or(KlevError::MathOverflow)?;

    let shares = (amount as u128)
        .checked_mul(effective_supply)
        .ok_or(KlevError::MathOverflow)?
        / effective_assets;

    Ok(shares as u64)
}

/// Convert share tokens to underlying amount (rounds down -- favor protocol on withdraw)
/// amount = shares * (total_assets + VIRTUAL_ASSETS) / (supply + VIRTUAL_SHARES)
pub fn shares_to_amount(shares: u64, supply: u64, total_assets: u64) -> Result<u64> {
    let effective_supply = (supply as u128)
        .checked_add(VIRTUAL_SHARES)
        .ok_or(KlevError::MathOverflow)?;
    let effective_assets = (total_assets as u128)
        .checked_add(VIRTUAL_ASSETS)
        .ok_or(KlevError::MathOverflow)?;

    let amount = (shares as u128)
        .checked_mul(effective_assets)
        .ok_or(KlevError::MathOverflow)?
        / effective_supply;

    Ok(amount as u64)
}

/// Compute dilutive fee shares to mint (Yearn V3 pattern)
/// fee_shares = fee_underlying * (supply + VIRTUAL_SHARES) / (total_assets + VIRTUAL_ASSETS - fee_underlying)
pub fn fee_shares(fee_underlying: u64, supply: u64, total_assets: u64) -> Result<u64> {
    if fee_underlying == 0 {
        return Ok(0);
    }

    let effective_supply = (supply as u128)
        .checked_add(VIRTUAL_SHARES)
        .ok_or(KlevError::MathOverflow)?;
    let effective_assets = (total_assets as u128)
        .checked_add(VIRTUAL_ASSETS)
        .ok_or(KlevError::MathOverflow)?;

    let denominator = effective_assets
        .checked_sub(fee_underlying as u128)
        .ok_or(KlevError::MathUnderflow)?;

    if denominator == 0 {
        return err!(KlevError::DivisionByZero);
    }

    let shares = (fee_underlying as u128)
        .checked_mul(effective_supply)
        .ok_or(KlevError::MathOverflow)?
        / denominator;

    Ok(shares as u64)
}

/// Compute underlying value from klend reserve shares
/// underlying = klend_shares * (reserve_total_assets + 1) / (reserve_total_shares + 1)
pub fn klend_shares_to_underlying(
    klend_shares: u64,
    reserve_total_shares: u64,
    reserve_total_assets: u64,
) -> Result<u64> {
    let effective_shares = (reserve_total_shares as u128)
        .checked_add(1)
        .ok_or(KlevError::MathOverflow)?;
    let effective_assets = (reserve_total_assets as u128)
        .checked_add(1)
        .ok_or(KlevError::MathOverflow)?;

    let underlying = (klend_shares as u128)
        .checked_mul(effective_assets)
        .ok_or(KlevError::MathOverflow)?
        / effective_shares;

    Ok(underlying as u64)
}

/// Convert debt amount from debt-token terms to collateral-token terms using oracle prices.
/// debt_in_collateral = debt_amount * debt_price * 10^collateral_decimals / (collateral_price * 10^debt_decimals)
///
/// Prices are in USD * 1e6 (ORACLE_PRICE_SCALE).
pub fn debt_to_collateral_terms(
    debt_amount: u64,
    debt_price: u64,
    debt_decimals: u8,
    collateral_price: u64,
    collateral_decimals: u8,
) -> Result<u64> {
    if collateral_price == 0 {
        return err!(KlevError::DivisionByZero);
    }

    // debt_value_usd = debt_amount * debt_price (both in native units * price_scale)
    let numerator = (debt_amount as u128)
        .checked_mul(debt_price as u128)
        .ok_or(KlevError::MathOverflow)?
        .checked_mul(10u128.pow(collateral_decimals as u32))
        .ok_or(KlevError::MathOverflow)?;

    let denominator = (collateral_price as u128)
        .checked_mul(10u128.pow(debt_decimals as u32))
        .ok_or(KlevError::MathOverflow)?;

    let result = numerator / denominator;
    Ok(result as u64)
}

/// Compute net equity in collateral terms.
/// net_equity = collateral_value - debt_in_collateral_terms
pub fn net_equity(collateral_value: u64, debt_in_collateral: u64) -> u64 {
    collateral_value.saturating_sub(debt_in_collateral)
}

/// Compute leverage ratio in basis points.
/// leverage = total_collateral * BPS_SCALE / net_equity
/// e.g. 2x = 20000 bps, 3x = 30000 bps
pub fn leverage_ratio_bps(total_collateral: u64, net_equity_val: u64) -> Result<u64> {
    if net_equity_val == 0 {
        return err!(KlevError::DivisionByZero);
    }

    let ratio = (total_collateral as u128)
        .checked_mul(BPS_SCALE as u128)
        .ok_or(KlevError::MathOverflow)?
        / (net_equity_val as u128);

    Ok(ratio as u64)
}

/// Compute current debt from klend obligation borrow entry.
/// current_debt = borrowed_amount_scaled * current_borrow_index / KLEND_SCALE (rounded up)
pub fn klend_current_debt(borrowed_amount_scaled: u128, current_borrow_index: u128) -> Result<u64> {
    if current_borrow_index == 0 {
        return Ok(0);
    }

    let product = borrowed_amount_scaled
        .checked_mul(current_borrow_index)
        .ok_or(KlevError::MathOverflow)?;
    let debt = product / KLEND_SCALE;
    let remainder = product % KLEND_SCALE;
    // Round up to favor protocol
    let rounded = if remainder > 0 { debt + 1 } else { debt };

    Ok(rounded as u64)
}
