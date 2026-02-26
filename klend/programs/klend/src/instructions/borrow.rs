use anchor_lang::prelude::*;
use anchor_spl::token::{self, Token, TokenAccount, Transfer};

use crate::constants::*;
use crate::errors::KlendError;
use crate::math;
use crate::state::{LendingMarket, MockOracle, Obligation, ObligationBorrow, Reserve};

#[derive(Accounts)]
pub struct Borrow<'info> {
    #[account(mut)]
    pub user: Signer<'info>,

    #[account(
        seeds = [LENDING_MARKET_SEED, lending_market.admin.as_ref()],
        bump = lending_market.bump,
    )]
    pub lending_market: Account<'info, LendingMarket>,

    /// The reserve to borrow from
    #[account(
        mut,
        seeds = [RESERVE_SEED, lending_market.key().as_ref(), borrow_reserve.token_mint.as_ref()],
        bump = borrow_reserve.bump,
        has_one = lending_market,
        has_one = token_vault,
    )]
    pub borrow_reserve: Account<'info, Reserve>,

    /// CHECK: PDA signing authority for borrow vault
    #[account(
        seeds = [RESERVE_AUTHORITY_SEED, borrow_reserve.key().as_ref()],
        bump = borrow_reserve.authority_bump,
    )]
    pub borrow_reserve_authority: UncheckedAccount<'info>,

    /// The reserve where user has collateral
    #[account(
        seeds = [RESERVE_SEED, lending_market.key().as_ref(), collateral_reserve.token_mint.as_ref()],
        bump = collateral_reserve.bump,
        has_one = lending_market,
    )]
    pub collateral_reserve: Account<'info, Reserve>,

    #[account(
        mut,
        seeds = [OBLIGATION_SEED, lending_market.key().as_ref(), user.key().as_ref()],
        bump = obligation.bump,
        has_one = owner @ KlendError::NoCollateralDeposit,
    )]
    pub obligation: Account<'info, Obligation>,

    /// CHECK: validated by has_one
    pub owner: AccountInfo<'info>,

    /// Oracle for the borrow asset
    pub borrow_oracle: Account<'info, MockOracle>,

    /// Oracle for the collateral asset
    pub collateral_oracle: Account<'info, MockOracle>,

    #[account(mut)]
    pub user_token_account: Account<'info, TokenAccount>,

    #[account(mut)]
    pub token_vault: Account<'info, TokenAccount>,

    pub token_program: Program<'info, Token>,
}

