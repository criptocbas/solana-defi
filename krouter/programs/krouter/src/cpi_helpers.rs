use anchor_lang::prelude::*;

use crate::errors::KrouterError;
use crate::types::PoolType;

/// Execute a swap leg via CPI given a slice of accounts from remaining_accounts.
///
/// Account layout per leg:
///   [0] DEX program (cpamm or kclmm)
///   [1] pool (mut)
///   [2] pool_authority
///   [3] vault_a (mut)
///   [4] vault_b (mut)
///   [5] input_mint
///   [6..] tick_arrays (kclmm only, mut)
///
/// The user, user_token_in, user_token_out, and token_program are passed separately.
pub fn execute_leg<'info>(
    pool_type: PoolType,
    leg_accounts: &[AccountInfo<'info>],
    user: &AccountInfo<'info>,
    user_token_in: &AccountInfo<'info>,
    user_token_out: &AccountInfo<'info>,
    token_program: &AccountInfo<'info>,
    amount_in: u64,
    sqrt_price_limit: u128,
) -> Result<()> {
    match pool_type {
        PoolType::Kpool => execute_kpool_leg(
            leg_accounts,
            user,
            user_token_in,
            user_token_out,
            token_program,
            amount_in,
        ),
        PoolType::Kclmm => execute_kclmm_leg(
            leg_accounts,
            user,
            user_token_in,
            user_token_out,
            token_program,
            amount_in,
            sqrt_price_limit,
        ),
    }
}

fn execute_kpool_leg<'info>(
    leg_accounts: &[AccountInfo<'info>],
    user: &AccountInfo<'info>,
    user_token_in: &AccountInfo<'info>,
    user_token_out: &AccountInfo<'info>,
    token_program: &AccountInfo<'info>,
    amount_in: u64,
) -> Result<()> {
    require!(leg_accounts.len() >= 6, KrouterError::InsufficientAccounts);

    let dex_program = &leg_accounts[0];
    let pool = &leg_accounts[1];
    let pool_authority = &leg_accounts[2];
    let vault_a = &leg_accounts[3];
    let vault_b = &leg_accounts[4];
    let input_mint = &leg_accounts[5];

    let cpi_accounts = cpamm::cpi::accounts::Swap {
        user: user.to_account_info(),
        pool: pool.to_account_info(),
        pool_authority: pool_authority.to_account_info(),
        vault_a: vault_a.to_account_info(),
        vault_b: vault_b.to_account_info(),
        user_token_in: user_token_in.to_account_info(),
        user_token_out: user_token_out.to_account_info(),
        input_mint: input_mint.to_account_info(),
        token_program: token_program.to_account_info(),
    };

    let cpi_ctx = CpiContext::new(dex_program.to_account_info(), cpi_accounts);
    // Pass 0 for min_out — krouter enforces slippage at the top level
    cpamm::cpi::swap(cpi_ctx, amount_in, 0)
}

fn execute_kclmm_leg<'info>(
    leg_accounts: &[AccountInfo<'info>],
    user: &AccountInfo<'info>,
    user_token_in: &AccountInfo<'info>,
    user_token_out: &AccountInfo<'info>,
    token_program: &AccountInfo<'info>,
    amount_in: u64,
    sqrt_price_limit: u128,
) -> Result<()> {
    require!(leg_accounts.len() >= 6, KrouterError::InsufficientAccounts);

    let dex_program = &leg_accounts[0];
    let pool = &leg_accounts[1];
    let pool_authority = &leg_accounts[2];
    let vault_a = &leg_accounts[3];
    let vault_b = &leg_accounts[4];
    let input_mint = &leg_accounts[5];

    let cpi_accounts = kclmm::cpi::accounts::Swap {
        user: user.to_account_info(),
        pool: pool.to_account_info(),
        vault_a: vault_a.to_account_info(),
        vault_b: vault_b.to_account_info(),
        pool_authority: pool_authority.to_account_info(),
        user_token_in: user_token_in.to_account_info(),
        user_token_out: user_token_out.to_account_info(),
        input_mint: input_mint.to_account_info(),
        token_program: token_program.to_account_info(),
    };

    // Tick arrays are accounts [6..]
    let tick_arrays: Vec<AccountInfo<'info>> = leg_accounts[6..].to_vec();

    let cpi_ctx = CpiContext::new(dex_program.to_account_info(), cpi_accounts)
        .with_remaining_accounts(tick_arrays);
    // Pass 0 for min_out — krouter enforces slippage at the top level
    kclmm::cpi::swap(cpi_ctx, amount_in, sqrt_price_limit, 0)
}
