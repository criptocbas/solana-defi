/// PDA seeds
pub const LEVERAGED_VAULT_SEED: &[u8] = b"leveraged_vault";
pub const LEV_VAULT_AUTHORITY_SEED: &[u8] = b"lev_vault_authority";
pub const LEV_SHARE_MINT_SEED: &[u8] = b"lev_share_mint";

/// Scaling
pub const BPS_SCALE: u64 = 10_000;
pub const SECONDS_PER_YEAR: u128 = 365 * 24 * 3600; // 31_536_000

/// Virtual shares/assets for inflation attack defense (ERC-4626)
pub const VIRTUAL_SHARES: u128 = 1;
pub const VIRTUAL_ASSETS: u128 = 1;

/// Vault authority funding amount (0.1 SOL for klend obligation rent)
pub const AUTHORITY_FUND_LAMPORTS: u64 = 100_000_000;

/// klend SCALE (1e18)
pub const KLEND_SCALE: u128 = 1_000_000_000_000_000_000;

/// Oracle price scale (1e6)
pub const ORACLE_PRICE_SCALE: u128 = 1_000_000;
