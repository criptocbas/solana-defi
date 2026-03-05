use crate::constants::*;
use crate::errors::KclmmError;
use crate::state::{Tick, TickArray};
use anchor_lang::prelude::*;

// ============================================================================
// U256 helpers (manual 256-bit via (u128, u128) = (hi, lo))
// ============================================================================

/// 128×128 → 256-bit multiply, returns (hi, lo)
pub fn mul_u128(a: u128, b: u128) -> (u128, u128) {
    let a_lo = a as u64 as u128;
    let a_hi = a >> 64;
    let b_lo = b as u64 as u128;
    let b_hi = b >> 64;

    let lo_lo = a_lo * b_lo;
    let lo_hi = a_lo * b_hi;
    let hi_lo = a_hi * b_lo;
    let hi_hi = a_hi * b_hi;

    let mid = lo_hi + (lo_lo >> 64);
    let mid2 = (mid as u64 as u128) + hi_lo;

    let lo = ((mid2 as u64 as u128) << 64) | (lo_lo as u64 as u128);
    let hi = hi_hi + (mid >> 64) + (mid2 >> 64);

    (hi, lo)
}

/// 256÷128 → 128-bit divide. Requires result fits in u128.
/// (hi, lo) / d
pub fn div_u256_by_u128(hi: u128, lo: u128, d: u128) -> u128 {
    assert!(d != 0, "div by zero");
    if hi == 0 {
        return lo / d;
    }

    // For our use cases, the result must fit in u128.
    let q_hi = hi / d;
    let r_hi = hi % d;
    assert!(q_hi == 0, "u256 div overflow");

    let mut rem: u128 = r_hi;
    let mut result: u128 = 0;

    // Process each bit of lo from MSB to LSB
    for i in (0..128).rev() {
        // Shift remainder left by 1
        let overflow = rem >> 127;
        rem = rem << 1;
        if lo & (1u128 << i) != 0 {
            rem |= 1;
        }

        if overflow != 0 || rem >= d {
            rem = rem.wrapping_sub(d);
            result |= 1u128 << i;
        }
    }

    result
}

// ============================================================================
// Q64.64 arithmetic
// ============================================================================

/// Q64 × Q64 → Q64 (truncates / rounds down)
pub fn q64_mul(a: u128, b: u128) -> u128 {
    let (hi, lo) = mul_u128(a, b);
    // Result = (hi, lo) >> 64 = (hi << 64) | (lo >> 64)
    (hi << 64) | (lo >> 64)
}

/// Q64 ÷ Q64 → Q64 (truncates / rounds down)
pub fn q64_div(a: u128, b: u128) -> u128 {
    assert!(b != 0, "q64 div by zero");
    // a / b in Q64 = (a << 64) / b
    let (hi, lo) = (a >> 64, a << 64);
    div_u256_by_u128(hi, lo, b)
}

/// amount × q64_fraction / Q64, with rounding control
/// Used to convert Q64.64 fee amounts to token amounts
pub fn mul_q64(amount: u128, frac: u128, round_up: bool) -> u128 {
    let (hi, lo) = mul_u128(amount, frac);
    let result = div_u256_by_u128(hi >> 64, (hi << 64) | (lo >> 64), 1u128);
    if round_up {
        // Check if there's a fractional remainder
        let lo_frac = lo & ((1u128 << 64) - 1);
        if lo_frac > 0 {
            result + 1
        } else {
            result
        }
    } else {
        result
    }
}

// ============================================================================
// Tick ↔ sqrt_price conversions
// ============================================================================

