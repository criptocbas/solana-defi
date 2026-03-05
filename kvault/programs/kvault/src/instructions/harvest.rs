use anchor_lang::prelude::*;
use anchor_spl::token::{self, Mint, MintTo, Token, TokenAccount};

use crate::constants::*;
use crate::errors::KvaultError;
use crate::math;
use crate::state::Vault;

#[derive(Accounts)]
pub struct Harvest<'info> {
    #[account(mut)]
    pub admin: Signer<'info>,

    #[account(
        mut,
        seeds = [VAULT_SEED, vault.underlying_mint.as_ref()],
        bump = vault.vault_bump,
        has_one = admin,
        has_one = vault_token_account,
        has_one = klend_reserve,
    )]
    pub vault: Account<'info, Vault>,

    /// CHECK: PDA signing authority
    #[account(
        seeds = [VAULT_AUTHORITY_SEED, vault.key().as_ref()],
        bump = vault.authority_bump,
    )]
    pub vault_authority: UncheckedAccount<'info>,

    #[account(
        mut,
        address = vault.share_mint,
    )]
    pub share_mint: Account<'info, Mint>,

    pub vault_token_account: Account<'info, TokenAccount>,

    /// The klend reserve -- read to compute current invested value
    /// CHECK: validated by has_one on vault
    pub klend_reserve: UncheckedAccount<'info>,

    /// The klend obligation for vault_authority
    /// CHECK: read to get obligation deposit shares
    pub klend_obligation: UncheckedAccount<'info>,

    /// Fee recipient receives minted fee shares
    #[account(
        mut,
        token::mint = vault.share_mint,
        token::authority = vault.fee_recipient,
    )]
    pub fee_recipient_share_account: Account<'info, TokenAccount>,

    pub token_program: Program<'info, Token>,
}

pub fn handle_harvest(ctx: Context<Harvest>) -> Result<()> {
    let clock = Clock::get()?;

    // Deserialize klend reserve to get total_shares and total_assets
    let klend_reserve_info = &ctx.accounts.klend_reserve;
    let klend_reserve_data = klend_reserve_info.try_borrow_data()?;
    // Skip 8-byte discriminator
    let klend_reserve: klend::state::Reserve =
        klend::state::Reserve::try_deserialize(&mut &klend_reserve_data[..])?;

    // Deserialize klend obligation to get vault's deposit shares
    let klend_obligation_info = &ctx.accounts.klend_obligation;
    let klend_obligation_data = klend_obligation_info.try_borrow_data()?;
    let klend_obligation: klend::state::Obligation =
        klend::state::Obligation::try_deserialize(&mut &klend_obligation_data[..])?;

    // Find vault's deposit in the obligation for this reserve
    let reserve_key = ctx.accounts.klend_reserve.key();
    let vault_klend_shares = klend_obligation
        .deposits
        .iter()
        .find(|d| d.reserve == reserve_key)
        .map(|d| d.shares)
        .unwrap_or(0);

    // Compute current invested value from klend shares
    let current_invested = math::klend_shares_to_underlying(
        vault_klend_shares,
        klend_reserve.total_shares,
        klend_reserve.total_assets(),
    )?;

    // Capture immutable reads before mutable borrow
    let vault_key = ctx.accounts.vault.key();
    let previous_invested = ctx.accounts.vault.total_invested;
    let performance_fee_bps = ctx.accounts.vault.performance_fee_bps;
    let management_fee_bps = ctx.accounts.vault.management_fee_bps;
    let last_harvest_timestamp = ctx.accounts.vault.last_harvest_timestamp;
    let authority_bump = ctx.accounts.vault.authority_bump;

    // Compute yield (only positive yield triggers fees)
    let yield_amount = current_invested.saturating_sub(previous_invested);

    // Compute total assets for fee share calculation
    let idle = ctx.accounts.vault_token_account.amount;
    let total_assets = idle
        .checked_add(current_invested)
        .ok_or(KvaultError::MathOverflow)?;

    let supply = ctx.accounts.share_mint.supply;

    // Performance fee: yield * performance_fee_bps / BPS_SCALE
    let perf_fee_underlying = (yield_amount as u128)
        .checked_mul(performance_fee_bps as u128)
        .ok_or(KvaultError::MathOverflow)?
        / (BPS_SCALE as u128);

    // Management fee: total_assets * management_fee_bps * elapsed / (BPS_SCALE * SECONDS_PER_YEAR)
    let elapsed = clock
        .unix_timestamp
        .saturating_sub(last_harvest_timestamp) as u128;
    let mgmt_fee_underlying = (total_assets as u128)
        .checked_mul(management_fee_bps as u128)
        .ok_or(KvaultError::MathOverflow)?
        .checked_mul(elapsed)
        .ok_or(KvaultError::MathOverflow)?
        / ((BPS_SCALE as u128)
            .checked_mul(SECONDS_PER_YEAR)
            .ok_or(KvaultError::MathOverflow)?);

    let total_fee_underlying = (perf_fee_underlying as u64)
        .checked_add(mgmt_fee_underlying as u64)
        .ok_or(KvaultError::MathOverflow)?;

    // Mint dilutive fee shares
    if total_fee_underlying > 0 {
        let fee_shares_to_mint = math::fee_shares(total_fee_underlying, supply, total_assets)?;

        if fee_shares_to_mint > 0 {
            let authority_seeds: &[&[u8]] = &[
                VAULT_AUTHORITY_SEED,
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

    // Update vault state
    let vault = &mut ctx.accounts.vault;
    vault.total_invested = current_invested;
    vault.last_harvest_timestamp = clock.unix_timestamp;

    Ok(())
}
