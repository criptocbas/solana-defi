use anchor_lang::prelude::*;
use anchor_spl::token::Token;

use crate::constants::*;
use crate::errors::KlevError;
use crate::math;
use crate::state::LeveragedVault;

#[derive(Accounts)]
pub struct LeverageUp<'info> {
    #[account(mut)]
    pub admin: Signer<'info>,

    #[account(
        mut,
        seeds = [LEVERAGED_VAULT_SEED, vault.collateral_mint.as_ref(), vault.debt_mint.as_ref()],
        bump = vault.vault_bump,
        has_one = admin,
        has_one = klend_program,
        has_one = klend_lending_market,
        has_one = klend_collateral_reserve,
        has_one = klend_debt_reserve,
        has_one = cpamm_program,
        has_one = cpamm_pool,
    )]
    pub vault: Box<Account<'info, LeveragedVault>>,

    /// CHECK: PDA signing authority
    #[account(
        mut,
        seeds = [LEV_VAULT_AUTHORITY_SEED, vault.key().as_ref()],
        bump = vault.authority_bump,
    )]
    pub vault_authority: UncheckedAccount<'info>,

    /// CHECK: vault collateral token account, validated by address match
    #[account(mut, address = vault.collateral_token_account)]
    pub collateral_token_account: UncheckedAccount<'info>,

    /// CHECK: vault debt token account, validated by address match
    #[account(mut, address = vault.debt_token_account)]
    pub debt_token_account: UncheckedAccount<'info>,

    // ── klend accounts ──
    /// CHECK: validated by has_one
    pub klend_program: UncheckedAccount<'info>,
    /// CHECK: klend lending market
    pub klend_lending_market: UncheckedAccount<'info>,
    /// CHECK: collateral reserve (SOL) - validated by has_one
    #[account(mut)]
    pub klend_collateral_reserve: UncheckedAccount<'info>,
    /// CHECK: debt reserve (USDC) - validated by has_one
    #[account(mut)]
    pub klend_debt_reserve: UncheckedAccount<'info>,
    /// CHECK: debt reserve authority PDA
    pub klend_debt_reserve_authority: UncheckedAccount<'info>,
    /// CHECK: klend obligation for vault_authority
    #[account(mut)]
    pub klend_obligation: UncheckedAccount<'info>,
    /// CHECK: klend collateral token vault
    #[account(mut)]
    pub klend_collateral_token_vault: UncheckedAccount<'info>,
    /// CHECK: klend debt token vault
    #[account(mut)]
    pub klend_debt_token_vault: UncheckedAccount<'info>,
    /// CHECK: oracle for collateral asset
    pub collateral_oracle: UncheckedAccount<'info>,
    /// CHECK: oracle for debt asset
    pub debt_oracle: UncheckedAccount<'info>,

    // ── cpamm accounts ──
    /// CHECK: validated by has_one
    pub cpamm_program: UncheckedAccount<'info>,
    /// CHECK: cpamm pool
    #[account(mut)]
    pub cpamm_pool: UncheckedAccount<'info>,
    /// CHECK: cpamm pool authority
    pub cpamm_pool_authority: UncheckedAccount<'info>,
    /// CHECK: cpamm vault A
    #[account(mut)]
    pub cpamm_vault_a: UncheckedAccount<'info>,
    /// CHECK: cpamm vault B
    #[account(mut)]
    pub cpamm_vault_b: UncheckedAccount<'info>,
    /// CHECK: input mint for swap (debt_mint)
    pub swap_input_mint: UncheckedAccount<'info>,

    pub token_program: Program<'info, Token>,
    pub system_program: Program<'info, System>,
}