/// Correctly computed sqrt(1.0001)^(2^k) for k=0..19 in Q64.64
/// Value = floor(sqrt(1.0001^(2^k)) * 2^64)
///
/// For tick_to_sqrt_price: result = product of table[k] for each set bit k in |tick|
/// If tick < 0, result = Q64^2 / result (reciprocal)
fn get_sqrt_ratio_at_bit(k: u32) -> u128 {
    match k {
        0  => 18447666387855959850,              // sqrt(1.0001^1)
        1  => 18448588748116922571,              // sqrt(1.0001^2)
        2  => 18450433606991734263,              // sqrt(1.0001^4)
        3  => 18454123878217468680,              // sqrt(1.0001^8)
        4  => 18461506635090006701,              // sqrt(1.0001^16)
        5  => 18476281010653910144,              // sqrt(1.0001^32)
        6  => 18505865242158250041,              // sqrt(1.0001^64)
        7  => 18565175891880433522,              // sqrt(1.0001^128)
        8  => 18684368066214940582,              // sqrt(1.0001^256)
        9  => 18925053041275764671,              // sqrt(1.0001^512)
        10 => 19415764168677886926,              // sqrt(1.0001^1024)
        11 => 20435687552633177494,              // sqrt(1.0001^2048)
        12 => 22639080592224303007,              // sqrt(1.0001^4096)
        13 => 27784196929998399742,              // sqrt(1.0001^8192)
        14 => 41848122137994986128,              // sqrt(1.0001^16384)
        15 => 94936283578220370716,              // sqrt(1.0001^32768)
        16 => 488590176327622479860,             // sqrt(1.0001^65536)
        17 => 12941056668319229769860,           // sqrt(1.0001^131072)
        18 => 9078618265828848800676189,         // sqrt(1.0001^262144)
        19 => 4468068147273140139091016147737,   // sqrt(1.0001^524288)
        _ => panic!("k out of range"),
    }
}

/// Convert tick index to sqrt price in Q64.64
/// P(tick) = 1.0001^tick, so sqrt_price = 1.0001^(tick/2) = sqrt(1.0001)^tick
/// Uses binary exponentiation with precomputed table
pub fn tick_to_sqrt_price(tick: i32) -> Result<u128> {
    require!(tick >= MIN_TICK && tick <= MAX_TICK, KclmmError::TickOutOfBounds);

    if tick == 0 {
        return Ok(Q64);
    }

    let abs_tick = tick.unsigned_abs();

    // Binary exponentiation: multiply table entries for each set bit
    let mut result = Q64; // Start with 1.0 in Q64.64

    for k in 0..20u32 {
        if abs_tick & (1 << k) != 0 {
            result = q64_mul(result, get_sqrt_ratio_at_bit(k));
        }
    }

    // For negative ticks, take reciprocal: Q64^2 / result
    if tick < 0 {
        result = q64_div(Q64, result);
    }

    Ok(result)
}

/// Convert sqrt price (Q64.64) to tick index
/// Find the largest tick i such that tick_to_sqrt_price(i) <= sqrt_price
pub fn sqrt_price_to_tick(sqrt_price: u128) -> Result<i32> {
    require!(sqrt_price >= MIN_SQRT_PRICE && sqrt_price <= MAX_SQRT_PRICE, KclmmError::InvalidSqrtPrice);

    // Determine sign: if sqrt_price < Q64 (i.e., < 1.0), tick is negative
    let negative = sqrt_price < Q64;

    // Work with the value >= 1.0
    let price = if negative {
        q64_div(Q64, sqrt_price)
    } else {
        sqrt_price
    };

    // Binary search: find which bits of the tick are set
    let mut tick: i32 = 0;
    let mut current = price;

    // Check from the highest bit down
    for k in (0..20u32).rev() {
        let ratio = get_sqrt_ratio_at_bit(k);
        if current >= ratio {
            current = q64_div(current, ratio);
            tick |= 1i32 << k;
        }
    }

    if negative {
        tick = -tick;
        // Verify: we want the largest tick where sqrt_price(tick) <= given sqrt_price
        // For negative ticks, we may need to adjust by -1
        let check = tick_to_sqrt_price(tick)?;
        if check > sqrt_price {
            tick -= 1;
        }
    } else {
        // For positive, verify we have the floor
        let check = tick_to_sqrt_price(tick)?;
        if check > sqrt_price {
            tick -= 1;
        }
    }

    Ok(tick)
}

// ============================================================================
// Swap step computation
// ============================================================================

pub struct SwapStepResult {
    pub sqrt_price_next: u128,
    pub amount_in: u64,
    pub amount_out: u64,
    pub fee_amount: u64,
}

