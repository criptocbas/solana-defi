use anchor_lang::prelude::*;
use anchor_spl::token::{self, Mint, MintTo, Token, TokenAccount};

use crate::constants::*;
use crate::errors::KlevError;
use crate::math;
use crate::state::LeveragedVault;

#[derive(Accounts)]
pub struct Harvest<'info> {
    #[account(mut)]
    pub admin: Signer<'info>,

    #[account(
        mut,
        seeds = [LEVERAGED_VAULT_SEED, vault.collateral_mint.as_ref(), vault.debt_mint.as_ref()],
        bump = vault.vault_bump,
        has_one = admin,
        has_one = collateral_token_account,
        has_one = klend_collateral_reserve,
        has_one = klend_debt_reserve,
    )]
    pub vault: Account<'info, LeveragedVault>,

    /// CHECK: PDA signing authority
    #[account(
        seeds = [LEV_VAULT_AUTHORITY_SEED, vault.key().as_ref()],
        bump = vault.authority_bump,
    )]
    pub vault_authority: UncheckedAccount<'info>,

    #[account(
        mut,
        address = vault.share_mint,
    )]
    pub share_mint: Account<'info, Mint>,

    pub collateral_token_account: Account<'info, TokenAccount>,

    /// The klend collateral reserve -- read to compute current collateral value
    /// CHECK: validated by has_one on vault
    pub klend_collateral_reserve: UncheckedAccount<'info>,

    /// The klend debt reserve -- read to compute current debt value
    /// CHECK: validated by has_one on vault
    pub klend_debt_reserve: UncheckedAccount<'info>,

    /// The klend obligation for vault_authority
    /// CHECK: read to get obligation deposits and borrows
    pub klend_obligation: UncheckedAccount<'info>,

    /// CHECK: oracle for collateral asset
    pub collateral_oracle: UncheckedAccount<'info>,

    /// CHECK: oracle for debt asset
    pub debt_oracle: UncheckedAccount<'info>,

    /// Fee recipient receives minted fee shares
    #[account(
        mut,
        token::mint = vault.share_mint,
    )]
    pub fee_recipient_share_account: Account<'info, TokenAccount>,

    pub token_program: Program<'info, Token>,
}

