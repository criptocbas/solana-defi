use anchor_lang::prelude::*;
use anchor_spl::token::{self, Token, TokenAccount, Transfer};

use crate::constants::*;
use crate::errors::KclmmError;
use crate::math;
use crate::state::Pool;

#[derive(Accounts)]
pub struct Swap<'info> {
    pub user: Signer<'info>,

    #[account(mut)]
    pub pool: Account<'info, Pool>,

    #[account(
        mut,
        constraint = vault_a.key() == pool.vault_a,
    )]
    pub vault_a: Account<'info, TokenAccount>,

    #[account(
        mut,
        constraint = vault_b.key() == pool.vault_b,
    )]
    pub vault_b: Account<'info, TokenAccount>,

    /// CHECK: PDA authority
    #[account(
        seeds = [POOL_AUTHORITY_SEED, pool.key().as_ref()],
        bump = pool.authority_bump,
    )]
    pub pool_authority: UncheckedAccount<'info>,

    #[account(mut)]
    pub user_token_in: Account<'info, TokenAccount>,

    #[account(mut)]
    pub user_token_out: Account<'info, TokenAccount>,

    pub input_mint: Account<'info, anchor_spl::token::Mint>,

    pub token_program: Program<'info, Token>,
    // remaining_accounts: up to 3 TickArray accounts (zero-copy, accessed via raw data)
}

/// Offsets within TickArray account data (after 8-byte discriminator)
/// Layout: pool(32) + start_tick_index(4) + padding(4) + initialized_bitmap(8) + ticks(64*64)
const TA_POOL_OFFSET: usize = 8;
const TA_START_TICK_OFFSET: usize = 40;
const TA_BITMAP_OFFSET: usize = 48;   // 40 + 4 (i32) + 4 (padding for u64 alignment)
const TA_TICKS_OFFSET: usize = 56;    // 48 + 8 (u64 bitmap)
const TICK_SIZE: usize = 64;

