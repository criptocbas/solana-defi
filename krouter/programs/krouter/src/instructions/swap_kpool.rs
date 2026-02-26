use anchor_lang::prelude::*;
use anchor_spl::token::{Mint, Token, TokenAccount};

use crate::errors::KrouterError;

#[derive(Accounts)]
pub struct SwapKpool<'info> {
    pub user: Signer<'info>,

    /// CHECK: kpool pool account (validated by cpamm via CPI)
    #[account(mut)]
    pub pool: UncheckedAccount<'info>,

    /// CHECK: Pool authority PDA (validated by cpamm via CPI)
    pub pool_authority: UncheckedAccount<'info>,

    #[account(mut)]
    pub vault_a: Account<'info, TokenAccount>,

    #[account(mut)]
    pub vault_b: Account<'info, TokenAccount>,

    #[account(mut)]
    pub user_token_in: Account<'info, TokenAccount>,

    #[account(mut)]
    pub user_token_out: Account<'info, TokenAccount>,

    pub input_mint: Account<'info, Mint>,

    pub cpamm_program: Program<'info, CpammProgram>,
    pub token_program: Program<'info, Token>,
}

pub fn handle_swap_kpool(
    ctx: Context<SwapKpool>,
    amount_in: u64,
    minimum_amount_out: u64,
) -> Result<()> {
    require!(amount_in > 0, KrouterError::ZeroSwapAmount);

    // Record output balance before CPI
    let balance_before = ctx.accounts.user_token_out.amount;

    // CPI into cpamm::swap with 0 min_out (we enforce slippage here)
    let cpi_accounts = cpamm::cpi::accounts::Swap {
        user: ctx.accounts.user.to_account_info(),
        pool: ctx.accounts.pool.to_account_info(),
        pool_authority: ctx.accounts.pool_authority.to_account_info(),
        vault_a: ctx.accounts.vault_a.to_account_info(),
        vault_b: ctx.accounts.vault_b.to_account_info(),
        user_token_in: ctx.accounts.user_token_in.to_account_info(),
        user_token_out: ctx.accounts.user_token_out.to_account_info(),
        input_mint: ctx.accounts.input_mint.to_account_info(),
        token_program: ctx.accounts.token_program.to_account_info(),
    };

    let cpi_ctx = CpiContext::new(
        ctx.accounts.cpamm_program.to_account_info(),
        cpi_accounts,
    );
    cpamm::cpi::swap(cpi_ctx, amount_in, 0)?;

    // Reload and measure actual output
    ctx.accounts.user_token_out.reload()?;
    let actual_out = ctx.accounts.user_token_out.amount
        .checked_sub(balance_before)
        .ok_or(KrouterError::MathOverflow)?;

    require!(actual_out >= minimum_amount_out, KrouterError::SlippageExceeded);

    Ok(())
}

// Anchor Program type wrapper for cpamm
#[derive(Clone)]
pub struct CpammProgram;

impl anchor_lang::Id for CpammProgram {
    fn id() -> Pubkey {
        cpamm::ID
    }
}
