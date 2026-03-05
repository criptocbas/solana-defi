pub mod kpool;
pub mod kclmm;

use anchor_lang::prelude::*;

use crate::errors::KaggError;
use crate::types::DexId;

/// Dispatch a swap to the appropriate DEX adapter.
///
/// Account layout per step (from remaining_accounts slice):
///   [0] DEX program
///   [1] pool (mut)
///   [2] pool_authority
///   [3] vault_a (mut)
///   [4] vault_b (mut)
///   [5] input_mint
///   [6..] tick_arrays (kclmm only, mut)
pub fn dispatch_swap<'info>(
    dex_id: DexId,
    step_accounts: &[AccountInfo<'info>],
    user: &AccountInfo<'info>,
    user_token_in: &AccountInfo<'info>,
    user_token_out: &AccountInfo<'info>,
    token_program: &AccountInfo<'info>,
    amount_in: u64,
    extra_data: &[u8],
) -> Result<()> {
    match dex_id {
        DexId::Kpool => kpool::execute_swap(
            step_accounts, user, user_token_in, user_token_out, token_program, amount_in,
        ),
        DexId::Kclmm => {
            let sqrt_price_limit = parse_sqrt_price_limit(extra_data)?;
            kclmm::execute_swap(
                step_accounts, user, user_token_in, user_token_out, token_program,
                amount_in, sqrt_price_limit,
            )
        }
    }
}

fn parse_sqrt_price_limit(extra_data: &[u8]) -> Result<u128> {
    if extra_data.len() >= 16 {
        Ok(u128::from_le_bytes(extra_data[..16].try_into().unwrap()))
    } else if extra_data.is_empty() {
        // Default: 0 means no limit (adapter should use MIN/MAX)
        Ok(0)
    } else {
        Err(error!(KaggError::UnknownDexId))
    }
}
