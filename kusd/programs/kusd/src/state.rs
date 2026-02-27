use anchor_lang::prelude::*;

/// CDP vault — one per collateral type
#[account]
pub struct CdpVault {
    pub admin: Pubkey,
    pub collateral_mint: Pubkey,
    pub kusd_mint: Pubkey,
    pub collateral_token_account: Pubkey,
    pub vault_authority: Pubkey,
    pub oracle: Pubkey,

    // Configuration (basis points)
    pub max_ltv_bps: u16,
    pub liquidation_threshold_bps: u16,
    pub liquidation_bonus_bps: u16,
    pub stability_fee_bps: u16,

    pub oracle_max_staleness: u64,
    pub debt_ceiling: u64, // max kUSD mintable (0 = no cap)

    // Accounting
    pub total_collateral: u64,
    pub total_debt_shares: u128,
    pub cumulative_fee_index: u128, // 1e18 scaled, starts at SCALE
    pub last_update_timestamp: i64,

    pub collateral_decimals: u8,
    pub halted: bool,

    pub vault_bump: u8,
    pub authority_bump: u8,
    pub kusd_mint_bump: u8,
}

impl CdpVault {
    pub const SPACE: usize = 32 * 6 // 6 pubkeys
        + 2 * 4                      // 4 u16 fields
        + 8 * 2                      // oracle_max_staleness + debt_ceiling
        + 8                          // total_collateral
        + 16 * 2                     // total_debt_shares + cumulative_fee_index
        + 8                          // last_update_timestamp
        + 1 * 5;                     // 5 u8/bool fields
}

/// Per-user CDP position
#[account]
pub struct CdpPosition {
    pub vault: Pubkey,
    pub owner: Pubkey,
    pub collateral_amount: u64,
    pub debt_shares: u128,
    pub bump: u8,
}

impl CdpPosition {
    pub const SPACE: usize = 32 * 2 // vault + owner
        + 8                          // collateral_amount
        + 16                         // debt_shares
        + 1;                         // bump
}

/// Mock oracle for collateral pricing
#[account]
pub struct MockOracle {
    pub token_mint: Pubkey,
    pub price: u64,    // USD per token * 1e6
    pub decimals: u8,  // token decimals
    pub timestamp: i64,
    pub bump: u8,
}
