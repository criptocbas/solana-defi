use anchor_lang::prelude::*;

use crate::constants::*;
use crate::errors::KvaultError;

/// Convert underlying amount to share tokens (rounds down -- favor protocol on deposit)
/// shares = amount * (supply + VIRTUAL_SHARES) / (total_assets + VIRTUAL_ASSETS)
pub fn amount_to_shares(amount: u64, supply: u64, total_assets: u64) -> Result<u64> {
    let effective_supply = (supply as u128)
        .checked_add(VIRTUAL_SHARES)
        .ok_or(KvaultError::MathOverflow)?;
    let effective_assets = (total_assets as u128)
        .checked_add(VIRTUAL_ASSETS)
        .ok_or(KvaultError::MathOverflow)?;

    let shares = (amount as u128)
        .checked_mul(effective_supply)
        .ok_or(KvaultError::MathOverflow)?
        / effective_assets;

    Ok(shares as u64)
}

/// Convert share tokens to underlying amount (rounds down -- favor protocol on withdraw)
/// amount = shares * (total_assets + VIRTUAL_ASSETS) / (supply + VIRTUAL_SHARES)
pub fn shares_to_amount(shares: u64, supply: u64, total_assets: u64) -> Result<u64> {
    let effective_supply = (supply as u128)
        .checked_add(VIRTUAL_SHARES)
        .ok_or(KvaultError::MathOverflow)?;
    let effective_assets = (total_assets as u128)
        .checked_add(VIRTUAL_ASSETS)
        .ok_or(KvaultError::MathOverflow)?;

    let amount = (shares as u128)
        .checked_mul(effective_assets)
        .ok_or(KvaultError::MathOverflow)?
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
        .ok_or(KvaultError::MathOverflow)?;
    let effective_assets = (total_assets as u128)
        .checked_add(VIRTUAL_ASSETS)
        .ok_or(KvaultError::MathOverflow)?;

    let denominator = effective_assets
        .checked_sub(fee_underlying as u128)
        .ok_or(KvaultError::MathUnderflow)?;

    if denominator == 0 {
        return err!(KvaultError::DivisionByZero);
    }

    let shares = (fee_underlying as u128)
        .checked_mul(effective_supply)
        .ok_or(KvaultError::MathOverflow)?
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
        .ok_or(KvaultError::MathOverflow)?;
    let effective_assets = (reserve_total_assets as u128)
        .checked_add(1)
        .ok_or(KvaultError::MathOverflow)?;

    let underlying = (klend_shares as u128)
        .checked_mul(effective_assets)
        .ok_or(KvaultError::MathOverflow)?
        / effective_shares;

    Ok(underlying as u64)
}
