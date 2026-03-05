use kagg::types::DexId;
use kclmm::constants::*;
use kclmm::math;
use solana_sdk::pubkey::Pubkey;
use super::QuotablePool;

/// Off-chain tick data (no Anchor overhead).
#[derive(Debug, Clone, Default)]
pub struct TickData {
    pub liquidity_net: i128,
    pub liquidity_gross: u128,
}

/// Off-chain tick array data.
#[derive(Debug, Clone)]
pub struct TickArrayData {
    pub address: Pubkey,
    pub start_tick_index: i32,
    pub initialized_bitmap: u64,
    pub ticks: Vec<TickData>, // exactly TICKS_PER_ARRAY (64) entries
}

/// Off-chain representation of a concentrated liquidity pool (kclmm).
#[derive(Debug, Clone)]
pub struct ClmmPool {
    pub address: Pubkey,
    pub mint_a: Pubkey,
    pub mint_b: Pubkey,
    pub vault_a: Pubkey,
    pub vault_b: Pubkey,
    pub authority: Pubkey,
    pub sqrt_price: u128,
    pub tick_current: i32,
    pub liquidity: u128,
    pub fee_rate: u32,
    pub tick_spacing: u16,
    pub tick_arrays: Vec<TickArrayData>,
    pub program_id: Pubkey,
}

impl ClmmPool {
    /// Find the next initialized tick in the given direction.
    /// Returns (tick_index, array_index_in_self, tick_index_in_array).
    fn find_next_tick(
        &self,
        current_tick: i32,
        a_to_b: bool,
        start_array_idx: &mut usize,
    ) -> Option<(i32, usize, usize)> {
        while *start_array_idx < self.tick_arrays.len() {
            let ta = &self.tick_arrays[*start_array_idx];
            let spacing = self.tick_spacing as i32;

            // Determine the search start position within this array
            let search_tick = if a_to_b { current_tick } else { current_tick + 1 };

            // Check if search_tick falls within this tick array's range
            let array_end = ta.start_tick_index + (TICKS_PER_ARRAY as i32) * spacing;
            if a_to_b {
                if search_tick < ta.start_tick_index {
                    *start_array_idx += 1;
                    continue;
                }
            } else {
                if search_tick >= array_end {
                    *start_array_idx += 1;
                    continue;
                }
            }

            // Compute the index within the array
            let offset = search_tick - ta.start_tick_index;
            let idx_in_array = if spacing > 0 { offset / spacing } else { 0 };

            if a_to_b {
                // Search backwards (prev_set_bit)
                let start = idx_in_array.min(TICKS_PER_ARRAY as i32 - 1).max(0) as usize;
                if let Some(bit_idx) = math::prev_set_bit(ta.initialized_bitmap, start) {
                    let tick = ta.start_tick_index + (bit_idx as i32) * spacing;
                    return Some((tick, *start_array_idx, bit_idx));
                }
            } else {
                // Search forwards (next_set_bit)
                let start = idx_in_array.max(0).min(TICKS_PER_ARRAY as i32) as usize;
                if start < TICKS_PER_ARRAY {
                    if let Some(bit_idx) = math::next_set_bit(ta.initialized_bitmap, start) {
                        let tick = ta.start_tick_index + (bit_idx as i32) * spacing;
                        return Some((tick, *start_array_idx, bit_idx));
                    }
                }
            }

            *start_array_idx += 1;
        }
        None
    }

    /// Determine which tick arrays are relevant for a swap.
    fn relevant_tick_array_addresses(&self, a_to_b: bool) -> Vec<Pubkey> {
        // For quoting we use all provided tick arrays.
        // For on-chain account ordering, we return addresses in traversal order.
        if a_to_b {
            // a_to_b: price decreases, traverse tick arrays in order (current → lower)
            self.tick_arrays.iter().map(|ta| ta.address).collect()
        } else {
            self.tick_arrays.iter().map(|ta| ta.address).collect()
        }
    }
}

