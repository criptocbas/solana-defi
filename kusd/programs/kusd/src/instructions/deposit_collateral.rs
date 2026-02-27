use anchor_lang::prelude::*;
use anchor_spl::token::{self, Token, TokenAccount, Transfer};

use crate::constants::*;
use crate::errors::KusdError;
use crate::state::{CdpPosition, CdpVault};

#[derive(Accounts)]
pub struct DepositCollateral<'info> {
    #[account(mut)]
    pub depositor: Signer<'info>,

    #[account(
        mut,
        seeds = [CDP_VAULT_SEED, vault.collateral_mint.as_ref()],
        bump = vault.vault_bump,
    )]
    pub vault: Account<'info, CdpVault>,

    #[account(
        mut,
        seeds = [CDP_POSITION_SEED, vault.key().as_ref(), depositor.key().as_ref()],
        bump = position.bump,
        has_one = vault,
        has_one = owner,
    )]
    pub position: Account<'info, CdpPosition>,

    /// CHECK: position owner, must match depositor
    #[account(constraint = owner.key() == depositor.key())]
    pub owner: AccountInfo<'info>,

    /// Depositor's collateral token account
    #[account(mut)]
    pub depositor_collateral: Account<'info, TokenAccount>,

    /// Vault's collateral token account
    #[account(
        mut,
        constraint = collateral_vault.key() == vault.collateral_token_account,
    )]
    pub collateral_vault: Account<'info, TokenAccount>,

    pub token_program: Program<'info, Token>,
}

pub fn handle_deposit_collateral(ctx: Context<DepositCollateral>, amount: u64) -> Result<()> {
    require!(amount > 0, KusdError::ZeroDeposit);

    // Transfer collateral from depositor to vault
    let cpi_accounts = Transfer {
        from: ctx.accounts.depositor_collateral.to_account_info(),
        to: ctx.accounts.collateral_vault.to_account_info(),
        authority: ctx.accounts.depositor.to_account_info(),
    };
    let cpi_ctx = CpiContext::new(ctx.accounts.token_program.to_account_info(), cpi_accounts);
    token::transfer(cpi_ctx, amount)?;

    // Update position
    let position = &mut ctx.accounts.position;
    position.collateral_amount = position
        .collateral_amount
        .checked_add(amount)
        .ok_or(KusdError::MathOverflow)?;

    // Update vault totals
    let vault = &mut ctx.accounts.vault;
    vault.total_collateral = vault
        .total_collateral
        .checked_add(amount)
        .ok_or(KusdError::MathOverflow)?;

    Ok(())
}
