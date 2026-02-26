use anchor_lang::prelude::*;
use anchor_spl::token::{Mint, Token, TokenAccount};

use crate::errors::KrouterError;

#[derive(Accounts)]
pub struct SwapKclmm<'info> {
    pub user: Signer<'info>,

    /// CHECK: kclmm pool account (validated by kclmm via CPI)
    #[account(mut)]
    pub pool: UncheckedAccount<'info>,

    /// CHECK: Pool authority PDA (validated by kclmm via CPI)
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

    pub kclmm_program: Program<'info, KclmmProgram>,
    pub token_program: Program<'info, Token>,

    // remaining_accounts: tick arrays (1-3, mut)
}

pub fn handle_swap_kclmm<'info>(
    ctx: Context<'_, '_, 'info, 'info, SwapKclmm<'info>>,
    amount_in: u64,
    sqrt_price_limit: u128,
    minimum_amount_out: u64,
) -> Result<()> {
    require!(amount_in > 0, KrouterError::ZeroSwapAmount);

    // Record output balance before CPI
    let balance_before = ctx.accounts.user_token_out.amount;

    // CPI into kclmm::swap with tick arrays forwarded
    let cpi_accounts = kclmm::cpi::accounts::Swap {
        user: ctx.accounts.user.to_account_info(),
        pool: ctx.accounts.pool.to_account_info(),
        vault_a: ctx.accounts.vault_a.to_account_info(),
        vault_b: ctx.accounts.vault_b.to_account_info(),
        pool_authority: ctx.accounts.pool_authority.to_account_info(),
        user_token_in: ctx.accounts.user_token_in.to_account_info(),
        user_token_out: ctx.accounts.user_token_out.to_account_info(),
        input_mint: ctx.accounts.input_mint.to_account_info(),
        token_program: ctx.accounts.token_program.to_account_info(),
    };

    let tick_arrays: Vec<AccountInfo> = ctx.remaining_accounts.to_vec();

    let cpi_ctx = CpiContext::new(
        ctx.accounts.kclmm_program.to_account_info(),
        cpi_accounts,
    ).with_remaining_accounts(tick_arrays);

    kclmm::cpi::swap(cpi_ctx, amount_in, sqrt_price_limit, 0)?;

    // Reload and measure actual output
    ctx.accounts.user_token_out.reload()?;
    let actual_out = ctx.accounts.user_token_out.amount
        .checked_sub(balance_before)
        .ok_or(KrouterError::MathOverflow)?;

    require!(actual_out >= minimum_amount_out, KrouterError::SlippageExceeded);

    Ok(())
}

// Anchor Program type wrapper for kclmm
#[derive(Clone)]
pub struct KclmmProgram;

impl anchor_lang::Id for KclmmProgram {
    fn id() -> Pubkey {
        kclmm::ID
    }
}
