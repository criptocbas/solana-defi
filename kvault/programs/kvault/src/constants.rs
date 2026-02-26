/// PDA seeds
pub const VAULT_SEED: &[u8] = b"vault";
pub const VAULT_AUTHORITY_SEED: &[u8] = b"vault_authority";
pub const SHARE_MINT_SEED: &[u8] = b"share_mint";

/// Scaling
pub const BPS_SCALE: u64 = 10_000;
pub const SECONDS_PER_YEAR: u128 = 365 * 24 * 3600; // 31_536_000

/// Virtual shares/assets for inflation attack defense (ERC-4626)
pub const VIRTUAL_SHARES: u128 = 1;
pub const VIRTUAL_ASSETS: u128 = 1;

/// Vault authority funding amount (0.1 SOL for klend obligation rent)
pub const AUTHORITY_FUND_LAMPORTS: u64 = 100_000_000;