pub fn handle_swap(
    ctx: Context<Swap>,
    amount_in: u64,
    sqrt_price_limit: u128,
    minimum_amount_out: u64,
) -> Result<()> {
    require!(amount_in > 0, KclmmError::ZeroSwapInput);

    let pool = &ctx.accounts.pool;
    let a_to_b = ctx.accounts.input_mint.key() == pool.mint_a;
    let b_to_a = ctx.accounts.input_mint.key() == pool.mint_b;
    require!(a_to_b || b_to_a, KclmmError::InvalidInputMint);

    // Validate sqrt_price_limit
    if a_to_b {
        require!(
            sqrt_price_limit < pool.sqrt_price && sqrt_price_limit >= MIN_SQRT_PRICE,
            KclmmError::InvalidSqrtPriceLimit
        );
    } else {
        require!(
            sqrt_price_limit > pool.sqrt_price && sqrt_price_limit <= MAX_SQRT_PRICE,
            KclmmError::InvalidSqrtPriceLimit
        );
    }

    let remaining = &ctx.remaining_accounts;
    require!(remaining.len() <= MAX_TICK_ARRAY_ACCOUNTS, KclmmError::NoMoreTickArrays);

    // Swap state
    let mut sqrt_price = pool.sqrt_price;
    let mut tick_current = pool.tick_current;
    let mut liquidity = pool.liquidity;
    let mut amount_remaining = amount_in;
    let mut amount_out_total: u64 = 0;
    let mut fee_growth_global = if a_to_b {
        pool.fee_growth_global_a
    } else {
        pool.fee_growth_global_b
    };
    let mut protocol_fee_total: u64 = 0;
    let mut tick_crossings: usize = 0;
    let mut array_idx: usize = 0;

    while amount_remaining > 0 && sqrt_price != sqrt_price_limit {
        // Find next initialized tick from remaining_accounts
        let search_result = find_next_tick_raw(
            remaining,
            &mut array_idx,
            tick_current,
            pool.tick_spacing,
            a_to_b,
            pool.key(),
        )?;

        let (next_tick, ta_idx, tick_idx_in_array) = match search_result {
            Some(r) => r,
            None => break, // No more ticks, stop
        };

        let next_sqrt_price = math::tick_to_sqrt_price(next_tick)?;
        let step_target = if a_to_b {
            next_sqrt_price.max(sqrt_price_limit)
        } else {
            next_sqrt_price.min(sqrt_price_limit)
        };

        // Skip if no liquidity
        if liquidity == 0 {
            sqrt_price = step_target;
            if sqrt_price == next_sqrt_price {
                tick_current = if a_to_b { next_tick - 1 } else { next_tick };
                tick_crossings += 1;
                require!(tick_crossings <= MAX_TICK_CROSSINGS, KclmmError::MaxTickCrossingsExceeded);
                cross_tick_raw(
                    remaining, ta_idx, tick_idx_in_array,
                    &mut liquidity, fee_growth_global,
                    if a_to_b { pool.fee_growth_global_b } else { pool.fee_growth_global_a },
                    a_to_b,
                )?;
            }
            continue;
        }

        let step = math::compute_swap_step(
            sqrt_price, step_target, liquidity, amount_remaining, pool.fee_rate,
        )?;

        sqrt_price = step.sqrt_price_next;
        amount_remaining = amount_remaining
            .checked_sub(step.amount_in)
            .and_then(|v| v.checked_sub(step.fee_amount))
            .ok_or(error!(KclmmError::MathOverflow))?;
        amount_out_total = amount_out_total
            .checked_add(step.amount_out)
            .ok_or(error!(KclmmError::MathOverflow))?;

        // Accrue fees
        if step.fee_amount > 0 && liquidity > 0 {
            let protocol_fee = if pool.protocol_fee_rate > 0 {
                (step.fee_amount as u128 * pool.protocol_fee_rate as u128
                    / PROTOCOL_FEE_DENOMINATOR as u128) as u64
            } else {
                0
            };
            protocol_fee_total = protocol_fee_total.saturating_add(protocol_fee);

            let lp_fee = step.fee_amount - protocol_fee;
            if lp_fee > 0 {
                let fee_delta = ((lp_fee as u128) << 64) / liquidity;
                fee_growth_global = fee_growth_global.wrapping_add(fee_delta);
            }
        }

        if sqrt_price == next_sqrt_price {
            tick_current = if a_to_b { next_tick - 1 } else { next_tick };
            tick_crossings += 1;
            require!(tick_crossings <= MAX_TICK_CROSSINGS, KclmmError::MaxTickCrossingsExceeded);
            cross_tick_raw(
                remaining, ta_idx, tick_idx_in_array,
                &mut liquidity, fee_growth_global,
                if a_to_b { pool.fee_growth_global_b } else { pool.fee_growth_global_a },
                a_to_b,
            )?;
        } else {
            tick_current = math::sqrt_price_to_tick(sqrt_price)?;
        }
    }

    require!(amount_out_total >= minimum_amount_out, KclmmError::SlippageExceeded);
    require!(amount_out_total > 0, KclmmError::ZeroOutput);

    // Update pool state
    let pool = &mut ctx.accounts.pool;
    pool.sqrt_price = sqrt_price;
    pool.tick_current = tick_current;
    pool.liquidity = liquidity;
    if a_to_b {
        pool.fee_growth_global_a = fee_growth_global;
        pool.protocol_fees_a = pool.protocol_fees_a.saturating_add(protocol_fee_total);
    } else {
        pool.fee_growth_global_b = fee_growth_global;
        pool.protocol_fees_b = pool.protocol_fees_b.saturating_add(protocol_fee_total);
    }

    // Transfers
    let actual_amount_in = amount_in - amount_remaining;
    let pool_key = pool.key();
    let authority_seeds: &[&[u8]] = &[
        POOL_AUTHORITY_SEED,
        pool_key.as_ref(),
        &[pool.authority_bump],
    ];

    token::transfer(
        CpiContext::new(
            ctx.accounts.token_program.to_account_info(),
            Transfer {
                from: ctx.accounts.user_token_in.to_account_info(),
                to: if a_to_b {
                    ctx.accounts.vault_a.to_account_info()
                } else {
                    ctx.accounts.vault_b.to_account_info()
                },
                authority: ctx.accounts.user.to_account_info(),
            },
        ),
        actual_amount_in,
    )?;

    token::transfer(
        CpiContext::new_with_signer(
            ctx.accounts.token_program.to_account_info(),
            Transfer {
                from: if a_to_b {
                    ctx.accounts.vault_b.to_account_info()
                } else {
                    ctx.accounts.vault_a.to_account_info()
                },
                to: ctx.accounts.user_token_out.to_account_info(),
                authority: ctx.accounts.pool_authority.to_account_info(),
            },
            &[authority_seeds],
        ),
        amount_out_total,
    )?;

    Ok(())
}

