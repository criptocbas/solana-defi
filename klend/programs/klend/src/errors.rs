use anchor_lang::prelude::*;

#[error_code]
pub enum KlendError {
    #[msg("Math overflow")]
    MathOverflow,
    #[msg("Math underflow")]
    MathUnderflow,
    #[msg("Division by zero")]
    DivisionByZero,
    #[msg("Invalid config: LTV must be less than liquidation threshold")]
    InvalidConfigLtv,
    #[msg("Invalid config: liquidation threshold exceeds 100%")]
    InvalidConfigLiqThreshold,
    #[msg("Invalid config: reserve factor exceeds 100%")]
    InvalidConfigReserveFactor,
    #[msg("Invalid config: optimal utilization exceeds 100%")]
    InvalidConfigUtilization,
    #[msg("Deposit amount is zero")]
    ZeroDeposit,
    #[msg("Withdraw shares is zero")]
    ZeroWithdraw,
    #[msg("Borrow amount is zero")]
    ZeroBorrow,
    #[msg("Repay amount is zero")]
    ZeroRepay,
    #[msg("Liquidation amount is zero")]
    ZeroLiquidation,
    #[msg("Supply cap exceeded")]
    SupplyCapExceeded,
    #[msg("Borrow cap exceeded")]
    BorrowCapExceeded,
    #[msg("Health factor too low for this operation")]
    HealthFactorTooLow,
    #[msg("Position is healthy, cannot liquidate")]
    PositionHealthy,
    #[msg("Liquidation amount exceeds close factor")]
    CloseFactorExceeded,
    #[msg("Oracle price is stale")]
    OracleStale,
    #[msg("No collateral deposited for this reserve")]
    NoCollateralDeposit,
    #[msg("No borrow found for this reserve")]
    NoBorrowFound,
    #[msg("Maximum obligation entries reached")]
    MaxEntriesReached,
    #[msg("Reserve must be refreshed before this operation")]
    ReserveStale,
    #[msg("Insufficient vault liquidity")]
    InsufficientLiquidity,
}