pub fn handle_harvest(ctx: Context<Harvest>) -> Result<()> {
    let clock = Clock::get()?;

    // ── Deserialize klend state ──

    let klend_coll_reserve_info = &ctx.accounts.klend_collateral_reserve;
    let klend_coll_reserve_data = klend_coll_reserve_info.try_borrow_data()?;
    let coll_reserve: klend::state::Reserve =
        klend::state::Reserve::try_deserialize(&mut &klend_coll_reserve_data[..])?;

    let klend_debt_reserve_info = &ctx.accounts.klend_debt_reserve;
    let klend_debt_reserve_data = klend_debt_reserve_info.try_borrow_data()?;
    let debt_reserve: klend::state::Reserve =
        klend::state::Reserve::try_deserialize(&mut &klend_debt_reserve_data[..])?;

    // Try to read obligation. If it doesn't exist yet (no leverage position),
    // collateral and debt are both 0.
    let (collateral_underlying, current_debt, current_net_equity) = {
        let klend_obligation_info = &ctx.accounts.klend_obligation;
        let obligation_data_result = klend_obligation_info.try_borrow_data();

        if let Ok(klend_obligation_data) = obligation_data_result {
            if klend_obligation_data.len() > 8 {
                if let Ok(obligation) = klend::state::Obligation::try_deserialize(&mut &klend_obligation_data[..]) {
                    let coll_reserve_key = ctx.accounts.klend_collateral_reserve.key();
                    let vault_coll_shares = obligation
                        .deposits
                        .iter()
                        .find(|d| d.reserve == coll_reserve_key)
                        .map(|d| d.shares)
                        .unwrap_or(0);

                    let coll_underlying = math::klend_shares_to_underlying(
                        vault_coll_shares,
                        coll_reserve.total_shares,
                        coll_reserve.total_assets(),
                    )?;

                    let debt_reserve_key = ctx.accounts.klend_debt_reserve.key();
                    let debt_scaled = obligation
                        .borrows
                        .iter()
                        .find(|b| b.reserve == debt_reserve_key)
                        .map(|b| b.borrowed_amount_scaled)
                        .unwrap_or(0);
                    let curr_debt = math::klend_current_debt(debt_scaled, debt_reserve.cumulative_borrow_index)?;

                    let coll_oracle_data = ctx.accounts.collateral_oracle.try_borrow_data()?;
                    let coll_oracle: klend::state::MockOracle =
                        klend::state::MockOracle::try_deserialize(&mut &coll_oracle_data[..])?;

                    let debt_oracle_data = ctx.accounts.debt_oracle.try_borrow_data()?;
                    let debt_oracle_state: klend::state::MockOracle =
                        klend::state::MockOracle::try_deserialize(&mut &debt_oracle_data[..])?;

                    let debt_in_coll = math::debt_to_collateral_terms(
                        curr_debt,
                        debt_oracle_state.price,
                        debt_oracle_state.decimals,
                        coll_oracle.price,
                        coll_oracle.decimals,
                    )?;

                    let net_eq = math::net_equity(coll_underlying, debt_in_coll);
                    (coll_underlying, curr_debt, net_eq)
                } else {
                    (0u64, 0u64, 0u64)
                }
            } else {
                (0u64, 0u64, 0u64)
            }
        } else {
            (0u64, 0u64, 0u64)
        }
    };

    // ── Compute yield and fees ──
    let vault_key = ctx.accounts.vault.key();
    let previous_net_equity = ctx.accounts.vault.cached_net_equity_collateral;
    let performance_fee_bps = ctx.accounts.vault.performance_fee_bps;
    let management_fee_bps = ctx.accounts.vault.management_fee_bps;
    let last_harvest_timestamp = ctx.accounts.vault.last_harvest_timestamp;
    let authority_bump = ctx.accounts.vault.authority_bump;

    // Only positive yield triggers performance fee
    let yield_amount = current_net_equity.saturating_sub(previous_net_equity);

    // Total assets = idle + net_equity
    let idle = ctx.accounts.collateral_token_account.amount;
    let total_assets = idle
        .checked_add(current_net_equity)
        .ok_or(KlevError::MathOverflow)?;

    let supply = ctx.accounts.share_mint.supply;

    // Performance fee: yield * performance_fee_bps / BPS_SCALE
    let perf_fee_underlying = (yield_amount as u128)
        .checked_mul(performance_fee_bps as u128)
        .ok_or(KlevError::MathOverflow)?
        / (BPS_SCALE as u128);

    // Management fee: total_assets * management_fee_bps * elapsed / (BPS_SCALE * SECONDS_PER_YEAR)
    let elapsed = clock
        .unix_timestamp
        .saturating_sub(last_harvest_timestamp) as u128;
    let mgmt_fee_underlying = (total_assets as u128)
        .checked_mul(management_fee_bps as u128)
        .ok_or(KlevError::MathOverflow)?
        .checked_mul(elapsed)
        .ok_or(KlevError::MathOverflow)?
        / ((BPS_SCALE as u128)
            .checked_mul(SECONDS_PER_YEAR)
            .ok_or(KlevError::MathOverflow)?);

    let total_fee_underlying = (perf_fee_underlying as u64)
        .checked_add(mgmt_fee_underlying as u64)
        .ok_or(KlevError::MathOverflow)?;

    // Mint dilutive fee shares
    if total_fee_underlying > 0 {
        let fee_shares_to_mint = math::fee_shares(total_fee_underlying, supply, total_assets)?;

        if fee_shares_to_mint > 0 {
            let authority_seeds: &[&[u8]] = &[
                LEV_VAULT_AUTHORITY_SEED,
                vault_key.as_ref(),
                &[authority_bump],
            ];
            let signer_seeds = [authority_seeds];

            let cpi_accounts = MintTo {
                mint: ctx.accounts.share_mint.to_account_info(),
                to: ctx.accounts.fee_recipient_share_account.to_account_info(),
                authority: ctx.accounts.vault_authority.to_account_info(),
            };
            let cpi_ctx = CpiContext::new_with_signer(
                ctx.accounts.token_program.to_account_info(),
                cpi_accounts,
                &signer_seeds,
            );
            token::mint_to(cpi_ctx, fee_shares_to_mint)?;
        }
    }

    // ── Update vault state ──
    let vault = &mut ctx.accounts.vault;
    vault.cached_collateral_value = collateral_underlying;
    vault.cached_debt_value = current_debt;
    vault.cached_net_equity_collateral = current_net_equity;
    vault.last_harvest_timestamp = clock.unix_timestamp;

    Ok(())
}
