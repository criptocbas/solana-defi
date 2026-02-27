use anchor_lang::prelude::*;
use anchor_spl::token::{self, Burn, Mint, Token, TokenAccount, Transfer};

use crate::constants::*;
use crate::errors::KlevError;
use crate::math;
use crate::state::LeveragedVault;

#[derive(Accounts)]
pub struct Withdraw<'info> {
    #[account(mut)]
    pub user: Signer<'info>,

    #[account(
        mut,
        seeds = [LEVERAGED_VAULT_SEED, vault.collateral_mint.as_ref(), vault.debt_mint.as_ref()],
        bump = vault.vault_bump,
    )]
    pub vault: Account<'info, LeveragedVault>,

    /// CHECK: PDA signing authority
    #[account(
        seeds = [LEV_VAULT_AUTHORITY_SEED, vault.key().as_ref()],
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
        address = vault.collateral_token_account,
    )]
    pub collateral_token_account: Account<'info, TokenAccount>,

    #[account(
        mut,
        token::mint = vault.collateral_mint,
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

pub fn handle_withdraw(ctx: Context<Withdraw>, shares: u64) -> Result<()> {
    require!(shares > 0, KlevError::ZeroWithdraw);
    // NOTE: withdrawals work even when halted (user safety)

    let vault = &ctx.accounts.vault;
    let idle = ctx.accounts.collateral_token_account.amount;
    let total_assets = idle
        .checked_add(vault.cached_net_equity_collateral)
        .ok_or(KlevError::MathOverflow)?;

    let supply = ctx.accounts.share_mint.supply;
    let amount = math::shares_to_amount(shares, supply, total_assets)?;

    // Check sufficient idle balance
    require!(amount <= idle, KlevError::InsufficientIdle);

    // Burn user's shares
    let cpi_accounts = Burn {
        mint: ctx.accounts.share_mint.to_account_info(),
        from: ctx.accounts.user_share_account.to_account_info(),
        authority: ctx.accounts.user.to_account_info(),
    };
    let cpi_ctx = CpiContext::new(ctx.accounts.token_program.to_account_info(), cpi_accounts);
    token::burn(cpi_ctx, shares)?;

    // Transfer collateral from vault to user
    let vault_key = ctx.accounts.vault.key();
    let authority_seeds: &[&[u8]] = &[
        LEV_VAULT_AUTHORITY_SEED,
        vault_key.as_ref(),
        &[vault.authority_bump],
    ];
    let signer_seeds = [authority_seeds];

    let cpi_accounts = Transfer {
        from: ctx.accounts.collateral_token_account.to_account_info(),
        to: ctx.accounts.user_token_account.to_account_info(),
        authority: ctx.accounts.vault_authority.to_account_info(),
    };
    let cpi_ctx = CpiContext::new_with_signer(
        ctx.accounts.token_program.to_account_info(),
        cpi_accounts,
        &signer_seeds,
    );
    token::transfer(cpi_ctx, amount)?;

    Ok(())
}
