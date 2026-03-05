mod cpamm_pool;
mod clmm_pool;

pub use cpamm_pool::CpammPool;
pub use clmm_pool::{ClmmPool, TickArrayData, TickData};

use kagg::types::DexId;
use solana_sdk::pubkey::Pubkey;

/// Trait for pools that can provide off-chain swap quotes.
pub trait QuotablePool {
    /// Compute output amount for a given input. Returns None if swap is impossible.
    fn quote(&self, amount_in: u64, a_to_b: bool) -> Option<u64>;
    fn mint_a(&self) -> Pubkey;
    fn mint_b(&self) -> Pubkey;
    fn address(&self) -> Pubkey;
    fn dex_id(&self) -> DexId;
    /// Returns (key, is_signer, is_writable) for each account needed by the on-chain swap.
    fn swap_accounts(&self, a_to_b: bool) -> Vec<(Pubkey, bool, bool)>;
    /// Adapter-specific extra data (e.g. sqrt_price_limit for CLMM).
    fn extra_data(&self, a_to_b: bool, amount_in: u64) -> Vec<u8>;
    /// Number of accounts this step consumes from remaining_accounts.
    fn num_accounts(&self, a_to_b: bool) -> u8;
}
