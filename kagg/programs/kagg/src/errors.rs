use anchor_lang::prelude::*;

#[error_code]
pub enum KaggError {
    #[msg("Slippage exceeded: output below minimum")]
    SlippageExceeded,

    #[msg("Unknown DEX ID")]
    UnknownDexId,

    #[msg("Insufficient remaining accounts for step")]
    InsufficientAccounts,

    #[msg("Math overflow")]
    MathOverflow,

    #[msg("Zero swap amount")]
    ZeroSwapAmount,

    #[msg("Empty route plan")]
    EmptyRoutePlan,

    #[msg("Invalid token ledger index")]
    InvalidTokenLedgerIndex,
}
