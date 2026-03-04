use anchor_lang::prelude::*;

use crate::constants::SCALE;

/// Global lending market config
#[account]
pub struct LendingMarket {
    pub admin: Pubkey,
    pub bump: u8,
}

/// Reserve configuration (passed as instruction arg)
#[derive(AnchorSerialize, AnchorDeserialize, Clone, Debug)]
pub struct ReserveConfig {
    pub ltv: u16,                   // basis points (8000 = 80%)
    pub liquidation_threshold: u16, // basis points (8500 = 85%)
    pub liquidation_bonus: u16,     // basis points (500 = 5%)
    pub reserve_factor: u16,        // basis points (1000 = 10%)
    pub r_base: u64,                // annual rate scaled by 1e18
    pub r_slope1: u64,              // annual rate scaled by 1e18
    pub r_slope2: u64,              // annual rate scaled by 1e18
    pub u_optimal: u64,             // utilization scaled by 1e18
    pub supply_cap: u64,            // max deposits in token units
    pub borrow_cap: u64,            // max borrows in token units
    pub oracle_max_staleness: u64,  // seconds
}

/// Per-asset reserve
#[account]
pub struct Reserve {
    pub lending_market: Pubkey,
    pub token_mint: Pubkey,
    pub token_vault: Pubkey,
    pub reserve_authority: Pubkey,
    pub oracle: Pubkey,

    // Accounting
    pub deposited_liquidity: u64,
    pub borrowed_liquidity: u64,
    pub accumulated_protocol_fees: u64,
    pub total_shares: u64,

    // Interest tracking (1e18 scaled)
    pub cumulative_borrow_index: u128,
    pub last_update_timestamp: i64,

    pub config: ReserveConfig,

    pub bump: u8,
    pub authority_bump: u8,
}

impl Reserve {
    /// Total assets available in the reserve (cash + borrows - fees)
    pub fn total_assets(&self) -> u64 {
        self.deposited_liquidity
            .saturating_add(self.borrowed_liquidity)
            .saturating_sub(self.accumulated_protocol_fees)
    }

    /// Cash available in vault (deposited_liquidity now tracks physical cash)
    pub fn available_liquidity(&self) -> u64 {
        self.deposited_liquidity
    }
}

/// Mock oracle for testing
#[account]
pub struct MockOracle {
    pub token_mint: Pubkey,
    pub price: u64,    // USD per token * 1e6
    pub decimals: u8,  // token decimals
    pub timestamp: i64,
    pub bump: u8,
}

/// Per-user obligation (position)
#[account]
pub struct Obligation {
    pub lending_market: Pubkey,
    pub owner: Pubkey,
    pub deposits: Vec<ObligationDeposit>,
    pub borrows: Vec<ObligationBorrow>,
    pub bump: u8,
}

/// Tracks a user's deposit in a specific reserve (shares, not tokens)
#[derive(AnchorSerialize, AnchorDeserialize, Clone, Debug, Default)]
pub struct ObligationDeposit {
    pub reserve: Pubkey,
    pub shares: u64,
}

/// Tracks a user's borrow from a specific reserve
#[derive(AnchorSerialize, AnchorDeserialize, Clone, Debug, Default)]
pub struct ObligationBorrow {
    pub reserve: Pubkey,
    pub borrowed_amount_scaled: u128, // amount * SCALE / borrow_index at time of borrow
}

impl ObligationBorrow {
    /// Current debt given the latest borrow index
    pub fn current_debt(&self, current_borrow_index: u128) -> Result<u64> {
        if current_borrow_index == 0 {
            return Ok(0);
        }
        let product = self
            .borrowed_amount_scaled
            .checked_mul(current_borrow_index)
            .ok_or(error!(crate::errors::KlendError::DebtOverflow))?;
        let debt = product / SCALE;
        // Round up to favor protocol
        let remainder = product % SCALE;
        let rounded = if remainder > 0 { debt + 1 } else { debt };
        Ok(rounded as u64)
    }
}
