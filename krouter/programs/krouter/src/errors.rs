use anchor_lang::prelude::*;

#[error_code]
pub enum KrouterError {
    #[msg("Slippage exceeded: output below minimum")]
    SlippageExceeded,

    #[msg("Invalid pool type")]
    InvalidPoolType,

    #[msg("Split amounts do not sum to total")]
    SplitAmountMismatch,

    #[msg("Insufficient accounts for leg")]
    InsufficientAccounts,

    #[msg("Math overflow")]
    MathOverflow,

    #[msg("Zero swap amount")]
    ZeroSwapAmount,
}
