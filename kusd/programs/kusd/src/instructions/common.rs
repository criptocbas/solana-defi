use anchor_lang::prelude::*;

use crate::math;
use crate::state::CdpVault;

/// Accrue stability fees on the vault. Called by mint, repay, withdraw, liquidate, accrue_fees.
pub fn accrue_vault_fees(vault: &mut CdpVault, now: i64) -> Result<()> {
    let elapsed = now.saturating_sub(vault.last_update_timestamp) as u64;
    if elapsed == 0 {
        return Ok(());
    }

    vault.cumulative_fee_index = math::accrue_index(
        vault.cumulative_fee_index,
        vault.stability_fee_bps,
        elapsed,
    )?;
    vault.last_update_timestamp = now;

    Ok(())
}