/// Find next initialized tick using raw account data access (avoids stack-allocating TickArray)
fn find_next_tick_raw(
    accounts: &[AccountInfo],
    array_idx: &mut usize,
    current_tick: i32,
    tick_spacing: u16,
    a_to_b: bool,
    pool_key: Pubkey,
) -> Result<Option<(i32, usize, usize)>> {
    let spacing_i32 = tick_spacing as i32;

    while *array_idx < accounts.len() {
        let data = accounts[*array_idx].try_borrow_data()?;
        if data.len() < TA_TICKS_OFFSET {
            *array_idx += 1;
            continue;
        }

        // Read pool pubkey and validate
        let ta_pool = Pubkey::try_from(&data[TA_POOL_OFFSET..TA_POOL_OFFSET + 32]).unwrap();
        require!(ta_pool == pool_key, KclmmError::TickArrayPoolMismatch);

        let start_tick = i32::from_le_bytes(data[TA_START_TICK_OFFSET..TA_START_TICK_OFFSET + 4].try_into().unwrap());
        let bitmap = u64::from_le_bytes(data[TA_BITMAP_OFFSET..TA_BITMAP_OFFSET + 8].try_into().unwrap());

        // Search bitmap for next initialized tick
        if a_to_b {
            let offset = current_tick - start_tick;
            if offset >= 0 {
                let idx = (offset / spacing_i32) as usize;
                let search_start = idx.min(TICKS_PER_ARRAY - 1);
                if let Some(i) = math::prev_set_bit(bitmap, search_start) {
                    let tick = start_tick + i as i32 * spacing_i32;
                    return Ok(Some((tick, *array_idx, i)));
                }
            }
        } else {
            let offset = current_tick - start_tick;
            let idx = if offset < 0 {
                0usize
            } else {
                (offset / spacing_i32 + 1) as usize
            };
            if idx < TICKS_PER_ARRAY {
                if let Some(i) = math::next_set_bit(bitmap, idx) {
                    let tick = start_tick + i as i32 * spacing_i32;
                    return Ok(Some((tick, *array_idx, i)));
                }
            }
        }

        *array_idx += 1;
    }

    Ok(None)
}

/// Cross a tick using raw account data access
fn cross_tick_raw(
    accounts: &[AccountInfo],
    array_idx: usize,
    tick_idx: usize,
    liquidity: &mut u128,
    fee_growth_global_input: u128,
    fee_growth_global_other: u128,
    a_to_b: bool,
) -> Result<()> {
    if array_idx >= accounts.len() {
        return Ok(());
    }

    let mut data = accounts[array_idx].try_borrow_mut_data()?;
    let tick_offset = TA_TICKS_OFFSET + tick_idx * TICK_SIZE;

    if tick_offset + TICK_SIZE > data.len() {
        return err!(KclmmError::TickNotInArray);
    }

    let liquidity_net = i128::from_le_bytes(data[tick_offset..tick_offset + 16].try_into().unwrap());
    let fee_outside_a = u128::from_le_bytes(data[tick_offset + 32..tick_offset + 48].try_into().unwrap());
    let fee_outside_b = u128::from_le_bytes(data[tick_offset + 48..tick_offset + 64].try_into().unwrap());

    // Flip fee_growth_outside
    // When crossing a_to_b: input token is A, other is B
    // When crossing b_to_a: input token is B, other is A
    let (new_fee_outside_a, new_fee_outside_b) = if a_to_b {
        (
            fee_growth_global_input.wrapping_sub(fee_outside_a),
            fee_growth_global_other.wrapping_sub(fee_outside_b),
        )
    } else {
        (
            fee_growth_global_other.wrapping_sub(fee_outside_a),
            fee_growth_global_input.wrapping_sub(fee_outside_b),
        )
    };

    data[tick_offset + 32..tick_offset + 48].copy_from_slice(&new_fee_outside_a.to_le_bytes());
    data[tick_offset + 48..tick_offset + 64].copy_from_slice(&new_fee_outside_b.to_le_bytes());

    // Update active liquidity
    if a_to_b {
        *liquidity = ((*liquidity as i128).checked_sub(liquidity_net)
            .ok_or(error!(KclmmError::MathOverflow))?) as u128;
    } else {
        *liquidity = ((*liquidity as i128).checked_add(liquidity_net)
            .ok_or(error!(KclmmError::MathOverflow))?) as u128;
    }

    Ok(())
}
