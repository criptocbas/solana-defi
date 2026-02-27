use anchor_lang::prelude::*;
use anchor_spl::associated_token::AssociatedToken;
use anchor_spl::token::{Mint, Token, TokenAccount};

use crate::constants::*;
use crate::errors::KlevError;
use crate::state::LeveragedVault;

#[derive(Accounts)]
pub struct InitVault<'info> {
    #[account(mut)]
    pub admin: Signer<'info>,

    /// Collateral token mint (e.g. SOL)
    pub collateral_mint: Account<'info, Mint>,
    /// Debt token mint (e.g. USDC)
    pub debt_mint: Account<'info, Mint>,

    #[account(
        init,
        payer = admin,
        space = 8 + LeveragedVault::SPACE,
        seeds = [LEVERAGED_VAULT_SEED, collateral_mint.key().as_ref(), debt_mint.key().as_ref()],
        bump,
    )]
    pub vault: Account<'info, LeveragedVault>,

    /// CHECK: PDA used as signing authority for transfers and CPI
    #[account(
        mut,
        seeds = [LEV_VAULT_AUTHORITY_SEED, vault.key().as_ref()],
        bump,
    )]
    pub vault_authority: UncheckedAccount<'info>,

    #[account(
        init,
        payer = admin,
        mint::decimals = collateral_mint.decimals,
        mint::authority = vault_authority,
        seeds = [LEV_SHARE_MINT_SEED, vault.key().as_ref()],
        bump,
    )]
    pub share_mint: Account<'info, Mint>,

    #[account(
        init,
        payer = admin,
        associated_token::mint = collateral_mint,
        associated_token::authority = vault_authority,
    )]
    pub collateral_token_account: Account<'info, TokenAccount>,

    #[account(
        init,
        payer = admin,
        associated_token::mint = debt_mint,
        associated_token::authority = vault_authority,
    )]
    pub debt_token_account: Account<'info, TokenAccount>,

    // ── klend references ──
    /// CHECK: klend program, stored in vault state
    pub klend_program: UncheckedAccount<'info>,
    /// CHECK: klend lending market
    pub klend_lending_market: UncheckedAccount<'info>,
    /// CHECK: klend collateral reserve (SOL)
    pub klend_collateral_reserve: UncheckedAccount<'info>,
    /// CHECK: klend debt reserve (USDC)
    pub klend_debt_reserve: UncheckedAccount<'info>,

    // ── cpamm references ──
    /// CHECK: cpamm program, stored in vault state
    pub cpamm_program: UncheckedAccount<'info>,
    /// CHECK: cpamm pool for collateral/debt pair
    pub cpamm_pool: UncheckedAccount<'info>,

    pub token_program: Program<'info, Token>,
    pub associated_token_program: Program<'info, AssociatedToken>,
    pub system_program: Program<'info, System>,
}

pub fn handle_init_vault(
    ctx: Context<InitVault>,
    performance_fee_bps: u16,
    management_fee_bps: u16,
    deposit_cap: u64,
    max_leverage_bps: u16,
    min_health_factor_bps: u16,
) -> Result<()> {
    require!(
        performance_fee_bps <= BPS_SCALE as u16,
        KlevError::PerformanceFeeExceeded
    );
    require!(
        management_fee_bps <= BPS_SCALE as u16,
        KlevError::ManagementFeeExceeded
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
    vault.collateral_mint = ctx.accounts.collateral_mint.key();
    vault.debt_mint = ctx.accounts.debt_mint.key();
    vault.share_mint = ctx.accounts.share_mint.key();
    vault.vault_authority = ctx.accounts.vault_authority.key();
    vault.collateral_token_account = ctx.accounts.collateral_token_account.key();
    vault.debt_token_account = ctx.accounts.debt_token_account.key();
    vault.klend_program = ctx.accounts.klend_program.key();
    vault.klend_lending_market = ctx.accounts.klend_lending_market.key();
    vault.klend_collateral_reserve = ctx.accounts.klend_collateral_reserve.key();
    vault.klend_debt_reserve = ctx.accounts.klend_debt_reserve.key();
    vault.cpamm_program = ctx.accounts.cpamm_program.key();
    vault.cpamm_pool = ctx.accounts.cpamm_pool.key();
    vault.cached_collateral_value = 0;
    vault.cached_debt_value = 0;
    vault.cached_net_equity_collateral = 0;
    vault.last_harvest_timestamp = Clock::get()?.unix_timestamp;
    vault.performance_fee_bps = performance_fee_bps;
    vault.management_fee_bps = management_fee_bps;
    vault.max_leverage_bps = max_leverage_bps;
    vault.min_health_factor_bps = min_health_factor_bps;
    vault.deposit_cap = deposit_cap;
    vault.halted = false;
    vault.vault_bump = ctx.bumps.vault;
    vault.authority_bump = ctx.bumps.vault_authority;
    vault.share_mint_bump = ctx.bumps.share_mint;

    Ok(())
}