pub fn handle_borrow(ctx: Context<Borrow>, amount: u64) -> Result<()> {
    require!(amount > 0, KlendError::ZeroBorrow);

    let borrow_reserve = &ctx.accounts.borrow_reserve;
    let collateral_reserve = &ctx.accounts.collateral_reserve;
    let clock = Clock::get()?;

    // Check reserve freshness
    require!(
        clock.unix_timestamp.saturating_sub(borrow_reserve.last_update_timestamp)
            <= RESERVE_FRESHNESS_SECONDS,
        KlendError::ReserveStale
    );
    require!(
        clock.unix_timestamp.saturating_sub(collateral_reserve.last_update_timestamp)
            <= RESERVE_FRESHNESS_SECONDS,
        KlendError::ReserveStale
    );

    // Validate oracle mints match reserves
    require!(
        ctx.accounts.borrow_oracle.token_mint == borrow_reserve.token_mint,
        KlendError::InvalidOracle
    );
    require!(
        ctx.accounts.collateral_oracle.token_mint == collateral_reserve.token_mint,
        KlendError::InvalidOracle
    );

    // Check oracle staleness
    let borrow_oracle_staleness = clock
        .unix_timestamp
        .saturating_sub(ctx.accounts.borrow_oracle.timestamp) as u64;
    require!(
        borrow_oracle_staleness <= borrow_reserve.config.oracle_max_staleness,
        KlendError::OracleStale
    );
    let collateral_oracle_staleness = clock
        .unix_timestamp
        .saturating_sub(ctx.accounts.collateral_oracle.timestamp) as u64;
    require!(
        collateral_oracle_staleness <= collateral_reserve.config.oracle_max_staleness,
        KlendError::OracleStale
    );

    // Check borrow cap
    let new_borrowed = borrow_reserve
        .borrowed_liquidity
        .checked_add(amount)
        .ok_or(KlendError::MathOverflow)?;
    if borrow_reserve.config.borrow_cap > 0 {
        require!(
            new_borrowed <= borrow_reserve.config.borrow_cap,
            KlendError::BorrowCapExceeded
        );
    }

    // Check vault has enough liquidity
    require!(
        amount <= borrow_reserve.available_liquidity(),
        KlendError::InsufficientLiquidity
    );

    // Compute collateral value
    let obligation = &ctx.accounts.obligation;
    let collateral_reserve_key = collateral_reserve.key();
    let collateral_deposit = obligation
        .deposits
        .iter()
        .find(|d| d.reserve == collateral_reserve_key)
        .ok_or(KlendError::NoCollateralDeposit)?;

    let collateral_underlying = math::shares_to_underlying(
        collateral_deposit.shares,
        collateral_reserve.total_shares,
        collateral_reserve.total_assets(),
    )?;

    let collateral_oracle = &ctx.accounts.collateral_oracle;
    let collateral_value = math::collateral_value_usd(
        collateral_underlying,
        collateral_oracle.price,
        collateral_oracle.decimals,
    )?;
    let weighted_collateral = math::weighted_collateral_value(
        collateral_value,
        collateral_reserve.config.liquidation_threshold,
    )?;

    // Compute existing debt value + new borrow
    let borrow_oracle = &ctx.accounts.borrow_oracle;
    let mut total_debt_value: u128 = 0;

    // Existing borrows
    for b in &obligation.borrows {
        // For simplicity in v1, we use the borrow oracle for the borrow reserve
        // and assume all borrows are from the same reserve
        let current_debt = b.current_debt(borrow_reserve.cumulative_borrow_index);
        let debt_val = math::collateral_value_usd(
            current_debt,
            borrow_oracle.price,
            borrow_oracle.decimals,
        )?;
        total_debt_value = total_debt_value
            .checked_add(debt_val)
            .ok_or(KlendError::MathOverflow)?;
    }

    // Add new borrow
    let new_borrow_value = math::collateral_value_usd(
        amount,
        borrow_oracle.price,
        borrow_oracle.decimals,
    )?;
    total_debt_value = total_debt_value
        .checked_add(new_borrow_value)
        .ok_or(KlendError::MathOverflow)?;

    // Health factor check
    let hf = math::health_factor(weighted_collateral, total_debt_value)?;
    require!(hf >= SCALE, KlendError::HealthFactorTooLow);

    // Transfer tokens from vault to user
    let reserve_key_bytes = borrow_reserve.key();
    let authority_seeds: &[&[u8]] = &[
        RESERVE_AUTHORITY_SEED,
        reserve_key_bytes.as_ref(),
        &[borrow_reserve.authority_bump],
    ];

    let cpi_accounts = Transfer {
        from: ctx.accounts.token_vault.to_account_info(),
        to: ctx.accounts.user_token_account.to_account_info(),
        authority: ctx.accounts.borrow_reserve_authority.to_account_info(),
    };
    let signer_seeds = [authority_seeds];
    let cpi_ctx = CpiContext::new_with_signer(
        ctx.accounts.token_program.to_account_info(),
        cpi_accounts,
        &signer_seeds,
    );
    token::transfer(cpi_ctx, amount)?;

    // Update borrow reserve
    let borrow_reserve = &mut ctx.accounts.borrow_reserve;
    borrow_reserve.borrowed_liquidity = new_borrowed;
    borrow_reserve.deposited_liquidity = borrow_reserve
        .deposited_liquidity
        .checked_sub(amount)
        .ok_or(KlendError::MathUnderflow)?;

    // Update obligation - add/update borrow entry
    let obligation = &mut ctx.accounts.obligation;
    let borrow_reserve_key = borrow_reserve.key();
    let borrow_index = borrow_reserve.cumulative_borrow_index;

    // Scale amount by current index: scaled = amount * SCALE / index
    let scaled_amount = (amount as u128)
        .checked_mul(SCALE)
        .ok_or(KlendError::MathOverflow)?
        / borrow_index;

    if let Some(borrow) = obligation
        .borrows
        .iter_mut()
        .find(|b| b.reserve == borrow_reserve_key)
    {
        borrow.borrowed_amount_scaled = borrow
            .borrowed_amount_scaled
            .checked_add(scaled_amount)
            .ok_or(KlendError::MathOverflow)?;
    } else {
        require!(
            obligation.borrows.len() < MAX_BORROWS,
            KlendError::MaxEntriesReached
        );
        obligation.borrows.push(ObligationBorrow {
            reserve: borrow_reserve_key,
            borrowed_amount_scaled: scaled_amount,
        });
    }

    Ok(())
}
