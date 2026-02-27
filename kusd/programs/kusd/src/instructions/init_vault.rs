use anchor_lang::prelude::*;
use anchor_spl::associated_token::AssociatedToken;
use anchor_spl::token::{Mint, Token, TokenAccount};

use crate::constants::*;
use crate::errors::KusdError;
use crate::state::{CdpVault, MockOracle};

#[derive(Accounts)]
pub struct InitVault<'info> {
    #[account(mut)]
    pub admin: Signer<'info>,

    pub collateral_mint: Account<'info, Mint>,

    #[account(
        constraint = oracle.token_mint == collateral_mint.key() @ KusdError::InvalidOracle,
    )]
    pub oracle: Account<'info, MockOracle>,

    #[account(
        init,
        payer = admin,
        space = 8 + CdpVault::SPACE,
        seeds = [CDP_VAULT_SEED, collateral_mint.key().as_ref()],
        bump,
    )]
    pub vault: Account<'info, CdpVault>,

    /// CHECK: PDA used as signing authority for transfers and mint/burn
    #[account(
        mut,
        seeds = [CDP_VAULT_AUTHORITY_SEED, vault.key().as_ref()],
        bump,
    )]
    pub vault_authority: UncheckedAccount<'info>,

    #[account(
        init,
        payer = admin,
        mint::decimals = KUSD_DECIMALS,
        mint::authority = vault_authority,
        seeds = [KUSD_MINT_SEED, vault.key().as_ref()],
        bump,
    )]
    pub kusd_mint: Account<'info, Mint>,

    #[account(
        init,
        payer = admin,
        associated_token::mint = collateral_mint,
        associated_token::authority = vault_authority,
    )]
    pub collateral_token_account: Account<'info, TokenAccount>,

    pub token_program: Program<'info, Token>,
    pub associated_token_program: Program<'info, AssociatedToken>,
    pub system_program: Program<'info, System>,
}

pub fn handle_init_vault(
    ctx: Context<InitVault>,
    max_ltv_bps: u16,
    liquidation_threshold_bps: u16,
    liquidation_bonus_bps: u16,
    stability_fee_bps: u16,
    debt_ceiling: u64,
    oracle_max_staleness: u64,
) -> Result<()> {
    // Validate config
    require!(
        max_ltv_bps < liquidation_threshold_bps,
        KusdError::InvalidConfigLtv
    );
    require!(
        liquidation_threshold_bps <= BPS_SCALE as u16,
        KusdError::InvalidConfigLiqThreshold
    );
    require!(
        liquidation_bonus_bps < BPS_SCALE as u16,
        KusdError::InvalidConfigBonus
    );
    require!(
        stability_fee_bps <= BPS_SCALE as u16,
        KusdError::InvalidConfigFee
    );

    // Fund vault authority with SOL for rent
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
    vault.collateral_mint = ctx.accounts.collateral_mint.key();
    vault.kusd_mint = ctx.accounts.kusd_mint.key();
    vault.collateral_token_account = ctx.accounts.collateral_token_account.key();
    vault.vault_authority = ctx.accounts.vault_authority.key();
    vault.oracle = ctx.accounts.oracle.key();

    vault.max_ltv_bps = max_ltv_bps;
    vault.liquidation_threshold_bps = liquidation_threshold_bps;
    vault.liquidation_bonus_bps = liquidation_bonus_bps;
    vault.stability_fee_bps = stability_fee_bps;
    vault.oracle_max_staleness = oracle_max_staleness;
    vault.debt_ceiling = debt_ceiling;

    vault.total_collateral = 0;
    vault.total_debt_shares = 0;
    vault.cumulative_fee_index = SCALE;
    vault.last_update_timestamp = Clock::get()?.unix_timestamp;

    vault.collateral_decimals = ctx.accounts.collateral_mint.decimals;
    vault.halted = false;

    vault.vault_bump = ctx.bumps.vault;
    vault.authority_bump = ctx.bumps.vault_authority;
    vault.kusd_mint_bump = ctx.bumps.kusd_mint;

    Ok(())
}
