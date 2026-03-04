use anchor_lang::prelude::*;

use crate::constants::*;
use crate::errors::KlendError;
use crate::math;
use crate::state::{MockOracle, Obligation, Reserve};

/// Whether to weight collateral by LTV (for borrows) or liquidation threshold (for liquidation/withdraw).
#[derive(Clone, Copy, PartialEq, Eq)]
pub enum WeightMode {
    Ltv,
    LiquidationThreshold,
}

/// Compute full obligation health across ALL positions using remaining_accounts.
///
/// `remaining_accounts` layout: `[reserve_0, oracle_0, reserve_1, oracle_1, ...]`
/// Must contain a (reserve, oracle) pair for every active deposit AND every active borrow.
///
/// Returns `(weighted_collateral, total_debt, health_factor)` — all u128, SCALE-denominated.
pub fn compute_obligation_health(
    obligation: &Obligation,
    remaining_accounts: &[AccountInfo],
    lending_market_key: &Pubkey,
    clock: &Clock,
    weight_mode: WeightMode,
) -> Result<(u128, u128, u128)> {
    require!(
        remaining_accounts.len() % 2 == 0,
        KlendError::MissingPositionAccounts
    );

    let mut weighted_collateral: u128 = 0;
    let mut total_debt: u128 = 0;

    // Sum collateral across all deposits
    for dep in &obligation.deposits {
        if dep.shares == 0 {
            continue;
        }
        let (reserve, oracle) = find_and_validate_pair(
            remaining_accounts,
            &dep.reserve,
            lending_market_key,
            clock,
        )?;

        let underlying = math::shares_to_underlying(
            dep.shares,
            reserve.total_shares,
            reserve.total_assets(),
        )?;

        let value_usd = math::collateral_value_usd(
            underlying,
            oracle.price,
            oracle.decimals,
        )?;

        let weight_bps = match weight_mode {
            WeightMode::Ltv => reserve.config.ltv,
            WeightMode::LiquidationThreshold => reserve.config.liquidation_threshold,
        };

        let weighted = math::weighted_collateral_value(value_usd, weight_bps)?;
        weighted_collateral = weighted_collateral
            .checked_add(weighted)
            .ok_or(KlendError::MathOverflow)?;
    }

    // Sum debt across all borrows
    for borrow_entry in &obligation.borrows {
        if borrow_entry.borrowed_amount_scaled == 0 {
            continue;
        }
        let (reserve, oracle) = find_and_validate_pair(
            remaining_accounts,
            &borrow_entry.reserve,
            lending_market_key,
            clock,
        )?;

        let current_debt = borrow_entry.current_debt(reserve.cumulative_borrow_index)?;
        let debt_usd = math::collateral_value_usd(
            current_debt,
            oracle.price,
            oracle.decimals,
        )?;

        total_debt = total_debt
            .checked_add(debt_usd)
            .ok_or(KlendError::MathOverflow)?;
    }

    let hf = math::health_factor(weighted_collateral, total_debt)?;
    Ok((weighted_collateral, total_debt, hf))
}

/// Find and validate a (Reserve, MockOracle) pair in remaining_accounts for a given reserve key.
/// Validates PDA derivation, oracle match, mint match, and staleness.
#[inline(never)]
fn find_and_validate_pair<'a>(
    remaining_accounts: &'a [AccountInfo],
    reserve_key: &Pubkey,
    lending_market_key: &Pubkey,
    clock: &Clock,
) -> Result<(Reserve, MockOracle)> {
    // Search pairs: (remaining_accounts[0], remaining_accounts[1]), (remaining_accounts[2], remaining_accounts[3]), ...
    let mut found = None;
    for chunk in remaining_accounts.chunks(2) {
        if chunk.len() < 2 {
            break;
        }
        if chunk[0].key() == *reserve_key {
            found = Some((&chunk[0], &chunk[1]));
            break;
        }
    }

    let (reserve_ai, oracle_ai) = found.ok_or(KlendError::MissingPositionAccounts)?;

    // Deserialize
    let reserve_data = reserve_ai.try_borrow_data()?;
    let reserve = Reserve::try_deserialize(&mut &reserve_data[..])?;
    drop(reserve_data);

    let oracle_data = oracle_ai.try_borrow_data()?;
    let oracle = MockOracle::try_deserialize(&mut &oracle_data[..])?;
    drop(oracle_data);

    // Validate reserve PDA
    let (expected_reserve_pda, _) = Pubkey::find_program_address(
        &[RESERVE_SEED, lending_market_key.as_ref(), reserve.token_mint.as_ref()],
        &crate::ID,
    );
    require!(
        reserve_ai.key() == expected_reserve_pda,
        KlendError::InvalidReserveAccount
    );

    // Validate oracle matches reserve
    require!(
        reserve.oracle == oracle_ai.key(),
        KlendError::InvalidOracle
    );

    // Validate oracle mint matches reserve mint
    require!(
        oracle.token_mint == reserve.token_mint,
        KlendError::InvalidOracle
    );

    // Oracle staleness
    let staleness = clock.unix_timestamp.saturating_sub(oracle.timestamp) as u64;
    require!(
        staleness <= reserve.config.oracle_max_staleness,
        KlendError::OracleStale
    );

    Ok((reserve, oracle))
}
