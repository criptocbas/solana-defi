use anchor_lang::prelude::*;
use anchor_spl::token::{self, Mint, MintTo, Token, TokenAccount, Transfer};

use crate::constants::*;
use crate::errors::KvaultError;
use crate::math;
use crate::state::Vault;

#[derive(Accounts)]
pub struct Deposit<'info> {
    #[account(mut)]
    pub user: Signer<'info>,

    #[account(
        mut,
        seeds = [VAULT_SEED, vault.underlying_mint.as_ref()],
        bump = vault.vault_bump,
    )]
    pub vault: Account<'info, Vault>,

    /// CHECK: PDA signing authority
    #[account(
        seeds = [VAULT_AUTHORITY_SEED, vault.key().as_ref()],
        bump = vault.authority_bump,
    )]
    pub vault_authority: UncheckedAccount<'info>,

    #[account(
        mut,
        address = vault.share_mint,
    )]
    pub share_mint: Account<'info, Mint>,

    #[account(
        mut,
        address = vault.vault_token_account,
    )]
    pub vault_token_account: Account<'info, TokenAccount>,

    #[account(
        mut,
        token::mint = vault.underlying_mint,
        token::authority = user,
    )]
    pub user_token_account: Account<'info, TokenAccount>,

    #[account(
        mut,
        token::mint = vault.share_mint,
        token::authority = user,
    )]
    pub user_share_account: Account<'info, TokenAccount>,

    pub token_program: Program<'info, Token>,
}

pub fn handle_deposit(ctx: Context<Deposit>, amount: u64) -> Result<()> {
    require!(amount > 0, KvaultError::ZeroDeposit);
    require!(!ctx.accounts.vault.halted, KvaultError::VaultHalted);

    let vault = &ctx.accounts.vault;
    let idle = ctx.accounts.vault_token_account.amount;
    let total_assets = idle
        .checked_add(vault.total_invested)
        .ok_or(KvaultError::MathOverflow)?;

    // Check deposit cap
    if vault.deposit_cap > 0 {
        let new_total = total_assets
            .checked_add(amount)
            .ok_or(KvaultError::MathOverflow)?;
        require!(new_total <= vault.deposit_cap, KvaultError::DepositCapExceeded);
    }

    let supply = ctx.accounts.share_mint.supply;
    let shares = math::amount_to_shares(amount, supply, total_assets)?;

    // Transfer underlying from user to vault
    let cpi_accounts = Transfer {
        from: ctx.accounts.user_token_account.to_account_info(),
        to: ctx.accounts.vault_token_account.to_account_info(),
        authority: ctx.accounts.user.to_account_info(),
    };
    let cpi_ctx = CpiContext::new(ctx.accounts.token_program.to_account_info(), cpi_accounts);
    token::transfer(cpi_ctx, amount)?;

    // Mint shares to user
    let vault_key = ctx.accounts.vault.key();
    let authority_seeds: &[&[u8]] = &[
        VAULT_AUTHORITY_SEED,
        vault_key.as_ref(),
        &[ctx.accounts.vault.authority_bump],
    ];
    let signer_seeds = [authority_seeds];

    let cpi_accounts = MintTo {
        mint: ctx.accounts.share_mint.to_account_info(),
        to: ctx.accounts.user_share_account.to_account_info(),
        authority: ctx.accounts.vault_authority.to_account_info(),
    };
    let cpi_ctx = CpiContext::new_with_signer(
        ctx.accounts.token_program.to_account_info(),
        cpi_accounts,
        &signer_seeds,
    );
    token::mint_to(cpi_ctx, shares)?;

    Ok(())
}
