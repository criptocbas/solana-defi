use anchor_lang::prelude::*;

#[error_code]
pub enum KlevError {
    #[msg("Math overflow")]
    MathOverflow,
    #[msg("Math underflow")]
    MathUnderflow,
    #[msg("Division by zero")]
    DivisionByZero,
    #[msg("Deposit amount must be greater than zero")]
    ZeroDeposit,
    #[msg("Withdraw shares must be greater than zero")]
    ZeroWithdraw,
    #[msg("Vault is halted")]
    VaultHalted,
    #[msg("Deposit cap exceeded")]
    DepositCapExceeded,
    #[msg("Insufficient idle balance for withdrawal")]
    InsufficientIdle,
    #[msg("Performance fee exceeds maximum (10000 bps)")]
    PerformanceFeeExceeded,
    #[msg("Management fee exceeds maximum (10000 bps)")]
    ManagementFeeExceeded,
    #[msg("Leverage ratio exceeds maximum")]
    MaxLeverageExceeded,
    #[msg("Health factor too low after operation")]
    HealthFactorTooLow,
    #[msg("Collateral amount must be greater than zero")]
    ZeroCollateral,
    #[msg("Borrow amount must be greater than zero")]
    ZeroBorrow,
    #[msg("Swap output below minimum")]
    SlippageExceeded,
    #[msg("Swap amount must be greater than zero")]
    ZeroSwapAmount,
    #[msg("Repay amount must be greater than zero")]
    ZeroRepay,
    #[msg("Withdraw shares must be greater than zero for deleverage")]
    ZeroWithdrawShares,
    #[msg("No collateral deposit found in obligation")]
    NoCollateralDeposit,
    #[msg("No borrow found in obligation")]
    NoBorrowFound,
}