/// Compute one step of a swap within a single tick range
/// Returns the new sqrt_price, amounts in/out, and fee
pub fn compute_swap_step(
    sqrt_price_current: u128,
    sqrt_price_target: u128,
    liquidity: u128,
    amount_remaining: u64,
    fee_rate: u32,
) -> Result<SwapStepResult> {
    let a_to_b = sqrt_price_current >= sqrt_price_target;

    // Compute max amount that can be swapped to reach target price
    let amount_remaining_less_fee = {
        let fee_complement = (FEE_RATE_DENOMINATOR - fee_rate) as u128;
        ((amount_remaining as u128) * fee_complement / FEE_RATE_DENOMINATOR as u128) as u64
    };

    let max_amount_in = if a_to_b {
        // a→b: input is token A, need delta_a to go from current to target
        get_amount_a_delta(sqrt_price_target, sqrt_price_current, liquidity, true)?
    } else {
        // b→a: input is token B
        get_amount_b_delta(sqrt_price_current, sqrt_price_target, liquidity, true)?
    };

    let (sqrt_price_next, amount_in, amount_out) = if amount_remaining_less_fee >= max_amount_in {
        // We can reach the target price
        let amount_out = if a_to_b {
            get_amount_b_delta(sqrt_price_target, sqrt_price_current, liquidity, false)?
        } else {
            get_amount_a_delta(sqrt_price_current, sqrt_price_target, liquidity, false)?
        };
        (sqrt_price_target, max_amount_in, amount_out)
    } else {
        // We run out of input before reaching target — compute new sqrt_price
        let sqrt_price_next = get_next_sqrt_price(
            sqrt_price_current,
            liquidity,
            amount_remaining_less_fee,
            a_to_b,
        )?;
        let amount_in = if a_to_b {
            get_amount_a_delta(sqrt_price_next, sqrt_price_current, liquidity, true)?
        } else {
            get_amount_b_delta(sqrt_price_current, sqrt_price_next, liquidity, true)?
        };
        let amount_out = if a_to_b {
            get_amount_b_delta(sqrt_price_next, sqrt_price_current, liquidity, false)?
        } else {
            get_amount_a_delta(sqrt_price_current, sqrt_price_next, liquidity, false)?
        };
        (sqrt_price_next, amount_in, amount_out)
    };

    // Fee is the difference between what was consumed and the net input
    let fee_amount = if amount_remaining_less_fee >= max_amount_in {
        // We reached the target, so fee is computed from the actual amount_in
        let amount_in_128 = amount_in as u128;
        let fee = amount_in_128 * fee_rate as u128 / (FEE_RATE_DENOMINATOR - fee_rate) as u128;
        // Round up
        let fee_rounded = if amount_in_128 * fee_rate as u128 % (FEE_RATE_DENOMINATOR - fee_rate) as u128 != 0 {
            fee + 1
        } else {
            fee
        };
        fee_rounded.min(u64::MAX as u128) as u64
    } else {
        // Used all input; fee = amount_remaining - amount_in
        amount_remaining.saturating_sub(amount_in)
    };

    Ok(SwapStepResult {
        sqrt_price_next,
        amount_in,
        amount_out,
        fee_amount,
    })
}

/// Given an input amount (after fee) and current state, compute the new sqrt_price
fn get_next_sqrt_price(
    sqrt_price: u128,
    liquidity: u128,
    amount: u64,
    a_to_b: bool,
) -> Result<u128> {
    if amount == 0 {
        return Ok(sqrt_price);
    }
    let amount = amount as u128;

    if a_to_b {
        // Adding token A (price decreases)
        // new_sqrt_P = L * sqrt_P / (L + amount * sqrt_P)
        // = L * sqrt_P / (L + amount * sqrt_P)
        // In Q64.64: numerator = L * sqrt_P (needs u256)
        //            denominator = L + amount * sqrt_P / Q64
        let (num_hi, num_lo) = mul_u128(liquidity, sqrt_price);

        // amount * sqrt_price / Q64 (amount is a raw number, sqrt_price is Q64.64)
        let (prod_hi, prod_lo) = mul_u128(amount, sqrt_price);
        let amount_times_price = (prod_hi << 64) | (prod_lo >> 64);

        let denom = liquidity.checked_add(amount_times_price)
            .ok_or(KclmmError::MathOverflow)?;

        let result = div_u256_by_u128(num_hi, num_lo, denom);
        Ok(result)
    } else {
        // Adding token B (price increases)
        // new_sqrt_P = sqrt_P + amount * Q64 / L
        // amount is raw tokens, need to convert to Q64.64 price delta
        let delta = ((amount as u128) << 64)
            .checked_div(liquidity)
            .ok_or(KclmmError::DivisionByZero)?;
        let result = sqrt_price.checked_add(delta)
            .ok_or(KclmmError::MathOverflow)?;
        Ok(result)
    }
}

