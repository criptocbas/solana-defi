use anchor_lang::prelude::*;

#[account]
pub struct Vault {
    /// Admin who can allocate/deallocate/harvest/halt
    pub admin: Pubkey,
    /// Fee recipient receives dilutive share tokens
    pub fee_recipient: Pubkey,
    /// Underlying token mint (e.g. USDC)
    pub underlying_mint: Pubkey,
    /// Share token mint (PDA)
    pub share_mint: Pubkey,
    /// Vault authority PDA (signs transfers + CPI)
    pub vault_authority: Pubkey,
    /// Vault's token account for idle underlying
    pub vault_token_account: Pubkey,

    /// klend program ID
    pub klend_program: Pubkey,
    /// klend reserve for the underlying token
    pub klend_reserve: Pubkey,

    /// Cached value of tokens invested in klend (updated on allocate/deallocate/harvest)
    pub total_invested: u64,
    /// Last harvest timestamp (unix seconds)
    pub last_harvest_timestamp: i64,

    /// Performance fee in basis points (applied to yield on harvest)
    pub performance_fee_bps: u16,
    /// Management fee in basis points (annual, applied on harvest)
    pub management_fee_bps: u16,

    /// Maximum total deposits (0 = no cap)
    pub deposit_cap: u64,

    /// Emergency halt flag (blocks deposits, not withdrawals)
    pub halted: bool,

    /// PDA bumps
    pub vault_bump: u8,
    pub authority_bump: u8,
    pub share_mint_bump: u8,
}

impl Vault {
    pub const SPACE: usize = 32  // admin
        + 32  // fee_recipient
        + 32  // underlying_mint
        + 32  // share_mint
        + 32  // vault_authority
        + 32  // vault_token_account
        + 32  // klend_program
        + 32  // klend_reserve
        + 8   // total_invested
        + 8   // last_harvest_timestamp
        + 2   // performance_fee_bps
        + 2   // management_fee_bps
        + 8   // deposit_cap
        + 1   // halted
        + 1   // vault_bump
        + 1   // authority_bump
        + 1;  // share_mint_bump
}
