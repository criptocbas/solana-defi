use anchor_lang::prelude::*;

#[error_code]
pub enum CpammError {
    #[msg("Token mints must be in canonical order (mint_a < mint_b by pubkey)")]
    InvalidTokenOrder,
    #[msg("Deposit amounts must be greater than zero")]
    ZeroDepositAmount,
    #[msg("Swap amount must be greater than zero")]
    ZeroSwapAmount,
    #[msg("Burn amount must be greater than zero")]
    ZeroBurnAmount,
    #[msg("Initial liquidity too low after locking minimum")]
    InsufficientInitialLiquidity,
    #[msg("LP tokens minted below minimum requested")]
    SlippageExceededMint,
    #[msg("Output amount below minimum requested")]
    SlippageExceededOutput,
    #[msg("Amount A received below minimum requested")]
    SlippageExceededAmountA,
    #[msg("Amount B received below minimum requested")]
    SlippageExceededAmountB,
    #[msg("Swap produced zero output")]
    ZeroOutputAmount,
    #[msg("Math overflow occurred")]
    MathOverflow,
    #[msg("Input mint does not match either pool token")]
    InvalidInputMint,
    #[msg("Pool has no liquidity")]
    EmptyPool,
}
