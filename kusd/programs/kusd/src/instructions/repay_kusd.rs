use anchor_lang::prelude::*;
use anchor_spl::token::{self, Burn, Mint, Token, TokenAccount};

use crate::constants::*;
use crate::errors::KusdError;
use crate::instructions::common::accrue_vault_fees;
use crate::math;
use crate::state::{CdpPosition, CdpVault};

#[derive(Accounts)]
pub struct RepayKusd<'info> {
    #[account(mut)]
    pub payer: Signer<'info>,

    #[account(
        mut,
        seeds = [CDP_VAULT_SEED, vault.collateral_mint.as_ref()],
        bump = vault.vault_bump,
    )]
    pub vault: Account<'info, CdpVault>,

    #[account(
        mut,
        seeds = [CDP_POSITION_SEED, vault.key().as_ref(), position_owner.key().as_ref()],
        bump = position.bump,
        has_one = vault,
    )]
    pub position: Account<'info, CdpPosition>,

    /// CHECK: owner of the position being repaid (anyone can repay on behalf)
    pub position_owner: AccountInfo<'info>,

    #[account(
        mut,
        seeds = [KUSD_MINT_SEED, vault.key().as_ref()],
        bump = vault.kusd_mint_bump,
    )]
    pub kusd_mint: Account<'info, Mint>,

    /// Payer's kUSD token account (kUSD burned from here)
    #[account(mut)]
    pub payer_kusd: Account<'info, TokenAccount>,

    pub token_program: Program<'info, Token>,
}

pub fn handle_repay_kusd(ctx: Context<RepayKusd>, amount: u64) -> Result<()> {
    require!(amount > 0, KusdError::ZeroRepay);

    let vault = &mut ctx.accounts.vault;

    // Accrue fees
    let clock = Clock::get()?;
    accrue_vault_fees(vault, clock.unix_timestamp)?;

    // Compute current debt
    let position = &ctx.accounts.position;
    let current_debt = math::shares_to_debt(position.debt_shares, vault.cumulative_fee_index)?;

    // Cap repay at current debt
    let repay_amount = amount.min(current_debt);
    if repay_amount == 0 {
        return Ok(());
    }

    // Burn kUSD from payer
    let cpi_accounts = Burn {
        mint: ctx.accounts.kusd_mint.to_account_info(),
        from: ctx.accounts.payer_kusd.to_account_info(),
        authority: ctx.accounts.payer.to_account_info(),
    };
    let cpi_ctx = CpiContext::new(ctx.accounts.token_program.to_account_info(), cpi_accounts);
    token::burn(cpi_ctx, repay_amount)?;

    // Update debt shares
    let position = &mut ctx.accounts.position;
    if repay_amount >= current_debt {
        // Full repay — zero out to avoid rounding dust
        let old_shares = position.debt_shares;
        position.debt_shares = 0;
        vault.total_debt_shares = vault.total_debt_shares.saturating_sub(old_shares);
    } else {
        // Partial repay
        let shares_to_remove = math::amount_to_shares(repay_amount, vault.cumulative_fee_index)?;
        position.debt_shares = position.debt_shares.saturating_sub(shares_to_remove);
        vault.total_debt_shares = vault.total_debt_shares.saturating_sub(shares_to_remove);
    }

    Ok(())
}