pub fn handle_leverage_up(
    ctx: Context<LeverageUp>,
    collateral_amount: u64,
    borrow_amount: u64,
    min_swap_output: u64,
) -> Result<()> {
    require!(collateral_amount > 0, KlevError::ZeroCollateral);
    require!(borrow_amount > 0, KlevError::ZeroBorrow);

    let vault_key = ctx.accounts.vault.key();
    let authority_bump = ctx.accounts.vault.authority_bump;
    let max_leverage = ctx.accounts.vault.max_leverage_bps;
    let authority_seeds: &[&[u8]] = &[
        LEV_VAULT_AUTHORITY_SEED,
        vault_key.as_ref(),
        &[authority_bump],
    ];
    let signer_seeds = [authority_seeds];

    // Step 1: CPI klend::deposit -- deposit collateral SOL from idle
    let cpi_accounts = klend::cpi::accounts::Deposit {
        user: ctx.accounts.vault_authority.to_account_info(),
        lending_market: ctx.accounts.klend_lending_market.to_account_info(),
        reserve: ctx.accounts.klend_collateral_reserve.to_account_info(),
        obligation: ctx.accounts.klend_obligation.to_account_info(),
        user_token_account: ctx.accounts.collateral_token_account.to_account_info(),
        token_vault: ctx.accounts.klend_collateral_token_vault.to_account_info(),
        token_program: ctx.accounts.token_program.to_account_info(),
        system_program: ctx.accounts.system_program.to_account_info(),
    };
    klend::cpi::deposit(
        CpiContext::new_with_signer(ctx.accounts.klend_program.to_account_info(), cpi_accounts, &signer_seeds),
        collateral_amount,
    )?;

    // Step 2: CPI klend::borrow -- borrow USDC against SOL collateral
    let cpi_accounts = klend::cpi::accounts::Borrow {
        user: ctx.accounts.vault_authority.to_account_info(),
        lending_market: ctx.accounts.klend_lending_market.to_account_info(),
        borrow_reserve: ctx.accounts.klend_debt_reserve.to_account_info(),
        borrow_reserve_authority: ctx.accounts.klend_debt_reserve_authority.to_account_info(),
        collateral_reserve: ctx.accounts.klend_collateral_reserve.to_account_info(),
        obligation: ctx.accounts.klend_obligation.to_account_info(),
        owner: ctx.accounts.vault_authority.to_account_info(),
        borrow_oracle: ctx.accounts.debt_oracle.to_account_info(),
        collateral_oracle: ctx.accounts.collateral_oracle.to_account_info(),
        user_token_account: ctx.accounts.debt_token_account.to_account_info(),
        token_vault: ctx.accounts.klend_debt_token_vault.to_account_info(),
        token_program: ctx.accounts.token_program.to_account_info(),
    };
    // Pass remaining_accounts for klend health check: [collateral_reserve, collateral_oracle, debt_reserve, debt_oracle]
    let health_accounts = vec![
        ctx.accounts.klend_collateral_reserve.to_account_info(),
        ctx.accounts.collateral_oracle.to_account_info(),
        ctx.accounts.klend_debt_reserve.to_account_info(),
        ctx.accounts.debt_oracle.to_account_info(),
    ];
    klend::cpi::borrow(
        CpiContext::new_with_signer(ctx.accounts.klend_program.to_account_info(), cpi_accounts, &signer_seeds)
            .with_remaining_accounts(health_accounts),
        borrow_amount,
    )?;

    // Step 3: CPI cpamm::swap -- swap USDC -> SOL via kpool
    // Read balance before swap using raw account data
    let collateral_balance_before = read_token_balance(&ctx.accounts.collateral_token_account)?;

    let cpi_accounts = cpamm::cpi::accounts::Swap {
        user: ctx.accounts.vault_authority.to_account_info(),
        pool: ctx.accounts.cpamm_pool.to_account_info(),
        pool_authority: ctx.accounts.cpamm_pool_authority.to_account_info(),
        vault_a: ctx.accounts.cpamm_vault_a.to_account_info(),
        vault_b: ctx.accounts.cpamm_vault_b.to_account_info(),
        user_token_in: ctx.accounts.debt_token_account.to_account_info(),
        user_token_out: ctx.accounts.collateral_token_account.to_account_info(),
        input_mint: ctx.accounts.swap_input_mint.to_account_info(),
        token_program: ctx.accounts.token_program.to_account_info(),
    };
    cpamm::cpi::swap(
        CpiContext::new_with_signer(ctx.accounts.cpamm_program.to_account_info(), cpi_accounts, &signer_seeds),
        borrow_amount,
        0,
    )?;

    // Measure swap output
    let collateral_balance_after = read_token_balance(&ctx.accounts.collateral_token_account)?;
    let swap_output = collateral_balance_after
        .checked_sub(collateral_balance_before)
        .ok_or(KlevError::MathUnderflow)?;
    require!(swap_output >= min_swap_output, KlevError::SlippageExceeded);

    // Step 4: CPI klend::deposit -- deposit swapped SOL back into klend
    let cpi_accounts = klend::cpi::accounts::Deposit {
        user: ctx.accounts.vault_authority.to_account_info(),
        lending_market: ctx.accounts.klend_lending_market.to_account_info(),
        reserve: ctx.accounts.klend_collateral_reserve.to_account_info(),
        obligation: ctx.accounts.klend_obligation.to_account_info(),
        user_token_account: ctx.accounts.collateral_token_account.to_account_info(),
        token_vault: ctx.accounts.klend_collateral_token_vault.to_account_info(),
        token_program: ctx.accounts.token_program.to_account_info(),
        system_program: ctx.accounts.system_program.to_account_info(),
    };
    klend::cpi::deposit(
        CpiContext::new_with_signer(ctx.accounts.klend_program.to_account_info(), cpi_accounts, &signer_seeds),
        swap_output,
    )?;

    // Post-check: read klend state and update cache
    let (collateral_underlying, current_debt, net_eq) = read_klend_cache(
        &ctx.accounts.klend_collateral_reserve,
        &ctx.accounts.klend_debt_reserve,
        &ctx.accounts.klend_obligation,
        &ctx.accounts.collateral_oracle,
        &ctx.accounts.debt_oracle,
    )?;

    let leverage = math::leverage_ratio_bps(collateral_underlying, net_eq)?;
    require!(
        leverage <= max_leverage as u64,
        KlevError::MaxLeverageExceeded
    );

    let vault = &mut ctx.accounts.vault;
    vault.cached_collateral_value = collateral_underlying;
    vault.cached_debt_value = current_debt;
    vault.cached_net_equity_collateral = net_eq;

    Ok(())
}

