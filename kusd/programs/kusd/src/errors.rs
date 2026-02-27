use anchor_lang::prelude::*;

#[error_code]
pub enum KusdError {
    #[msg("Math overflow")]
    MathOverflow,
    #[msg("Math underflow")]
    MathUnderflow,
    #[msg("Division by zero")]
    DivisionByZero,
    #[msg("Invalid config: max LTV must be less than liquidation threshold")]
    InvalidConfigLtv,
    #[msg("Invalid config: liquidation threshold exceeds 100%")]
    InvalidConfigLiqThreshold,
    #[msg("Invalid config: liquidation bonus exceeds 100%")]
    InvalidConfigBonus,
    #[msg("Invalid config: stability fee exceeds 100%")]
    InvalidConfigFee,
    #[msg("Oracle mint does not match collateral mint")]
    InvalidOracle,
    #[msg("Oracle price is stale")]
    OracleStale,
    #[msg("Deposit amount is zero")]
    ZeroDeposit,
    #[msg("Mint amount is zero")]
    ZeroMint,
    #[msg("Repay amount is zero")]
    ZeroRepay,
    #[msg("Withdraw amount is zero")]
    ZeroWithdraw,
    #[msg("Liquidation amount is zero")]
    ZeroLiquidation,
    #[msg("Vault is halted")]
    VaultHalted,
    #[msg("Exceeds max LTV")]
    ExceedsMaxLtv,
    #[msg("Debt ceiling exceeded")]
    DebtCeilingExceeded,
    #[msg("Insufficient collateral for withdrawal")]
    InsufficientCollateral,
    #[msg("Position is healthy, cannot liquidate")]
    PositionHealthy,
    #[msg("Liquidation amount exceeds close factor")]
    CloseFactorExceeded,
}
