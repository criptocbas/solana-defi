use anchor_lang::prelude::*;
use anchor_spl::token::{Token, TokenAccount};

pub mod adapters;
pub mod errors;
pub mod types;

use adapters::dispatch_swap;
use errors::KaggError;
use types::RoutePlanStep;

declare_id!("3YJj1erVbKvjEJxJEMarh7dDYBQTrq7fNCcWdTPjuWLn");

#[program]
pub mod kagg {
    use super::*;

    /// Execute an arbitrary N-step route plan.
    ///
    /// Each step in the route plan specifies:
    /// - Which DEX to swap through
    /// - How many remaining_accounts it needs
    /// - Input/output token indices into remaining_accounts
    /// - Amount to swap (0 = use previous step's output)
    /// - Extra data (e.g. sqrt_price_limit for CLMM)
    ///
    /// Accounts:
    ///   [0] user (signer)
    ///   [1] user_token_source (mut) — input token account
    ///   [2] user_token_destination (mut) — output token account
    ///   [3] token_program
    ///   remaining_accounts: [token_accounts..., pool_step_accounts...]
    ///
    /// The token_ledger_len specifies how many of the first remaining_accounts
    /// are intermediate token accounts. Pool step accounts follow after.
    pub fn execute_route<'info>(
        ctx: Context<'_, '_, 'info, 'info, ExecuteRoute<'info>>,
        route_plan: Vec<RoutePlanStep>,
        token_ledger_len: u8,
        minimum_amount_out: u64,
    ) -> Result<()> {
        require!(!route_plan.is_empty(), KaggError::EmptyRoutePlan);

        let remaining = ctx.remaining_accounts;
        let ledger_len = token_ledger_len as usize;

        // Validate all token ledger indices are within remaining_accounts
        require!(remaining.len() >= ledger_len, KaggError::InvalidTokenLedgerIndex);

        // Record destination balance before
        let dest_before = read_token_balance(&ctx.accounts.user_token_destination.to_account_info())?;

        // Track intermediate amounts: step outputs indexed by step number
        let mut step_outputs: Vec<u64> = Vec::with_capacity(route_plan.len());

        // Cursor into remaining_accounts after the token ledger
        let mut account_cursor = ledger_len;

        for (i, step) in route_plan.iter().enumerate() {
            let num = step.num_accounts as usize;
            require!(
                account_cursor + num <= remaining.len(),
                KaggError::InsufficientAccounts
            );

            let step_accounts = &remaining[account_cursor..account_cursor + num];
            account_cursor += num;

            // Determine amount_in
            let amount_in = if step.amount_in > 0 {
                step.amount_in
            } else if i > 0 {
                // Use previous step's output
                step_outputs[i - 1]
            } else {
                return Err(error!(KaggError::ZeroSwapAmount));
            };

            require!(amount_in > 0, KaggError::ZeroSwapAmount);

            // Resolve input/output token accounts
            let user_token_in = resolve_token_account(
                step.input_token_index,
                &ctx.accounts.user_token_source,
                &ctx.accounts.user_token_destination,
                remaining,
                ledger_len,
            )?;
            let user_token_out = resolve_token_account(
                step.output_token_index,
                &ctx.accounts.user_token_source,
                &ctx.accounts.user_token_destination,
                remaining,
                ledger_len,
            )?;

            // Record output balance before swap
            let out_before = read_token_balance(&user_token_out)?;

            dispatch_swap(
                step.dex_id,
                step_accounts,
                &ctx.accounts.user.to_account_info(),
                &user_token_in,
                &user_token_out,
                &ctx.accounts.token_program.to_account_info(),
                amount_in,
                &step.extra_data,
            )?;

            // Read output balance after swap
            let out_after = read_token_balance(&user_token_out)?;
            let delta = out_after
                .checked_sub(out_before)
                .ok_or(KaggError::MathOverflow)?;

            step_outputs.push(delta);
        }

        // Enforce end-to-end slippage on destination
        let dest_after = read_token_balance(&ctx.accounts.user_token_destination.to_account_info())?;
        let actual_out = dest_after
            .checked_sub(dest_before)
            .ok_or(KaggError::MathOverflow)?;

        require!(actual_out >= minimum_amount_out, KaggError::SlippageExceeded);

        Ok(())
    }
}

#[derive(Accounts)]
pub struct ExecuteRoute<'info> {
    pub user: Signer<'info>,

    /// User's source (input) token account
    #[account(mut)]
    pub user_token_source: Account<'info, TokenAccount>,

    /// User's destination (output) token account
    #[account(mut)]
    pub user_token_destination: Account<'info, TokenAccount>,

    pub token_program: Program<'info, Token>,

    // remaining_accounts:
    //   [0..token_ledger_len] = intermediate token accounts (for multi-hop)
    //   [token_ledger_len..] = pool step accounts (consumed sequentially by route_plan steps)
}

/// Resolve a token account by index.
/// Index 0 = user_token_source, 1 = user_token_destination,
/// 2..2+token_ledger_len = intermediate accounts from remaining_accounts[0..ledger_len]
fn resolve_token_account<'info>(
    index: u8,
    source: &Account<'info, TokenAccount>,
    destination: &Account<'info, TokenAccount>,
    remaining: &[AccountInfo<'info>],
    ledger_len: usize,
) -> Result<AccountInfo<'info>> {
    match index {
        0 => Ok(source.to_account_info()),
        1 => Ok(destination.to_account_info()),
        n => {
            let ledger_idx = (n - 2) as usize;
            require!(ledger_idx < ledger_len, KaggError::InvalidTokenLedgerIndex);
            Ok(remaining[ledger_idx].to_account_info())
        }
    }
}

/// Read the token balance from raw account data (bytes 64..72) to avoid
/// full TokenAccount deserialization on the BPF stack.
fn read_token_balance(account: &AccountInfo) -> Result<u64> {
    let data = account.try_borrow_data()?;
    require!(data.len() >= 72, KaggError::InvalidTokenLedgerIndex);
    let amount_bytes: [u8; 8] = data[64..72].try_into().unwrap();
    Ok(u64::from_le_bytes(amount_bytes))
}
