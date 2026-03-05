use anchor_lang::prelude::*;
use anchor_spl::{
    associated_token::AssociatedToken,
    token::{self, Mint, MintTo, Token, TokenAccount, Transfer},
};

use crate::constants::*;
use crate::errors::CpammError;
use crate::state::Pool;

#[derive(Accounts)]
pub struct AddLiquidity<'info> {
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

    #[account(
        init_if_needed,
        payer = user,
        associated_token::mint = lp_mint,
        associated_token::authority = user,
    )]
    pub user_lp_token: Account<'info, TokenAccount>,

    /// The vault that holds the permanently locked MINIMUM_LIQUIDITY LP tokens.
    #[account(
        mut,
        associated_token::mint = lp_mint,
        associated_token::authority = pool_authority,
    )]
    pub locked_lp_vault: Account<'info, TokenAccount>,

    pub system_program: Program<'info, System>,
    pub token_program: Program<'info, Token>,
    pub associated_token_program: Program<'info, AssociatedToken>,
}

pub fn handle_add_liquidity(
    ctx: Context<AddLiquidity>,
    amount_a_desired: u64,
    amount_b_desired: u64,
    minimum_lp_tokens: u64,
) -> Result<()> {
    require!(
        amount_a_desired > 0 && amount_b_desired > 0,
        CpammError::ZeroDepositAmount
    );

    let pool = &ctx.accounts.pool;
    let total_supply = ctx.accounts.lp_mint.supply;

    let (amount_a, amount_b, lp_tokens_to_user) = if total_supply == 0 {
        // First deposit: LP = sqrt(a * b) - MINIMUM_LIQUIDITY
        let product = (amount_a_desired as u128)
            .checked_mul(amount_b_desired as u128)
            .ok_or(CpammError::MathOverflow)?;
        let sqrt = isqrt(product);
        require!(
            sqrt > MINIMUM_LIQUIDITY as u128,
            CpammError::InsufficientInitialLiquidity
        );
        let lp_total = sqrt as u64;
        let lp_to_user = lp_total - MINIMUM_LIQUIDITY;
        (amount_a_desired, amount_b_desired, lp_to_user)
    } else {
        // Subsequent deposit: proportional
        let reserve_a = pool.reserve_a;
        let reserve_b = pool.reserve_b;

        let lp_from_a = (amount_a_desired as u128)
            .checked_mul(total_supply as u128)
            .ok_or(CpammError::MathOverflow)?
            .checked_div(reserve_a as u128)
            .ok_or(CpammError::MathOverflow)?;

        let lp_from_b = (amount_b_desired as u128)
            .checked_mul(total_supply as u128)
            .ok_or(CpammError::MathOverflow)?
            .checked_div(reserve_b as u128)
            .ok_or(CpammError::MathOverflow)?;

        let lp_tokens = lp_from_a.min(lp_from_b) as u64;

        // Calculate actual amounts based on limiting side
        let (actual_a, actual_b) = if lp_from_a <= lp_from_b {
            // A is the limiting factor
            let actual_b = (lp_tokens as u128)
                .checked_mul(reserve_b as u128)
                .ok_or(CpammError::MathOverflow)?
                .checked_div(total_supply as u128)
                .ok_or(CpammError::MathOverflow)? as u64;
            (amount_a_desired, actual_b)
        } else {
            // B is the limiting factor
            let actual_a = (lp_tokens as u128)
                .checked_mul(reserve_a as u128)
                .ok_or(CpammError::MathOverflow)?
                .checked_div(total_supply as u128)
                .ok_or(CpammError::MathOverflow)? as u64;
            (actual_a, amount_b_desired)
        };

        (actual_a, actual_b, lp_tokens)
    };

    require!(
        lp_tokens_to_user >= minimum_lp_tokens,
        CpammError::SlippageExceededMint
    );

    // CEI: Update reserves before transfers
    let pool_key = ctx.accounts.pool.key();
    let authority_bump = ctx.accounts.pool.authority_bump;

    let pool = &mut ctx.accounts.pool;
    pool.reserve_a = pool
        .reserve_a
        .checked_add(amount_a)
        .ok_or(CpammError::MathOverflow)?;
    pool.reserve_b = pool
        .reserve_b
        .checked_add(amount_b)
        .ok_or(CpammError::MathOverflow)?;

    // Transfer tokens from user to vaults
    token::transfer(
        CpiContext::new(
            ctx.accounts.token_program.to_account_info(),
            Transfer {
                from: ctx.accounts.user_token_a.to_account_info(),
                to: ctx.accounts.vault_a.to_account_info(),
                authority: ctx.accounts.user.to_account_info(),
            },
        ),
        amount_a,
    )?;

    token::transfer(
        CpiContext::new(
            ctx.accounts.token_program.to_account_info(),
            Transfer {
                from: ctx.accounts.user_token_b.to_account_info(),
                to: ctx.accounts.vault_b.to_account_info(),
                authority: ctx.accounts.user.to_account_info(),
            },
        ),
        amount_b,
    )?;

    // Mint LP tokens
    let authority_seeds: &[&[u8]] = &[
        POOL_AUTHORITY_SEED,
        pool_key.as_ref(),
        &[authority_bump],
    ];

    if total_supply == 0 {
        // First deposit: mint MINIMUM_LIQUIDITY to locked vault
        token::mint_to(
            CpiContext::new_with_signer(
                ctx.accounts.token_program.to_account_info(),
                MintTo {
                    mint: ctx.accounts.lp_mint.to_account_info(),
                    to: ctx.accounts.locked_lp_vault.to_account_info(),
                    authority: ctx.accounts.pool_authority.to_account_info(),
                },
                &[authority_seeds],
            ),
            MINIMUM_LIQUIDITY,
        )?;
    }

    // Mint LP tokens to user
    token::mint_to(
        CpiContext::new_with_signer(
            ctx.accounts.token_program.to_account_info(),
            MintTo {
                mint: ctx.accounts.lp_mint.to_account_info(),
                to: ctx.accounts.user_lp_token.to_account_info(),
                authority: ctx.accounts.pool_authority.to_account_info(),
            },
            &[authority_seeds],
        ),
        lp_tokens_to_user,
    )?;

    Ok(())
}

/// Integer square root via Newton's method.
fn isqrt(n: u128) -> u128 {
    if n == 0 {
        return 0;
    }
    let mut x = n;
    let mut y = (x + 1) / 2;
    while y < x {
        x = y;
        y = (x + n / x) / 2;
    }
    x
}
