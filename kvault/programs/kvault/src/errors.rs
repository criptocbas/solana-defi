use anchor_lang::prelude::*;

#[error_code]
pub enum KvaultError {
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
    #[msg("Allocate amount must be greater than zero")]
    ZeroAllocate,
    #[msg("Deallocate shares must be greater than zero")]
    ZeroDeallocate,
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
    #[msg("Unauthorized: admin only")]
    Unauthorized,
    #[msg("No yield to harvest")]
    NoYield,
    #[msg("Invalid klend program")]
    InvalidKlendProgram,
}