// ============================================================================
// Liquidity ↔ token amount conversions
// ============================================================================

/// Amount of token A for a given liquidity and price range
/// delta_a = L * (1/sqrt_P_lower - 1/sqrt_P_upper)
///         = L * (sqrt_P_upper - sqrt_P_lower) / (sqrt_P_lower * sqrt_P_upper)
/// Requires sqrt_lower < sqrt_upper
fn get_amount_a_delta(sqrt_lower: u128, sqrt_upper: u128, liquidity: u128, round_up: bool) -> Result<u64> {
    if sqrt_lower == sqrt_upper || liquidity == 0 {
        return Ok(0);
    }

    // delta_a = L * (sqrt_upper - sqrt_lower) * Q64 / (sqrt_lower * sqrt_upper)
    // Rearranged as: (L * (sqrt_upper - sqrt_lower)) / ((sqrt_lower * sqrt_upper) >> 64)
    let price_diff = sqrt_upper - sqrt_lower;

    // numerator = L * price_diff (u256)
    let (num_hi, num_lo) = mul_u128(liquidity, price_diff);

    // denominator = sqrt_lower * sqrt_upper (u256) >> 64
    let (denom_hi, denom_lo) = mul_u128(sqrt_lower, sqrt_upper);
    let denom = (denom_hi << 64) | (denom_lo >> 64);

    if denom == 0 {
        return err!(KclmmError::DivisionByZero);
    }

    let result = div_u256_by_u128(num_hi, num_lo, denom);

    if round_up {
        // Check remainder: if result * denom != numerator, round up
        let (check_hi, check_lo) = mul_u128(result, denom);
        if check_hi != num_hi || check_lo != num_lo {
            Ok((result + 1).min(u64::MAX as u128) as u64)
        } else {
            Ok(result.min(u64::MAX as u128) as u64)
        }
    } else {
        Ok(result.min(u64::MAX as u128) as u64)
    }
}

/// Amount of token B for a given liquidity and price range
/// delta_b = L * (sqrt_P_upper - sqrt_P_lower) / Q64
/// Requires sqrt_lower < sqrt_upper
fn get_amount_b_delta(sqrt_lower: u128, sqrt_upper: u128, liquidity: u128, round_up: bool) -> Result<u64> {
    if sqrt_lower == sqrt_upper || liquidity == 0 {
        return Ok(0);
    }

    let price_diff = sqrt_upper - sqrt_lower;

    // L * price_diff / Q64
    let (hi, lo) = mul_u128(liquidity, price_diff);
    // >> 64 to divide by Q64
    let result = (hi << 64) | (lo >> 64);
    let remainder = lo & ((1u128 << 64) - 1);

    if round_up && remainder > 0 {
        Ok((result + 1).min(u64::MAX as u128) as u64)
    } else {
        Ok(result.min(u64::MAX as u128) as u64)
    }
}

/// Get token amounts needed/held for a given liquidity and position range
pub fn get_amounts_for_liquidity(
    sqrt_price: u128,
    sqrt_lower: u128,
    sqrt_upper: u128,
    liquidity: u128,
    round_up: bool,
) -> Result<(u64, u64)> {
    if sqrt_price <= sqrt_lower {
        // Below range: all token A
        let amount_a = get_amount_a_delta(sqrt_lower, sqrt_upper, liquidity, round_up)?;
        Ok((amount_a, 0))
    } else if sqrt_price < sqrt_upper {
        // In range: both tokens
        let amount_a = get_amount_a_delta(sqrt_price, sqrt_upper, liquidity, round_up)?;
        let amount_b = get_amount_b_delta(sqrt_lower, sqrt_price, liquidity, round_up)?;
        Ok((amount_a, amount_b))
    } else {
        // Above range: all token B
        let amount_b = get_amount_b_delta(sqrt_lower, sqrt_upper, liquidity, round_up)?;
        Ok((0, amount_b))
    }
}

