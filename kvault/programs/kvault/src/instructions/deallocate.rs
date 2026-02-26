use anchor_lang::prelude::*;
use anchor_spl::token::{Token, TokenAccount};

use crate::constants::*;
use crate::errors::KvaultError;
use crate::state::Vault;

#[derive(Accounts)]
pub struct Deallocate<'info> {
    #[account(mut)]
    pub admin: Signer<'info>,

    #[account(
        mut,
        seeds = [VAULT_SEED, vault.underlying_mint.as_ref()],
        bump = vault.vault_bump,
        has_one = admin,
        has_one = vault_token_account,
        has_one = klend_program,
        has_one = klend_reserve,
    )]
    pub vault: Account<'info, Vault>,

    /// CHECK: PDA signing authority for CPI (mut required by klend::Withdraw)
    #[account(
        mut,
        seeds = [VAULT_AUTHORITY_SEED, vault.key().as_ref()],
        bump = vault.authority_bump,
    )]
    pub vault_authority: UncheckedAccount<'info>,

    #[account(mut)]
    pub vault_token_account: Account<'info, TokenAccount>,

    // ── klend accounts ──
    /// CHECK: klend lending market
    pub lending_market: UncheckedAccount<'info>,

    /// CHECK: validated by has_one on vault
    #[account(mut)]
    pub klend_reserve: UncheckedAccount<'info>,

    /// CHECK: klend reserve authority PDA
    pub klend_reserve_authority: UncheckedAccount<'info>,

    /// CHECK: klend obligation
    #[account(mut)]
    pub klend_obligation: UncheckedAccount<'info>,

    /// CHECK: klend token vault
    #[account(mut)]
    pub klend_token_vault: UncheckedAccount<'info>,

    /// CHECK: klend oracle for this reserve
    pub klend_oracle: UncheckedAccount<'info>,

    /// CHECK: validated by has_one on vault
    pub klend_program: UncheckedAccount<'info>,

    pub token_program: Program<'info, Token>,
}

pub fn handle_deallocate(ctx: Context<Deallocate>, klend_shares: u64) -> Result<()> {
    require!(klend_shares > 0, KvaultError::ZeroDeallocate);

    // Record balance before CPI
    let balance_before = ctx.accounts.vault_token_account.amount;

    let vault_key = ctx.accounts.vault.key();
    let authority_seeds: &[&[u8]] = &[
        VAULT_AUTHORITY_SEED,
        vault_key.as_ref(),
        &[ctx.accounts.vault.authority_bump],
    ];
    let signer_seeds = [authority_seeds];

    // Build klend withdraw CPI
    let cpi_accounts = klend::cpi::accounts::Withdraw {
        user: ctx.accounts.vault_authority.to_account_info(),
        lending_market: ctx.accounts.lending_market.to_account_info(),
        reserve: ctx.accounts.klend_reserve.to_account_info(),
        reserve_authority: ctx.accounts.klend_reserve_authority.to_account_info(),
        obligation: ctx.accounts.klend_obligation.to_account_info(),
        owner: ctx.accounts.vault_authority.to_account_info(),
        user_token_account: ctx.accounts.vault_token_account.to_account_info(),
        token_vault: ctx.accounts.klend_token_vault.to_account_info(),
        oracle: ctx.accounts.klend_oracle.to_account_info(),
        token_program: ctx.accounts.token_program.to_account_info(),
    };

    let cpi_ctx = CpiContext::new_with_signer(
        ctx.accounts.klend_program.to_account_info(),
        cpi_accounts,
        &signer_seeds,
    );
    klend::cpi::withdraw(cpi_ctx, klend_shares)?;

    // Reload vault token account to measure actual received amount
    ctx.accounts.vault_token_account.reload()?;
    let balance_after = ctx.accounts.vault_token_account.amount;
    let received = balance_after
        .checked_sub(balance_before)
        .ok_or(KvaultError::MathUnderflow)?;

    require!(received > 0, KvaultError::ZeroDeallocate);

    // Update cached invested amount (decrease by actual received)
    let vault = &mut ctx.accounts.vault;
    vault.total_invested = vault
        .total_invested
        .checked_sub(received)
        .ok_or(KvaultError::MathUnderflow)?;

    Ok(())
}
