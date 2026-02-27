use anchor_lang::prelude::*;

pub mod constants;
pub mod errors;
pub mod instructions;
pub mod math;
pub mod state;

use instructions::*;

declare_id!("85ZLT4UTCsk3btQUCXuj6jmKo9cR9JKL1g9QEBKabQvn");

#[program]
pub mod klev {
    use super::*;

    pub fn init_vault(
        ctx: Context<InitVault>,
        performance_fee_bps: u16,
        management_fee_bps: u16,
        deposit_cap: u64,
        max_leverage_bps: u16,
        min_health_factor_bps: u16,
    ) -> Result<()> {
        instructions::init_vault::handle_init_vault(
            ctx,
            performance_fee_bps,
            management_fee_bps,
            deposit_cap,
            max_leverage_bps,
            min_health_factor_bps,
        )
    }

    pub fn deposit(ctx: Context<Deposit>, amount: u64) -> Result<()> {
        instructions::deposit::handle_deposit(ctx, amount)
    }

    pub fn withdraw(ctx: Context<Withdraw>, shares: u64) -> Result<()> {
        instructions::withdraw::handle_withdraw(ctx, shares)
    }

    pub fn leverage_up(
        ctx: Context<LeverageUp>,
        collateral_amount: u64,
        borrow_amount: u64,
        min_swap_output: u64,
    ) -> Result<()> {
        instructions::leverage_up::handle_leverage_up(ctx, collateral_amount, borrow_amount, min_swap_output)
    }

    pub fn deleverage(
        ctx: Context<Deleverage>,
        withdraw_klend_shares: u64,
        swap_amount: u64,
        min_swap_output: u64,
        repay_amount: u64,
    ) -> Result<()> {
        instructions::deleverage::handle_deleverage(ctx, withdraw_klend_shares, swap_amount, min_swap_output, repay_amount)
    }

    pub fn harvest(ctx: Context<Harvest>) -> Result<()> {
        instructions::harvest::handle_harvest(ctx)
    }

    pub fn set_halt(ctx: Context<SetHalt>, halted: bool) -> Result<()> {
        instructions::set_halt::handle_set_halt(ctx, halted)
    }
}