impl QuotablePool for ClmmPool {
    fn quote(&self, amount_in: u64, a_to_b: bool) -> Option<u64> {
        if amount_in == 0 {
            return None;
        }

        let sqrt_price_limit = if a_to_b {
            MIN_SQRT_PRICE
        } else {
            MAX_SQRT_PRICE
        };

        let mut sqrt_price = self.sqrt_price;
        let mut tick_current = self.tick_current;
        let mut liquidity = self.liquidity;
        let mut amount_remaining = amount_in;
        let mut amount_out_total: u64 = 0;
        let mut tick_crossings: usize = 0;
        let mut array_idx: usize = 0;

        while amount_remaining > 0 && sqrt_price != sqrt_price_limit {
            // Find next initialized tick
            let mut search_idx = array_idx;
            let search_result = self.find_next_tick(tick_current, a_to_b, &mut search_idx);

            let (next_tick, ta_idx, tick_idx_in_array) = match search_result {
                Some(r) => r,
                None => break,
            };
            // Update array_idx for next iteration
            array_idx = ta_idx;

            let next_sqrt_price = math::tick_to_sqrt_price(next_tick).ok()?;
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
                    if tick_crossings > MAX_TICK_CROSSINGS {
                        break;
                    }
                    // Cross tick: apply liquidity_net
                    let ta = &self.tick_arrays[ta_idx];
                    let net = ta.ticks[tick_idx_in_array].liquidity_net;
                    if a_to_b {
                        liquidity = liquidity.wrapping_sub(net as u128);
                    } else {
                        liquidity = liquidity.wrapping_add(net as u128);
                    }
                }
                continue;
            }

            // Compute swap step
            let step = math::compute_swap_step(
                sqrt_price,
                step_target,
                liquidity,
                amount_remaining,
                self.fee_rate,
            )
            .ok()?;

            sqrt_price = step.sqrt_price_next;
            amount_remaining = amount_remaining
                .checked_sub(step.amount_in)?
                .checked_sub(step.fee_amount)?;
            amount_out_total = amount_out_total.checked_add(step.amount_out)?;

            // Tick crossing
            if sqrt_price == next_sqrt_price {
                tick_current = if a_to_b { next_tick - 1 } else { next_tick };
                tick_crossings += 1;
                if tick_crossings > MAX_TICK_CROSSINGS {
                    break;
                }
                let ta = &self.tick_arrays[ta_idx];
                let net = ta.ticks[tick_idx_in_array].liquidity_net;
                if a_to_b {
                    liquidity = liquidity.wrapping_sub(net as u128);
                } else {
                    liquidity = liquidity.wrapping_add(net as u128);
                }
            } else {
                tick_current = math::sqrt_price_to_tick(sqrt_price).ok()?;
            }
        }

        if amount_out_total == 0 { None } else { Some(amount_out_total) }
    }

    fn mint_a(&self) -> Pubkey { self.mint_a }
    fn mint_b(&self) -> Pubkey { self.mint_b }
    fn address(&self) -> Pubkey { self.address }
    fn dex_id(&self) -> DexId { DexId::Kclmm }

    fn swap_accounts(&self, a_to_b: bool) -> Vec<(Pubkey, bool, bool)> {
        let input_mint = if a_to_b { self.mint_a } else { self.mint_b };
        let mut accounts = vec![
            (self.program_id, false, false), // kclmm program
            (self.address, false, true),      // pool (mut)
            (self.authority, false, false),    // pool authority
            (self.vault_a, false, true),       // vault_a (mut)
            (self.vault_b, false, true),       // vault_b (mut)
            (input_mint, false, false),        // input mint
        ];
        // Append relevant tick array accounts
        for addr in self.relevant_tick_array_addresses(a_to_b) {
            accounts.push((addr, false, true)); // tick arrays are mutable
        }
        accounts
    }

    fn extra_data(&self, a_to_b: bool, _amount_in: u64) -> Vec<u8> {
        let limit = if a_to_b { MIN_SQRT_PRICE } else { MAX_SQRT_PRICE };
        limit.to_le_bytes().to_vec()
    }

    fn num_accounts(&self, a_to_b: bool) -> u8 {
        (6 + self.relevant_tick_array_addresses(a_to_b).len()) as u8
    }
}
