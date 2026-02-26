/// PDA seeds
pub const LENDING_MARKET_SEED: &[u8] = b"lending_market";
pub const RESERVE_SEED: &[u8] = b"reserve";
pub const RESERVE_AUTHORITY_SEED: &[u8] = b"reserve_authority";
pub const OBLIGATION_SEED: &[u8] = b"obligation";
pub const MOCK_ORACLE_SEED: &[u8] = b"mock_oracle";

/// Scaling
pub const SCALE: u128 = 1_000_000_000_000_000_000; // 1e18
pub const BPS_SCALE: u64 = 10_000;
pub const PRICE_SCALE: u64 = 1_000_000; // oracle prices in USD * 1e6
pub const SECONDS_PER_YEAR: u128 = 365 * 24 * 3600; // 31_536_000

/// Virtual shares/assets for inflation attack defense
pub const VIRTUAL_SHARES: u128 = 1;
pub const VIRTUAL_ASSETS: u128 = 1;

/// Liquidation
pub const CLOSE_FACTOR_BPS: u64 = 5_000; // 50%

/// Caps (max entries per obligation)
pub const MAX_DEPOSITS: usize = 5;
pub const MAX_BORROWS: usize = 5;
