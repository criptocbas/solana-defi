use anchor_lang::prelude::*;

#[account]
#[derive(InitSpace)]
pub struct Pool {
    pub mint_a: Pubkey,
    pub mint_b: Pubkey,
    pub vault_a: Pubkey,
    pub vault_b: Pubkey,
    pub lp_mint: Pubkey,
    pub pool_authority: Pubkey,
    pub reserve_a: u64,
    pub reserve_b: u64,
    pub pool_bump: u8,
    pub authority_bump: u8,
    pub lp_mint_bump: u8,
}
