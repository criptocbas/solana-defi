use crate::constants::*;
use crate::errors::KusdError;
use anchor_lang::prelude::*;

/// Accrue fee index: new = old + old * fee_bps * elapsed / (BPS * SECONDS_PER_YEAR)
pub fn accrue_index(old_index: u128, fee_bps: u16, elapsed_seconds: u64) -> Result<u128> {
    if elapsed_seconds == 0 || fee_bps == 0 {
        return Ok(old_index);
    }

    let interest = old_index
        .checked_mul(fee_bps as u128)
        .ok_or(KusdError::MathOverflow)?
        .checked_mul(elapsed_seconds as u128)
        .ok_or(KusdError::MathOverflow)?
        / (BPS_SCALE as u128)
        / SECONDS_PER_YEAR;

    old_index
        .checked_add(interest)
        .ok_or(KusdError::MathOverflow.into())
}

/// Convert kUSD amount to debt shares (rounds down — favors protocol on mint)
pub fn amount_to_shares(amount: u64, fee_index: u128) -> Result<u128> {
    if fee_index == 0 {
        return err!(KusdError::DivisionByZero);
    }
    let shares = (amount as u128)
        .checked_mul(SCALE)
        .ok_or(KusdError::MathOverflow)?
        / fee_index;
    Ok(shares)
}

/// Convert debt shares to kUSD debt (rounds up — favors protocol on repay)
pub fn shares_to_debt(shares: u128, fee_index: u128) -> Result<u64> {
    let product = shares
        .checked_mul(fee_index)
        .ok_or(KusdError::MathOverflow)?;
    let debt = product / SCALE;
    let remainder = product % SCALE;
    let rounded = if remainder > 0 { debt + 1 } else { debt };
    Ok(rounded as u64)
}

/// Collateral value in USD (scaled by PRICE_SCALE = 1e6)
/// value = amount * price / 10^decimals
pub fn collateral_value_usd(amount: u64, price: u64, decimals: u8) -> Result<u128> {
    let value = (amount as u128)
        .checked_mul(price as u128)
        .ok_or(KusdError::MathOverflow)?
        / 10u128.pow(decimals as u32);
    Ok(value)
}

/// Health factor (1e18 scaled): coll_usd * liq_threshold / (BPS * debt_usd)
/// HF >= SCALE means healthy
pub fn health_factor(coll_usd: u128, debt_usd: u128, liq_threshold_bps: u16) -> Result<u128> {
    if debt_usd == 0 {
        return Ok(u128::MAX); // no debt = infinitely healthy
    }
    let hf = coll_usd
        .checked_mul(liq_threshold_bps as u128)
        .ok_or(KusdError::MathOverflow)?
        .checked_mul(SCALE)
        .ok_or(KusdError::MathOverflow)?
        / (BPS_SCALE as u128)
        / debt_usd;
    Ok(hf)
}

/// Max additional kUSD mintable given current collateral and debt
pub fn max_mintable(coll_usd: u128, debt_usd: u128, max_ltv_bps: u16) -> Result<u128> {
    let max_debt = coll_usd
        .checked_mul(max_ltv_bps as u128)
        .ok_or(KusdError::MathOverflow)?
        / (BPS_SCALE as u128);
    Ok(max_debt.saturating_sub(debt_usd))
}

/// Collateral to seize during liquidation
/// Since kUSD = $1 with 6 decimals and PRICE_SCALE = 1e6:
/// seized = repay_amount * (BPS + bonus) * 10^coll_dec / (BPS * coll_price)
pub fn liquidation_collateral_seized(
    repay_amount: u64,
    collateral_price: u64,
    collateral_decimals: u8,
    bonus_bps: u16,
) -> Result<u64> {
    if collateral_price == 0 {
        return err!(KusdError::DivisionByZero);
    }
    let bonus_factor = (BPS_SCALE as u128)
        .checked_add(bonus_bps as u128)
        .ok_or(KusdError::MathOverflow)?;

    let numerator = (repay_amount as u128)
        .checked_mul(bonus_factor)
        .ok_or(KusdError::MathOverflow)?
        .checked_mul(10u128.pow(collateral_decimals as u32))
        .ok_or(KusdError::MathOverflow)?;

    let denominator = (BPS_SCALE as u128)
        .checked_mul(collateral_price as u128)
        .ok_or(KusdError::MathOverflow)?;

    let seized = numerator / denominator;
    Ok(seized as u64)
}

/// Collateral ratio in basis points (for display): coll_usd * BPS / debt_usd
pub fn collateral_ratio_bps(coll_usd: u128, debt_usd: u128) -> Result<u128> {
    if debt_usd == 0 {
        return Ok(u128::MAX);
    }
    let ratio = coll_usd
        .checked_mul(BPS_SCALE as u128)
        .ok_or(KusdError::MathOverflow)?
        / debt_usd;
    Ok(ratio)
}
