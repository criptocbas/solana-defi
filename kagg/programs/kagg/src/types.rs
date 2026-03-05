use anchor_lang::prelude::*;

/// Identifies which DEX adapter to use for a swap step.
#[derive(AnchorSerialize, AnchorDeserialize, Clone, Copy, PartialEq, Eq)]
#[repr(u8)]
pub enum DexId {
    Kpool = 0,
    Kclmm = 1,
}

/// One step in a route plan. Steps are processed sequentially.
///
/// For multi-hop routes, each step's output becomes the next step's input.
/// For splits, multiple steps share the same input token but use explicit amount_in.
#[derive(AnchorSerialize, AnchorDeserialize, Clone)]
pub struct RoutePlanStep {
    /// Which DEX to route through
    pub dex_id: DexId,
    /// Number of accounts this step consumes from remaining_accounts
    /// kpool = 6, kclmm = 6 + N tick arrays
    pub num_accounts: u8,
    /// Input amount for this step.
    /// 0 = use the full output of the previous step (for sequential hops)
    pub amount_in: u64,
    /// Index into remaining_accounts for this step's input token account.
    /// For the first step, this is typically the user's source token.
    pub input_token_index: u8,
    /// Index into remaining_accounts for this step's output token account.
    /// For the last step, this is typically the user's destination token.
    pub output_token_index: u8,
    /// Adapter-specific extra data (e.g. sqrt_price_limit for CLMM as little-endian u128)
    pub extra_data: Vec<u8>,
}
