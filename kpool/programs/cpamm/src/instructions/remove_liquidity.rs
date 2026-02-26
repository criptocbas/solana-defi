use anchor_lang::prelude::*;
use anchor_spl::token::{self, Burn, Mint, Token, TokenAccount, Transfer};

use crate::constants::*;
use crate::errors::CpammError;
use crate::state::Pool;

#[derive(Accounts)]
pub struct RemoveLiquidity<'info> {
    #[account(mut)]
    pub user: Signer<'info>,

    #[account(
        mut,
        seeds = [POOL_SEED, pool.mint_a.as_ref(), pool.mint_b.as_ref()],
        bump = pool.pool_bump,
    )]
    pub pool: Account<'info, Pool>,

    /// CHECK: Pool authority PDA.
    #[account(
        seeds = [POOL_AUTHORITY_SEED, pool.key().as_ref()],
        bump = pool.authority_bump,
    )]
    pub pool_authority: UncheckedAccount<'info>,

    #[account(
        mut,
        address = pool.lp_mint,
    )]
    pub lp_mint: Account<'info, Mint>,

    #[account(
        mut,
        address = pool.vault_a,
    )]
    pub vault_a: Account<'info, TokenAccount>,

    #[account(
        mut,
        address = pool.vault_b,
    )]
    pub vault_b: Account<'info, TokenAccount>,

    #[account(
        mut,
        token::mint = pool.lp_mint,
        token::authority = user,
    )]
    pub user_lp_token: Account<'info, TokenAccount>,

    #[account(
        mut,
        token::mint = pool.mint_a,
        token::authority = user,
    )]
    pub user_token_a: Account<'info, TokenAccount>,

    #[account(
        mut,
        token::mint = pool.mint_b,
        token::authority = user,
    )]
    pub user_token_b: Account<'info, TokenAccount>,

    pub token_program: Program<'info, Token>,
}

pub fn handle_remove_liquidity(
    ctx: Context<RemoveLiquidity>,
    lp_burn: u64,
    min_amount_a: u64,
    min_amount_b: u64,
) -> Result<()> {
    require!(lp_burn > 0, CpammError::ZeroBurnAmount);

    let pool = &ctx.accounts.pool;
    let total_supply = ctx.accounts.lp_mint.supply;

    // Calculate proportional withdrawal amounts
    let amount_a = (lp_burn as u128)
        .checked_mul(pool.reserve_a as u128)
        .ok_or(CpammError::MathOverflow)?
        .checked_div(total_supply as u128)
        .ok_or(CpammError::MathOverflow)? as u64;

    let amount_b = (lp_burn as u128)
        .checked_mul(pool.reserve_b as u128)
        .ok_or(CpammError::MathOverflow)?
        .checked_div(total_supply as u128)
        .ok_or(CpammError::MathOverflow)? as u64;

    require!(amount_a >= min_amount_a, CpammError::SlippageExceededAmountA);
    require!(amount_b >= min_amount_b, CpammError::SlippageExceededAmountB);

    // Burn user's LP tokens
    token::burn(
        CpiContext::new(
            ctx.accounts.token_program.to_account_info(),
            Burn {
                mint: ctx.accounts.lp_mint.to_account_info(),
                from: ctx.accounts.user_lp_token.to_account_info(),
                authority: ctx.accounts.user.to_account_info(),
            },
        ),
        lp_burn,
    )?;

    // Transfer tokens from vaults to user (signed by pool_authority)
    let pool_key = ctx.accounts.pool.key();
    let authority_seeds: &[&[u8]] = &[
        POOL_AUTHORITY_SEED,
        pool_key.as_ref(),
        &[ctx.accounts.pool.authority_bump],
    ];

    token::transfer(
        CpiContext::new_with_signer(
            ctx.accounts.token_program.to_account_info(),
            Transfer {
                from: ctx.accounts.vault_a.to_account_info(),
                to: ctx.accounts.user_token_a.to_account_info(),
                authority: ctx.accounts.pool_authority.to_account_info(),
            },
            &[authority_seeds],
        ),
        amount_a,
    )?;

    token::transfer(
        CpiContext::new_with_signer(
            ctx.accounts.token_program.to_account_info(),
            Transfer {
                from: ctx.accounts.vault_b.to_account_info(),
                to: ctx.accounts.user_token_b.to_account_info(),
                authority: ctx.accounts.pool_authority.to_account_info(),
            },
            &[authority_seeds],
        ),
        amount_b,
    )?;

    // Update reserves
    let pool = &mut ctx.accounts.pool;
    pool.reserve_a = pool
        .reserve_a
        .checked_sub(amount_a)
        .ok_or(CpammError::MathOverflow)?;
    pool.reserve_b = pool
        .reserve_b
        .checked_sub(amount_b)
        .ok_or(CpammError::MathOverflow)?;

    Ok(())
}
