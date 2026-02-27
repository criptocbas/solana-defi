use anchor_lang::prelude::*;
use anchor_spl::token::{self, Token, TokenAccount, Transfer};

use crate::constants::*;
use crate::errors::KusdError;
use crate::instructions::common::accrue_vault_fees;
use crate::math;
use crate::state::{CdpPosition, CdpVault, MockOracle};

#[derive(Accounts)]
pub struct WithdrawCollateral<'info> {
    #[account(mut)]
    pub owner: Signer<'info>,

    #[account(
        mut,
        seeds = [CDP_VAULT_SEED, vault.collateral_mint.as_ref()],
        bump = vault.vault_bump,
    )]
    pub vault: Account<'info, CdpVault>,

    /// CHECK: PDA signing authority
    #[account(
        seeds = [CDP_VAULT_AUTHORITY_SEED, vault.key().as_ref()],
        bump = vault.authority_bump,
    )]
    pub vault_authority: UncheckedAccount<'info>,

    #[account(
        mut,
        seeds = [CDP_POSITION_SEED, vault.key().as_ref(), owner.key().as_ref()],
        bump = position.bump,
        has_one = vault,
        has_one = owner,
    )]
    pub position: Account<'info, CdpPosition>,

    /// Owner's collateral token account (receives withdrawn collateral)
    #[account(mut)]
    pub owner_collateral: Account<'info, TokenAccount>,

    /// Vault's collateral token account
    #[account(
        mut,
        constraint = collateral_vault.key() == vault.collateral_token_account,
    )]
    pub collateral_vault: Account<'info, TokenAccount>,

    #[account(
        constraint = oracle.key() == vault.oracle @ KusdError::InvalidOracle,
    )]
    pub oracle: Account<'info, MockOracle>,

    pub token_program: Program<'info, Token>,
}

pub fn handle_withdraw_collateral(ctx: Context<WithdrawCollateral>, amount: u64) -> Result<()> {
    require!(amount > 0, KusdError::ZeroWithdraw);

    let vault = &mut ctx.accounts.vault;

    // Accrue fees
    let clock = Clock::get()?;
    accrue_vault_fees(vault, clock.unix_timestamp)?;

    // Check oracle staleness
    let oracle = &ctx.accounts.oracle;
    let staleness = clock.unix_timestamp.saturating_sub(oracle.timestamp) as u64;
    require!(staleness <= vault.oracle_max_staleness, KusdError::OracleStale);

    let position = &ctx.accounts.position;
    require!(
        amount <= position.collateral_amount,
        KusdError::InsufficientCollateral
    );

    // If position has debt, check LTV after withdrawal
    if position.debt_shares > 0 {
        let current_debt = math::shares_to_debt(position.debt_shares, vault.cumulative_fee_index)?;
        let new_collateral = position.collateral_amount - amount;
        let coll_usd = math::collateral_value_usd(new_collateral, oracle.price, oracle.decimals)?;

        let max_debt = coll_usd
            .checked_mul(vault.max_ltv_bps as u128)
            .ok_or(KusdError::MathOverflow)?
            / (BPS_SCALE as u128);
        require!(current_debt as u128 <= max_debt, KusdError::ExceedsMaxLtv);
    }

    // Transfer collateral back to owner via PDA signer
    let vault_key = vault.key();
    let authority_seeds: &[&[u8]] = &[
        CDP_VAULT_AUTHORITY_SEED,
        vault_key.as_ref(),
        &[vault.authority_bump],
    ];
    let signer_seeds = [authority_seeds];

    let cpi_accounts = Transfer {
        from: ctx.accounts.collateral_vault.to_account_info(),
        to: ctx.accounts.owner_collateral.to_account_info(),
        authority: ctx.accounts.vault_authority.to_account_info(),
    };
    let cpi_ctx = CpiContext::new_with_signer(
        ctx.accounts.token_program.to_account_info(),
        cpi_accounts,
        &signer_seeds,
    );
    token::transfer(cpi_ctx, amount)?;

    // Update position
    let position = &mut ctx.accounts.position;
    position.collateral_amount = position.collateral_amount.saturating_sub(amount);

    // Update vault totals
    let vault = &mut ctx.accounts.vault;
    vault.total_collateral = vault.total_collateral.saturating_sub(amount);

    Ok(())
}
