use anchor_lang::prelude::*;

#[derive(AnchorSerialize, AnchorDeserialize, Clone, Copy, PartialEq, Eq)]
pub enum PoolType {
    Kpool,
    Kclmm,
}

/// Describes one leg of a multi-hop or split route.
#[derive(AnchorSerialize, AnchorDeserialize, Clone, Copy)]
pub struct LegDescriptor {
    pub pool_type: PoolType,
    /// Number of accounts in remaining_accounts for this leg
    /// kpool = 6, kclmm = 6 + N tick arrays
    pub num_accounts: u8,
    /// Only used for kclmm legs; 0 means use default limit
    pub sqrt_price_limit: u128,
}

/// Describes one leg in a split route (same pair, two pools).
#[derive(AnchorSerialize, AnchorDeserialize, Clone, Copy)]
pub struct SplitLegDescriptor {
    pub pool_type: PoolType,
    pub num_accounts: u8,
    pub amount_in: u64,
    pub sqrt_price_limit: u128,
}
