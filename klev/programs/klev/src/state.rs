use anchor_lang::prelude::*;

#[account]
pub struct LeveragedVault {
    /// Admin who can leverage/deleverage/harvest/halt
    pub admin: Pubkey,
    /// Fee recipient receives dilutive share tokens
    pub fee_recipient: Pubkey,

    /// Collateral token mint (e.g. SOL)
    pub collateral_mint: Pubkey,
    /// Debt token mint (e.g. USDC)
    pub debt_mint: Pubkey,

    /// Share token mint (PDA)
    pub share_mint: Pubkey,
    /// Vault authority PDA (signs transfers + CPI)
    pub vault_authority: Pubkey,

    /// Vault's collateral token account (idle SOL)
    pub collateral_token_account: Pubkey,
    /// Vault's debt token account (intermediate USDC)
    pub debt_token_account: Pubkey,

    // ── klend references ──
    pub klend_program: Pubkey,
    pub klend_lending_market: Pubkey,
    pub klend_collateral_reserve: Pubkey,
    pub klend_debt_reserve: Pubkey,

    // ── cpamm references ──
    pub cpamm_program: Pubkey,
    pub cpamm_pool: Pubkey,

    // ── cached state (updated at harvest / leverage / deleverage) ──
    /// SOL deposited in klend (underlying value, not klend shares)
    pub cached_collateral_value: u64,
    /// USDC debt in klend (current debt value)
    pub cached_debt_value: u64,
    /// Net equity in SOL terms: collateral - debt_in_sol
    pub cached_net_equity_collateral: u64,

    /// Last harvest timestamp (unix seconds)
    pub last_harvest_timestamp: i64,

    /// Performance fee in basis points (applied to yield on harvest)
    pub performance_fee_bps: u16,
    /// Management fee in basis points (annual, applied on harvest)
    pub management_fee_bps: u16,
    /// Maximum leverage in basis points (30000 = 3x)
    pub max_leverage_bps: u16,
    /// Minimum health factor in basis points (11000 = 1.1)
    pub min_health_factor_bps: u16,

    /// Maximum total deposits (0 = no cap)
    pub deposit_cap: u64,

    /// Emergency halt flag (blocks deposits, not withdrawals)
    pub halted: bool,

    /// PDA bumps
    pub vault_bump: u8,
    pub authority_bump: u8,
    pub share_mint_bump: u8,
}

impl LeveragedVault {
    pub const SPACE: usize = 32  // admin
        + 32  // fee_recipient
        + 32  // collateral_mint
        + 32  // debt_mint
        + 32  // share_mint
        + 32  // vault_authority
        + 32  // collateral_token_account
        + 32  // debt_token_account
        + 32  // klend_program
        + 32  // klend_lending_market
        + 32  // klend_collateral_reserve
        + 32  // klend_debt_reserve
        + 32  // cpamm_program
        + 32  // cpamm_pool
        + 8   // cached_collateral_value
        + 8   // cached_debt_value
        + 8   // cached_net_equity_collateral
        + 8   // last_harvest_timestamp
        + 2   // performance_fee_bps
        + 2   // management_fee_bps
        + 2   // max_leverage_bps
        + 2   // min_health_factor_bps
        + 8   // deposit_cap
        + 1   // halted
        + 1   // vault_bump
        + 1   // authority_bump
        + 1;  // share_mint_bump
}
