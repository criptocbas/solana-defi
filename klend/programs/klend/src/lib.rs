use anchor_lang::prelude::*;

pub mod constants;
pub mod errors;
pub mod instructions;
pub mod math;
pub mod state;

use instructions::*;
use state::ReserveConfig;

declare_id!("D91U4ZA4bcSWRNhqAf9oMPBMNYhEkwZNooXPUUZSM68v");

#[program]
pub mod klend {
    use super::*;

    pub fn init_market(ctx: Context<InitMarket>) -> Result<()> {
        instructions::init_market::handle_init_market(ctx)
    }

    pub fn init_mock_oracle(
        ctx: Context<InitMockOracle>,
        price: u64,
        decimals: u8,
    ) -> Result<()> {
        instructions::init_mock_oracle::handle_init_mock_oracle(ctx, price, decimals)
    }

    pub fn update_mock_oracle(ctx: Context<UpdateMockOracle>, price: u64) -> Result<()> {
        instructions::update_mock_oracle::handle_update_mock_oracle(ctx, price)
    }

    pub fn init_reserve(ctx: Context<InitReserve>, config: ReserveConfig) -> Result<()> {
        instructions::init_reserve::handle_init_reserve(ctx, config)
    }

    pub fn refresh_reserve(ctx: Context<RefreshReserve>) -> Result<()> {
        instructions::refresh_reserve::handle_refresh_reserve(ctx)
    }

    pub fn deposit(ctx: Context<Deposit>, amount: u64) -> Result<()> {
        instructions::deposit::handle_deposit(ctx, amount)
    }

    pub fn withdraw(ctx: Context<Withdraw>, shares: u64) -> Result<()> {
        instructions::withdraw::handle_withdraw(ctx, shares)
    }

    pub fn borrow(ctx: Context<Borrow>, amount: u64) -> Result<()> {
        instructions::borrow::handle_borrow(ctx, amount)
    }

    pub fn repay(ctx: Context<Repay>, amount: u64) -> Result<()> {
        instructions::repay::handle_repay(ctx, amount)
    }

    pub fn liquidate(ctx: Context<Liquidate>, amount: u64) -> Result<()> {
        instructions::liquidate::handle_liquidate(ctx, amount)
    }
}
