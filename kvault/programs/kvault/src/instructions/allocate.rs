use anchor_lang::prelude::*;
use anchor_spl::token::{Token, TokenAccount};

use crate::constants::*;
use crate::errors::KvaultError;
use crate::state::Vault;

#[derive(Accounts)]
pub struct Allocate<'info> {
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

    /// CHECK: PDA signing authority for CPI
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

    /// CHECK: klend obligation (init_if_needed by klend)
    #[account(mut)]
    pub klend_obligation: UncheckedAccount<'info>,

    /// CHECK: klend token vault
    #[account(mut)]
    pub klend_token_vault: UncheckedAccount<'info>,

    /// CHECK: validated by has_one on vault
    pub klend_program: UncheckedAccount<'info>,

    pub token_program: Program<'info, Token>,
    pub system_program: Program<'info, System>,
}

pub fn handle_allocate(ctx: Context<Allocate>, amount: u64) -> Result<()> {
    require!(amount > 0, KvaultError::ZeroAllocate);
    require!(
        amount <= ctx.accounts.vault_token_account.amount,
        KvaultError::InsufficientIdle
    );

    let vault_key = ctx.accounts.vault.key();
    let authority_seeds: &[&[u8]] = &[
        VAULT_AUTHORITY_SEED,
        vault_key.as_ref(),
        &[ctx.accounts.vault.authority_bump],
    ];
    let signer_seeds = [authority_seeds];

    // Build klend deposit CPI
    let cpi_accounts = klend::cpi::accounts::Deposit {
        user: ctx.accounts.vault_authority.to_account_info(),
        lending_market: ctx.accounts.lending_market.to_account_info(),
        reserve: ctx.accounts.klend_reserve.to_account_info(),
        obligation: ctx.accounts.klend_obligation.to_account_info(),
        user_token_account: ctx.accounts.vault_token_account.to_account_info(),
        token_vault: ctx.accounts.klend_token_vault.to_account_info(),
        token_program: ctx.accounts.token_program.to_account_info(),
        system_program: ctx.accounts.system_program.to_account_info(),
    };

    let cpi_ctx = CpiContext::new_with_signer(
        ctx.accounts.klend_program.to_account_info(),
        cpi_accounts,
        &signer_seeds,
    );
    klend::cpi::deposit(cpi_ctx, amount)?;

    // Update cached invested amount
    let vault = &mut ctx.accounts.vault;
    vault.total_invested = vault
        .total_invested
        .checked_add(amount)
        .ok_or(KvaultError::MathOverflow)?;

    Ok(())
}
