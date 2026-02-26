use anchor_lang::prelude::*;

#[error_code]
pub enum KclmmError {
    #[msg("Invalid fee tier")]
    InvalidFeeTier,
    #[msg("Token mints must be ordered: mint_a < mint_b")]
    InvalidTokenOrder,
    #[msg("Initial sqrt price out of bounds")]
    InvalidSqrtPrice,
    #[msg("Tick out of bounds")]
    TickOutOfBounds,
    #[msg("Tick not aligned to tick spacing")]
    TickNotAligned,
    #[msg("Tick lower must be less than tick upper")]
    InvalidTickRange,
    #[msg("Invalid tick array start index")]
    InvalidTickArrayStartIndex,
    #[msg("Tick array mismatch: does not belong to this pool")]
    TickArrayPoolMismatch,
    #[msg("Tick not in array range")]
    TickNotInArray,
    #[msg("Liquidity delta must be greater than zero")]
    ZeroLiquidityDelta,
    #[msg("Amount A exceeds maximum")]
    AmountAExceedsMax,
    #[msg("Amount B exceeds maximum")]
    AmountBExceedsMax,
    #[msg("Amount A below minimum")]
    AmountABelowMin,
    #[msg("Amount B below minimum")]
    AmountBBelowMin,
    #[msg("Swap input amount must be greater than zero")]
    ZeroSwapInput,
    #[msg("Invalid sqrt price limit")]
    InvalidSqrtPriceLimit,
    #[msg("Swap output below minimum (slippage exceeded)")]
    SlippageExceeded,
    #[msg("Exceeded maximum tick crossings per swap")]
    MaxTickCrossingsExceeded,
    #[msg("No more tick arrays available for swap")]
    NoMoreTickArrays,
    #[msg("Position not empty: liquidity or fees remaining")]
    PositionNotEmpty,
    #[msg("Math overflow")]
    MathOverflow,
    #[msg("Division by zero")]
    DivisionByZero,
    #[msg("Insufficient liquidity for swap")]
    InsufficientLiquidity,
    #[msg("Position does not belong to this pool")]
    PositionPoolMismatch,
    #[msg("Invalid input mint")]
    InvalidInputMint,
    #[msg("Swap produced zero output")]
    ZeroOutput,
}
