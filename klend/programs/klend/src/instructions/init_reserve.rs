use anchor_lang::prelude::*;
use anchor_spl::associated_token::AssociatedToken;
use anchor_spl::token::{Mint, Token, TokenAccount};

use crate::constants::*;
use crate::errors::KlendError;
use crate::state::{LendingMarket, MockOracle, Reserve, ReserveConfig};

#[derive(Accounts)]
pub struct InitReserve<'info> {
    #[account(mut)]
    pub admin: Signer<'info>,

    #[account(
        seeds = [LENDING_MARKET_SEED, admin.key().as_ref()],
        bump = lending_market.bump,
        has_one = admin,
    )]
    pub lending_market: Account<'info, LendingMarket>,

    pub token_mint: Account<'info, Mint>,

    #[account(
        seeds = [MOCK_ORACLE_SEED, token_mint.key().as_ref()],
        bump = oracle.bump,
    )]
    pub oracle: Account<'info, MockOracle>,

    #[account(
        init,
        payer = admin,
        space = 8 + Reserve::SPACE,
        seeds = [RESERVE_SEED, lending_market.key().as_ref(), token_mint.key().as_ref()],
        bump,
    )]
    pub reserve: Account<'info, Reserve>,

    /// CHECK: PDA used as signing authority for vault
    #[account(
        seeds = [RESERVE_AUTHORITY_SEED, reserve.key().as_ref()],
        bump,
    )]
    pub reserve_authority: UncheckedAccount<'info>,

    #[account(
        init,
        payer = admin,
        associated_token::mint = token_mint,
        associated_token::authority = reserve_authority,
    )]
    pub token_vault: Account<'info, TokenAccount>,

    pub system_program: Program<'info, System>,
    pub token_program: Program<'info, Token>,
    pub associated_token_program: Program<'info, AssociatedToken>,
}

impl Reserve {
    pub const SPACE: usize = 32  // lending_market
        + 32  // token_mint
        + 32  // token_vault
        + 32  // reserve_authority
        + 32  // oracle
        + 8   // deposited_liquidity
        + 8   // borrowed_liquidity
        + 8   // accumulated_protocol_fees
        + 8   // total_shares
        + 16  // cumulative_borrow_index
        + 8   // last_update_timestamp
        // ReserveConfig
        + 2   // ltv
        + 2   // liquidation_threshold
        + 2   // liquidation_bonus
        + 2   // reserve_factor
        + 8   // r_base
        + 8   // r_slope1
        + 8   // r_slope2
        + 8   // u_optimal
        + 8   // supply_cap
        + 8   // borrow_cap
        + 8   // oracle_max_staleness
        + 1   // bump
        + 1;  // authority_bump
}

pub fn handle_init_reserve(ctx: Context<InitReserve>, config: ReserveConfig) -> Result<()> {
    // Validate config
    require!(
        config.ltv < config.liquidation_threshold,
        KlendError::InvalidConfigLtv
    );
    require!(
        config.liquidation_threshold <= BPS_SCALE as u16,
        KlendError::InvalidConfigLiqThreshold
    );
    require!(
        config.reserve_factor <= BPS_SCALE as u16,
        KlendError::InvalidConfigReserveFactor
    );
    require!(
        config.u_optimal <= SCALE as u64,
        KlendError::InvalidConfigUtilization
    );

    let reserve = &mut ctx.accounts.reserve;
    reserve.lending_market = ctx.accounts.lending_market.key();
    reserve.token_mint = ctx.accounts.token_mint.key();
    reserve.token_vault = ctx.accounts.token_vault.key();
    reserve.reserve_authority = ctx.accounts.reserve_authority.key();
    reserve.oracle = ctx.accounts.oracle.key();

    reserve.deposited_liquidity = 0;
    reserve.borrowed_liquidity = 0;
    reserve.accumulated_protocol_fees = 0;
    reserve.total_shares = 0;

    reserve.cumulative_borrow_index = SCALE;
    reserve.last_update_timestamp = Clock::get()?.unix_timestamp;

    reserve.config = config;

    reserve.bump = ctx.bumps.reserve;
    reserve.authority_bump = ctx.bumps.reserve_authority;

    Ok(())
}