/// Read token account balance from raw account data (avoids large Account<TokenAccount> on stack)
fn read_token_balance(account: &AccountInfo) -> Result<u64> {
    let data = account.try_borrow_data()?;
    // SPL Token Account: amount is at offset 64 (8 bytes, little-endian)
    if data.len() < 72 {
        return err!(KlevError::MathOverflow);
    }
    Ok(u64::from_le_bytes(data[64..72].try_into().unwrap()))
}

/// Read klend state and compute cache values. Extracted to reduce stack frame of caller.
#[inline(never)]
fn read_klend_cache(
    klend_collateral_reserve: &AccountInfo,
    klend_debt_reserve: &AccountInfo,
    klend_obligation: &AccountInfo,
    collateral_oracle: &AccountInfo,
    debt_oracle: &AccountInfo,
) -> Result<(u64, u64, u64)> {
    let coll_data = klend_collateral_reserve.try_borrow_data()?;
    let coll_reserve: klend::state::Reserve =
        klend::state::Reserve::try_deserialize(&mut &coll_data[..])?;

    let debt_data = klend_debt_reserve.try_borrow_data()?;
    let debt_reserve: klend::state::Reserve =
        klend::state::Reserve::try_deserialize(&mut &debt_data[..])?;

    let oblig_data = klend_obligation.try_borrow_data()?;
    let obligation: klend::state::Obligation =
        klend::state::Obligation::try_deserialize(&mut &oblig_data[..])?;

    let coll_reserve_key = klend_collateral_reserve.key();
    let vault_coll_shares = obligation
        .deposits
        .iter()
        .find(|d| d.reserve == coll_reserve_key)
        .map(|d| d.shares)
        .unwrap_or(0);

    let collateral_underlying = math::klend_shares_to_underlying(
        vault_coll_shares,
        coll_reserve.total_shares,
        coll_reserve.total_assets(),
    )?;

    let debt_reserve_key = klend_debt_reserve.key();
    let debt_scaled = obligation
        .borrows
        .iter()
        .find(|b| b.reserve == debt_reserve_key)
        .map(|b| b.borrowed_amount_scaled)
        .unwrap_or(0);
    let current_debt = math::klend_current_debt(debt_scaled, debt_reserve.cumulative_borrow_index)?;

    let coll_oracle_data = collateral_oracle.try_borrow_data()?;
    let coll_oracle: klend::state::MockOracle =
        klend::state::MockOracle::try_deserialize(&mut &coll_oracle_data[..])?;

    let debt_oracle_data_ref = debt_oracle.try_borrow_data()?;
    let debt_oracle_state: klend::state::MockOracle =
        klend::state::MockOracle::try_deserialize(&mut &debt_oracle_data_ref[..])?;

    let debt_in_collateral = math::debt_to_collateral_terms(
        current_debt,
        debt_oracle_state.price,
        debt_oracle_state.decimals,
        coll_oracle.price,
        coll_oracle.decimals,
    )?;

    let net_eq = math::net_equity(collateral_underlying, debt_in_collateral);
    Ok((collateral_underlying, current_debt, net_eq))
}
