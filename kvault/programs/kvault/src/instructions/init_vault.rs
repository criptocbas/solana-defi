use anchor_lang::prelude::*;
use anchor_spl::associated_token::AssociatedToken;
use anchor_spl::token::{Mint, Token, TokenAccount};

use crate::constants::*;
use crate::errors::KvaultError;
use crate::state::Vault;

#[derive(Accounts)]
pub struct InitVault<'info> {
    #[account(mut)]
    pub admin: Signer<'info>,

    pub underlying_mint: Account<'info, Mint>,

    #[account(
        init,
        payer = admin,
        space = 8 + Vault::SPACE,
        seeds = [VAULT_SEED, underlying_mint.key().as_ref()],
        bump,
    )]
    pub vault: Account<'info, Vault>,

    /// CHECK: PDA used as signing authority for transfers and CPI
    #[account(
        mut,
        seeds = [VAULT_AUTHORITY_SEED, vault.key().as_ref()],
        bump,
    )]
    pub vault_authority: UncheckedAccount<'info>,

    #[account(
        init,
        payer = admin,
        mint::decimals = underlying_mint.decimals,
        mint::authority = vault_authority,
        seeds = [SHARE_MINT_SEED, vault.key().as_ref()],
        bump,
    )]
    pub share_mint: Account<'info, Mint>,

    #[account(
        init,
        payer = admin,
        associated_token::mint = underlying_mint,
        associated_token::authority = vault_authority,
    )]
    pub vault_token_account: Account<'info, TokenAccount>,

    /// The klend reserve for the underlying token
    /// CHECK: Stored in vault state, validated during allocate/deallocate
    pub klend_reserve: UncheckedAccount<'info>,

    /// CHECK: klend program ID, stored in vault state
    pub klend_program: UncheckedAccount<'info>,

    pub token_program: Program<'info, Token>,
    pub associated_token_program: Program<'info, AssociatedToken>,
    pub system_program: Program<'info, System>,
}

pub fn handle_init_vault(
    ctx: Context<InitVault>,
    performance_fee_bps: u16,
    management_fee_bps: u16,
    deposit_cap: u64,
) -> Result<()> {
    require!(
        performance_fee_bps <= BPS_SCALE as u16,
        KvaultError::PerformanceFeeExceeded
    );
    require!(
        management_fee_bps <= BPS_SCALE as u16,
        KvaultError::ManagementFeeExceeded
    );

    // Fund vault authority with SOL for klend obligation rent
    let transfer_ix = anchor_lang::solana_program::system_instruction::transfer(
        &ctx.accounts.admin.key(),
        &ctx.accounts.vault_authority.key(),
        AUTHORITY_FUND_LAMPORTS,
    );
    anchor_lang::solana_program::program::invoke(
        &transfer_ix,
        &[
            ctx.accounts.admin.to_account_info(),
            ctx.accounts.vault_authority.to_account_info(),
            ctx.accounts.system_program.to_account_info(),
        ],
    )?;

    let vault = &mut ctx.accounts.vault;
    vault.admin = ctx.accounts.admin.key();
    vault.fee_recipient = ctx.accounts.admin.key();
    vault.underlying_mint = ctx.accounts.underlying_mint.key();
    vault.share_mint = ctx.accounts.share_mint.key();
    vault.vault_authority = ctx.accounts.vault_authority.key();
    vault.vault_token_account = ctx.accounts.vault_token_account.key();
    vault.klend_program = ctx.accounts.klend_program.key();
    vault.klend_reserve = ctx.accounts.klend_reserve.key();
    vault.total_invested = 0;
    vault.last_harvest_timestamp = Clock::get()?.unix_timestamp;
    vault.performance_fee_bps = performance_fee_bps;
    vault.management_fee_bps = management_fee_bps;
    vault.deposit_cap = deposit_cap;
    vault.halted = false;
    vault.vault_bump = ctx.bumps.vault;
    vault.authority_bump = ctx.bumps.vault_authority;
    vault.share_mint_bump = ctx.bumps.share_mint;

    Ok(())
}