/// Get the maximum liquidity that can be provided given token amounts and a price range
pub fn get_liquidity_for_amounts(
    sqrt_price: u128,
    sqrt_lower: u128,
    sqrt_upper: u128,
    amount_a: u64,
    amount_b: u64,
) -> Result<u128> {
    if sqrt_lower >= sqrt_upper {
        return err!(KclmmError::InvalidTickRange);
    }

    if sqrt_price <= sqrt_lower {
        // Below range: liquidity determined by token A
        // L = amount_a * sqrt_lower * sqrt_upper / (sqrt_upper - sqrt_lower) / Q64
        let l = liquidity_from_amount_a(sqrt_lower, sqrt_upper, amount_a)?;
        Ok(l)
    } else if sqrt_price < sqrt_upper {
        // In range: take the minimum of both
        let l_a = liquidity_from_amount_a(sqrt_price, sqrt_upper, amount_a)?;
        let l_b = liquidity_from_amount_b(sqrt_lower, sqrt_price, amount_b)?;
        Ok(l_a.min(l_b))
    } else {
        // Above range: liquidity determined by token B
        let l = liquidity_from_amount_b(sqrt_lower, sqrt_upper, amount_b)?;
        Ok(l)
    }
}

/// L from token A: L = amount_a * (sqrt_lower * sqrt_upper / Q64) / (sqrt_upper - sqrt_lower)
fn liquidity_from_amount_a(sqrt_lower: u128, sqrt_upper: u128, amount: u64) -> Result<u128> {
    let price_diff = sqrt_upper - sqrt_lower;
    if price_diff == 0 {
        return Ok(0);
    }

    // sqrt_lower * sqrt_upper (u256), >> 64 to Q64
    let (prod_hi, prod_lo) = mul_u128(sqrt_lower, sqrt_upper);
    let product = (prod_hi << 64) | (prod_lo >> 64);

    // amount * product (u256)
    let (num_hi, num_lo) = mul_u128(amount as u128, product);

    // / price_diff
    let result = div_u256_by_u128(num_hi, num_lo, price_diff);
    Ok(result)
}

/// L from token B: L = amount_b * Q64 / (sqrt_upper - sqrt_lower)
fn liquidity_from_amount_b(sqrt_lower: u128, sqrt_upper: u128, amount: u64) -> Result<u128> {
    let price_diff = sqrt_upper - sqrt_lower;
    if price_diff == 0 {
        return Ok(0);
    }

    // amount * Q64 / price_diff
    let numerator = (amount as u128) << 64;
    Ok(numerator / price_diff)
}

// ============================================================================
// Fee growth helpers
// ============================================================================

/// Compute fee_growth_inside for a position's tick range
/// Returns (fee_growth_inside_a, fee_growth_inside_b)
pub fn fee_growth_inside(
    lower_tick: &Tick,
    upper_tick: &Tick,
    tick_lower_index: i32,
    tick_upper_index: i32,
    tick_current: i32,
    fee_growth_global_a: u128,
    fee_growth_global_b: u128,
) -> (u128, u128) {
    // fee_growth_below(tick_lower)
    let (below_a, below_b) = if tick_current >= tick_lower_index {
        (lower_tick.fee_growth_outside_a, lower_tick.fee_growth_outside_b)
    } else {
        (
            fee_growth_global_a.wrapping_sub(lower_tick.fee_growth_outside_a),
            fee_growth_global_b.wrapping_sub(lower_tick.fee_growth_outside_b),
        )
    };

    // fee_growth_above(tick_upper)
    let (above_a, above_b) = if tick_current < tick_upper_index {
        (upper_tick.fee_growth_outside_a, upper_tick.fee_growth_outside_b)
    } else {
        (
            fee_growth_global_a.wrapping_sub(upper_tick.fee_growth_outside_a),
            fee_growth_global_b.wrapping_sub(upper_tick.fee_growth_outside_b),
        )
    };

    // inside = global - below - above
    let inside_a = fee_growth_global_a.wrapping_sub(below_a).wrapping_sub(above_a);
    let inside_b = fee_growth_global_b.wrapping_sub(below_b).wrapping_sub(above_b);

    (inside_a, inside_b)
}

