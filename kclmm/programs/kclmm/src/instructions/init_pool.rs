use anchor_lang::prelude::*;
use anchor_spl::associated_token::AssociatedToken;
use anchor_spl::token::{Mint, Token, TokenAccount};

use crate::constants::*;
use crate::errors::KclmmError;
use crate::math;
use crate::state::Pool;

#[derive(Accounts)]
#[instruction(fee_rate: u32, initial_sqrt_price: u128)]
pub struct InitPool<'info> {
    #[account(mut)]
    pub payer: Signer<'info>,

    pub mint_a: Account<'info, Mint>,
    pub mint_b: Account<'info, Mint>,

    #[account(
        init,
        payer = payer,
        space = Pool::SPACE,
        seeds = [POOL_SEED, mint_a.key().as_ref(), mint_b.key().as_ref(), &fee_rate.to_le_bytes()],
        bump,
    )]
    pub pool: Account<'info, Pool>,

    /// CHECK: PDA used as token authority, no data
    #[account(
        seeds = [POOL_AUTHORITY_SEED, pool.key().as_ref()],
        bump,
    )]
    pub pool_authority: UncheckedAccount<'info>,

    #[account(
        init,
        payer = payer,
        associated_token::mint = mint_a,
        associated_token::authority = pool_authority,
    )]
    pub vault_a: Account<'info, TokenAccount>,

    #[account(
        init,
        payer = payer,
        associated_token::mint = mint_b,
        associated_token::authority = pool_authority,
    )]
    pub vault_b: Account<'info, TokenAccount>,

    pub system_program: Program<'info, System>,
    pub token_program: Program<'info, Token>,
    pub associated_token_program: Program<'info, AssociatedToken>,
}

pub fn handle_init_pool(
    ctx: Context<InitPool>,
    fee_rate: u32,
    initial_sqrt_price: u128,
) -> Result<()> {
    // Validate token order
    require!(
        ctx.accounts.mint_a.key() < ctx.accounts.mint_b.key(),
        KclmmError::InvalidTokenOrder
    );

    // Validate fee tier
    let tick_spacing = fee_rate_to_tick_spacing(fee_rate)
        .ok_or(error!(KclmmError::InvalidFeeTier))?;

    // Validate sqrt price
    require!(
        initial_sqrt_price >= MIN_SQRT_PRICE && initial_sqrt_price <= MAX_SQRT_PRICE,
        KclmmError::InvalidSqrtPrice
    );

    // Derive current tick from sqrt price
    let tick_current = math::sqrt_price_to_tick(initial_sqrt_price)?;

    let pool = &mut ctx.accounts.pool;
    pool.mint_a = ctx.accounts.mint_a.key();
    pool.mint_b = ctx.accounts.mint_b.key();
    pool.vault_a = ctx.accounts.vault_a.key();
    pool.vault_b = ctx.accounts.vault_b.key();
    pool.pool_authority = ctx.accounts.pool_authority.key();
    pool.fee_rate = fee_rate;
    pool.tick_spacing = tick_spacing;
    pool.protocol_fee_rate = DEFAULT_PROTOCOL_FEE_RATE;
    pool.sqrt_price = initial_sqrt_price;
    pool.tick_current = tick_current;
    pool.liquidity = 0;
    pool.fee_growth_global_a = 0;
    pool.fee_growth_global_b = 0;
    pool.protocol_fees_a = 0;
    pool.protocol_fees_b = 0;
    pool.pool_bump = ctx.bumps.pool;
    pool.authority_bump = ctx.bumps.pool_authority;
    pool._padding = [0u8; 6];

    Ok(())
}
