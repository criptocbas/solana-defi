// PDA seeds
pub const POOL_SEED: &[u8] = b"pool";
pub const POOL_AUTHORITY_SEED: &[u8] = b"pool_authority";
pub const TICK_ARRAY_SEED: &[u8] = b"tick_array";
pub const POSITION_SEED: &[u8] = b"position";

// Q64.64 fixed-point: 1.0 = 2^64
pub const Q64: u128 = 1u128 << 64;

// Tick bounds (same as Uniswap v3)
pub const MIN_TICK: i32 = -443636;
pub const MAX_TICK: i32 = 443636;

// sqrt_price bounds in Q64.64
// MIN_SQRT_PRICE = sqrt(1.0001^MIN_TICK) * 2^64  ≈  very small
// MAX_SQRT_PRICE = sqrt(1.0001^MAX_TICK) * 2^64  ≈  very large
// We use the same approach as Uniswap v3: computed from tick bounds
pub const MIN_SQRT_PRICE: u128 = 4295048016; // tick_to_sqrt_price(MIN_TICK) in Q64.64
pub const MAX_SQRT_PRICE: u128 = 79226673515401279963822778343; // tick_to_sqrt_price(MAX_TICK) in Q64.64

// Fee tiers in hundredths of a basis point (1 = 0.01bps, 100 = 1bps)
// fee_rate is stored as parts per million: 100 = 0.01%, 500 = 0.05%, 3000 = 0.30%, 10000 = 1.00%
pub const FEE_RATE_1: u32 = 100; // 0.01% — tick spacing 1
pub const FEE_RATE_5: u32 = 500; // 0.05% — tick spacing 10
pub const FEE_RATE_30: u32 = 3000; // 0.30% — tick spacing 60
pub const FEE_RATE_100: u32 = 10000; // 1.00% — tick spacing 200
pub const FEE_RATE_DENOMINATOR: u32 = 1_000_000;

// Default protocol fee: 10% of swap fees (1000 basis points)
pub const DEFAULT_PROTOCOL_FEE_RATE: u16 = 1000;
pub const PROTOCOL_FEE_DENOMINATOR: u16 = 10000;

// Ticks per tick array
pub const TICKS_PER_ARRAY: usize = 64;

// Max tick crossings per swap (CU budget safety)
pub const MAX_TICK_CROSSINGS: usize = 20;

// Max tick arrays passed via remaining_accounts
pub const MAX_TICK_ARRAY_ACCOUNTS: usize = 3;

/// Returns tick spacing for a fee rate
pub fn fee_rate_to_tick_spacing(fee_rate: u32) -> Option<u16> {
    match fee_rate {
        FEE_RATE_1 => Some(1),
        FEE_RATE_5 => Some(10),
        FEE_RATE_30 => Some(60),
        FEE_RATE_100 => Some(200),
        _ => None,
    }
}