/// Compute fees owed to a position
pub fn compute_fees_owed(
    fee_growth_inside_a: u128,
    fee_growth_inside_b: u128,
    fee_growth_inside_last_a: u128,
    fee_growth_inside_last_b: u128,
    liquidity: u128,
) -> (u64, u64) {
    let delta_a = fee_growth_inside_a.wrapping_sub(fee_growth_inside_last_a);
    let delta_b = fee_growth_inside_b.wrapping_sub(fee_growth_inside_last_b);

    // fees = L * delta / Q64 (round down)
    let fees_a = if delta_a > 0 && liquidity > 0 {
        let (hi, lo) = mul_u128(liquidity, delta_a);
        let result = (hi << 64) | (lo >> 64);
        result.min(u64::MAX as u128) as u64
    } else {
        0
    };

    let fees_b = if delta_b > 0 && liquidity > 0 {
        let (hi, lo) = mul_u128(liquidity, delta_b);
        let result = (hi << 64) | (lo >> 64);
        result.min(u64::MAX as u128) as u64
    } else {
        0
    };

    (fees_a, fees_b)
}

// ============================================================================
// Tick array utilities
// ============================================================================

/// Check if a tick is aligned to the given spacing
pub fn is_tick_aligned(tick: i32, spacing: u16) -> bool {
    let spacing = spacing as i32;
    if spacing == 0 {
        return false;
    }
    tick % spacing == 0
}

/// Get the start_tick_index of the tick array containing a given tick
pub fn tick_array_start_for_tick(tick: i32, spacing: u16) -> i32 {
    let ticks_in_array = spacing as i32 * TICKS_PER_ARRAY as i32;
    let mut start = tick / ticks_in_array * ticks_in_array;
    if tick < 0 && tick % ticks_in_array != 0 {
        start -= ticks_in_array;
    }
    start
}

/// Get the index within a tick array for a given tick
pub fn tick_index_in_array(tick: i32, array_start: i32, spacing: u16) -> Option<usize> {
    let spacing = spacing as i32;
    if spacing == 0 {
        return None;
    }
    let offset = tick - array_start;
    if offset < 0 {
        return None;
    }
    let idx = offset / spacing;
    if idx < 0 || idx >= TICKS_PER_ARRAY as i32 {
        return None;
    }
    if offset % spacing != 0 {
        return None;
    }
    Some(idx as usize)
}

// ============================================================================
// Bitmap helpers
// ============================================================================

pub fn set_bit(bitmap: &mut u64, idx: usize) {
    *bitmap |= 1u64 << idx;
}

pub fn clear_bit(bitmap: &mut u64, idx: usize) {
    *bitmap &= !(1u64 << idx);
}

pub fn is_set(bitmap: u64, idx: usize) -> bool {
    bitmap & (1u64 << idx) != 0
}

/// Find next set bit at or after `start`, going rightward (increasing index)
/// Returns None if no set bit found
pub fn next_set_bit(bitmap: u64, start: usize) -> Option<usize> {
    if start >= 64 {
        return None;
    }
    // Mask out bits below start
    let masked = bitmap & (!0u64 << start);
    if masked == 0 {
        None
    } else {
        Some(masked.trailing_zeros() as usize)
    }
}

/// Find previous set bit at or before `start`, going leftward (decreasing index)
/// Returns None if no set bit found
pub fn prev_set_bit(bitmap: u64, start: usize) -> Option<usize> {
    if start >= 63 {
        // Check all bits (start >= 63 means we want all bits up to and including bit 63)
        if bitmap == 0 {
            return None;
        }
        return Some(63 - bitmap.leading_zeros() as usize);
    }
    // Mask out bits above start
    let masked = bitmap & ((1u64 << (start + 1)) - 1);
    if masked == 0 {
        None
    } else {
        Some(63 - masked.leading_zeros() as usize)
    }
}

