use anchor_lang::prelude::*;
use anchor_spl::token::{self, Mint, Token, TokenAccount, Transfer};

use crate::constants::*;
use crate::errors::CpammError;
use crate::state::Pool;

#[derive(Accounts)]
pub struct Swap<'info> {
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
        address = pool.vault_a,
    )]
    pub vault_a: Account<'info, TokenAccount>,

    #[account(
        mut,
        address = pool.vault_b,
    )]
    pub vault_b: Account<'info, TokenAccount>,

    /// The user's token account for the input mint.
    #[account(mut)]
    pub user_token_in: Account<'info, TokenAccount>,

    /// The user's token account for the output mint.
    #[account(mut)]
    pub user_token_out: Account<'info, TokenAccount>,

    /// The mint of the input token (used to determine swap direction).
    pub input_mint: Account<'info, Mint>,

    pub token_program: Program<'info, Token>,
}

pub fn handle_swap(ctx: Context<Swap>, amount_in: u64, minimum_amount_out: u64) -> Result<()> {
    require!(amount_in > 0, CpammError::ZeroSwapAmount);

    let pool = &ctx.accounts.pool;
    require!(
        pool.reserve_a > 0 && pool.reserve_b > 0,
        CpammError::EmptyPool
    );

    let input_mint = ctx.accounts.input_mint.key();

    // Determine swap direction
    let (reserve_in, reserve_out, is_a_to_b) = if input_mint == pool.mint_a {
        (pool.reserve_a, pool.reserve_b, true)
    } else if input_mint == pool.mint_b {
        (pool.reserve_b, pool.reserve_a, false)
    } else {
        return Err(CpammError::InvalidInputMint.into());
    };

    // Validate token accounts match the swap direction
    if is_a_to_b {
        require!(
            ctx.accounts.user_token_in.mint == pool.mint_a
                && ctx.accounts.user_token_out.mint == pool.mint_b,
            CpammError::InvalidInputMint
        );
    } else {
        require!(
            ctx.accounts.user_token_in.mint == pool.mint_b
                && ctx.accounts.user_token_out.mint == pool.mint_a,
            CpammError::InvalidInputMint
        );
    }

    // Calculate output with fee:
    // amount_out = (reserve_out * amount_in_with_fee) / (reserve_in * 10000 + amount_in_with_fee)
    // where amount_in_with_fee = amount_in * (10000 - 30)
    let amount_in_with_fee = (amount_in as u128)
        .checked_mul((FEE_DENOMINATOR - FEE_NUMERATOR) as u128)
        .ok_or(CpammError::MathOverflow)?;

    let numerator = (reserve_out as u128)
        .checked_mul(amount_in_with_fee)
        .ok_or(CpammError::MathOverflow)?;

    let denominator = (reserve_in as u128)
        .checked_mul(FEE_DENOMINATOR as u128)
        .ok_or(CpammError::MathOverflow)?
        .checked_add(amount_in_with_fee)
        .ok_or(CpammError::MathOverflow)?;

    let amount_out = numerator
        .checked_div(denominator)
        .ok_or(CpammError::MathOverflow)? as u64;

    require!(amount_out > 0, CpammError::ZeroOutputAmount);
    require!(
        amount_out >= minimum_amount_out,
        CpammError::SlippageExceededOutput
    );

    // Transfer input from user to vault
    let (vault_in, vault_out) = if is_a_to_b {
        (
            ctx.accounts.vault_a.to_account_info(),
            ctx.accounts.vault_b.to_account_info(),
        )
    } else {
        (
            ctx.accounts.vault_b.to_account_info(),
            ctx.accounts.vault_a.to_account_info(),
        )
    };

    token::transfer(
        CpiContext::new(
            ctx.accounts.token_program.to_account_info(),
            Transfer {
                from: ctx.accounts.user_token_in.to_account_info(),
                to: vault_in,
                authority: ctx.accounts.user.to_account_info(),
            },
        ),
        amount_in,
    )?;

    // Transfer output from vault to user (signed by pool_authority)
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
                from: vault_out,
                to: ctx.accounts.user_token_out.to_account_info(),
                authority: ctx.accounts.pool_authority.to_account_info(),
            },
            &[authority_seeds],
        ),
        amount_out,
    )?;

    // Update reserves
    let pool = &mut ctx.accounts.pool;
    if is_a_to_b {
        pool.reserve_a = pool
            .reserve_a
            .checked_add(amount_in)
            .ok_or(CpammError::MathOverflow)?;
        pool.reserve_b = pool
            .reserve_b
            .checked_sub(amount_out)
            .ok_or(CpammError::MathOverflow)?;
    } else {
        pool.reserve_b = pool
            .reserve_b
            .checked_add(amount_in)
            .ok_or(CpammError::MathOverflow)?;
        pool.reserve_a = pool
            .reserve_a
            .checked_sub(amount_out)
            .ok_or(CpammError::MathOverflow)?;
    }

    Ok(())
}
