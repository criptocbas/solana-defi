/// PDA seeds
pub const CDP_VAULT_SEED: &[u8] = b"cdp_vault";
pub const CDP_VAULT_AUTHORITY_SEED: &[u8] = b"cdp_vault_authority";
pub const KUSD_MINT_SEED: &[u8] = b"kusd_mint";
pub const CDP_POSITION_SEED: &[u8] = b"cdp_position";
pub const MOCK_ORACLE_SEED: &[u8] = b"mock_oracle";

/// Scaling
pub const SCALE: u128 = 1_000_000_000_000_000_000; // 1e18
pub const BPS_SCALE: u64 = 10_000;
pub const PRICE_SCALE: u64 = 1_000_000; // oracle prices in USD * 1e6
pub const SECONDS_PER_YEAR: u128 = 31_536_000; // 365 * 24 * 3600

/// Liquidation
pub const CLOSE_FACTOR_BPS: u64 = 5_000; // 50% max per liquidation

/// Authority funding (0.01 SOL for rent)
pub const AUTHORITY_FUND_LAMPORTS: u64 = 10_000_000;

/// kUSD stablecoin decimals
pub const KUSD_DECIMALS: u8 = 6;
