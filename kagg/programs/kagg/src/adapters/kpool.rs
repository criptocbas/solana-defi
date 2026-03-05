use anchor_lang::prelude::*;

use crate::errors::KaggError;

/// Execute a swap through kpool (constant product AMM).
///
/// Expected accounts in step_accounts slice:
///   [0] cpamm program
///   [1] pool (mut)
///   [2] pool_authority
///   [3] vault_a (mut)
///   [4] vault_b (mut)
///   [5] input_mint
pub fn execute_swap<'info>(
    step_accounts: &[AccountInfo<'info>],
    user: &AccountInfo<'info>,
    user_token_in: &AccountInfo<'info>,
    user_token_out: &AccountInfo<'info>,
    token_program: &AccountInfo<'info>,
    amount_in: u64,
) -> Result<()> {
    require!(step_accounts.len() >= 6, KaggError::InsufficientAccounts);

    let dex_program = &step_accounts[0];
    let pool = &step_accounts[1];
    let pool_authority = &step_accounts[2];
    let vault_a = &step_accounts[3];
    let vault_b = &step_accounts[4];
    let input_mint = &step_accounts[5];

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
    cpamm::cpi::swap(cpi_ctx, amount_in, 0)
}