/// Find the next initialized tick in a tick array
/// For a_to_b (price decreasing): search leftward from current position
/// For b_to_a (price increasing): search rightward from current position
pub fn next_initialized_tick_in_array(
    array: &TickArray,
    current_tick: i32,
    spacing: u16,
    a_to_b: bool,
) -> Option<(i32, usize)> {
    let spacing_i32 = spacing as i32;

    if a_to_b {
        // Searching leftward (lower ticks)
        // Find the index at or below current_tick
        let offset = current_tick - array.start_tick_index;
        if offset < 0 {
            return None;
        }
        let idx = offset / spacing_i32;
        let search_start = (idx as usize).min(TICKS_PER_ARRAY - 1);
        prev_set_bit(array.initialized_bitmap, search_start)
            .map(|i| (array.start_tick_index + i as i32 * spacing_i32, i))
    } else {
        // Searching rightward (higher ticks)
        let offset = current_tick - array.start_tick_index;
        let idx = if offset < 0 {
            0
        } else {
            // Start searching from the tick ABOVE current
            (offset / spacing_i32 + 1) as usize
        };
        if idx >= TICKS_PER_ARRAY {
            return None;
        }
        next_set_bit(array.initialized_bitmap, idx)
            .map(|i| (array.start_tick_index + i as i32 * spacing_i32, i))
    }
}

// ============================================================================
// Tests (unit tests for math correctness)
// ============================================================================

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_mul_u128_simple() {
        let (hi, lo) = mul_u128(1, 1);
        assert_eq!(hi, 0);
        assert_eq!(lo, 1);

        let (hi, lo) = mul_u128(Q64, Q64);
        // Q64 * Q64 = 2^128
        assert_eq!(hi, 1);
        assert_eq!(lo, 0);
    }

    #[test]
    fn test_q64_mul() {
        // 1.0 * 1.0 = 1.0
        assert_eq!(q64_mul(Q64, Q64), Q64);
        // 2.0 * 3.0 = 6.0
        assert_eq!(q64_mul(2 * Q64, 3 * Q64), 6 * Q64);
    }

    #[test]
    fn test_q64_div() {
        // 1.0 / 1.0 = 1.0
        assert_eq!(q64_div(Q64, Q64), Q64);
        // 6.0 / 2.0 = 3.0
        assert_eq!(q64_div(6 * Q64, 2 * Q64), 3 * Q64);
    }

    #[test]
    fn test_tick_zero_gives_q64() {
        let price = tick_to_sqrt_price(0).unwrap();
        assert_eq!(price, Q64);
    }

    #[test]
    fn test_tick_positive_gives_above_q64() {
        let price = tick_to_sqrt_price(1).unwrap();
        assert!(price > Q64);
    }

    #[test]
    fn test_tick_negative_gives_below_q64() {
        let price = tick_to_sqrt_price(-1).unwrap();
        assert!(price < Q64);
    }

    #[test]
    fn test_tick_roundtrip() {
        for tick in [-100000, -10000, -1000, -100, -1, 0, 1, 100, 1000, 10000, 100000] {
            let price = tick_to_sqrt_price(tick).unwrap();
            let recovered = sqrt_price_to_tick(price).unwrap();
            assert!((recovered - tick).abs() <= 1, "tick {} roundtrip gave {}", tick, recovered);
        }
    }

    #[test]
    fn test_bitmap_ops() {
        let mut bitmap: u64 = 0;
        set_bit(&mut bitmap, 5);
        assert!(is_set(bitmap, 5));
        assert!(!is_set(bitmap, 4));

        set_bit(&mut bitmap, 10);
        assert_eq!(next_set_bit(bitmap, 0), Some(5));
        assert_eq!(next_set_bit(bitmap, 6), Some(10));
        assert_eq!(prev_set_bit(bitmap, 10), Some(10));
        assert_eq!(prev_set_bit(bitmap, 9), Some(5));

        clear_bit(&mut bitmap, 5);
        assert!(!is_set(bitmap, 5));
    }

    #[test]
    fn test_tick_array_start() {
        // With spacing 60, ticks_in_array = 60 * 64 = 3840
        assert_eq!(tick_array_start_for_tick(0, 60), 0);
        assert_eq!(tick_array_start_for_tick(3839, 60), 0);
        assert_eq!(tick_array_start_for_tick(3840, 60), 3840);
        assert_eq!(tick_array_start_for_tick(-1, 60), -3840);
        assert_eq!(tick_array_start_for_tick(-3840, 60), -3840);
        assert_eq!(tick_array_start_for_tick(-3841, 60), -7680);
    }
}
