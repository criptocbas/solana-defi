use anchor_lang::prelude::*;
use crate::constants::TICKS_PER_ARRAY;

#[account]
pub struct Pool {
    pub mint_a: Pubkey,               // 32
    pub mint_b: Pubkey,               // 32
    pub vault_a: Pubkey,              // 32
    pub vault_b: Pubkey,              // 32
    pub pool_authority: Pubkey,       // 32
    pub fee_rate: u32,                // 4   (e.g. 3000 = 0.30%)
    pub tick_spacing: u16,            // 2
    pub protocol_fee_rate: u16,       // 2   (BPS of swap fee)
    pub sqrt_price: u128,             // 16  (Q64.64)
    pub tick_current: i32,            // 4
    pub liquidity: u128,              // 16  (active L)
    pub fee_growth_global_a: u128,    // 16  (Q64.64)
    pub fee_growth_global_b: u128,    // 16  (Q64.64)
    pub protocol_fees_a: u64,         // 8
    pub protocol_fees_b: u64,         // 8
    pub pool_bump: u8,                // 1
    pub authority_bump: u8,           // 1
    pub _padding: [u8; 6],           // 6
}

impl Pool {
    // 32*5 + 4 + 2 + 2 + 16 + 4 + 16 + 16 + 16 + 8 + 8 + 1 + 1 + 6 = 260
    pub const SPACE: usize = 8 + 260;
}

/// Zero-copy tick array — too large (4KB+) for regular stack-based Account deserialization.
/// Use AccountLoader<'info, TickArray> in instruction accounts.
#[account(zero_copy(unsafe))]
#[repr(C)]
pub struct TickArray {
    pub pool: Pubkey,                 // 32
    pub start_tick_index: i32,        // 4
    pub initialized_bitmap: u64,      // 8
    pub ticks: [Tick; TICKS_PER_ARRAY], // 64 * 64 = 4096
}

impl TickArray {
    // repr(C) layout: pool(32) + start_tick_index(4) + padding(4) + initialized_bitmap(8) + ticks(4096) = 4144
    // plus 8-byte discriminator
    pub const SPACE: usize = 8 + 4144;
}

#[derive(AnchorSerialize, AnchorDeserialize, Clone, Copy, Default)]
#[repr(C)]
pub struct Tick {
    pub liquidity_net: i128,            // 16
    pub liquidity_gross: u128,          // 16
    pub fee_growth_outside_a: u128,     // 16 (Q64.64)
    pub fee_growth_outside_b: u128,     // 16 (Q64.64)
}

#[account]
pub struct Position {
    pub pool: Pubkey,                   // 32
    pub owner: Pubkey,                  // 32
    pub tick_lower: i32,                // 4
    pub tick_upper: i32,                // 4
    pub liquidity: u128,                // 16
    pub fee_growth_inside_last_a: u128, // 16 (Q64.64)
    pub fee_growth_inside_last_b: u128, // 16 (Q64.64)
    pub tokens_owed_a: u64,            // 8
    pub tokens_owed_b: u64,            // 8
    pub bump: u8,                       // 1
}

impl Position {
    // 32 + 32 + 4 + 4 + 16 + 16 + 16 + 8 + 8 + 1 = 137
    pub const SPACE: usize = 8 + 137;
}
