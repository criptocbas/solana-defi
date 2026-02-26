use anchor_lang::prelude::*;

pub mod constants;
pub mod errors;
pub mod instructions;
pub mod math;
pub mod state;

use instructions::*;

declare_id!("FEiBosN66wZt8wYTzpUPoCeqqzbKG9FATmeWnm8RNZE1");

#[program]
pub mod kvault {
    use super::*;

    pub fn init_vault(
        ctx: Context<InitVault>,
        performance_fee_bps: u16,
        management_fee_bps: u16,
        deposit_cap: u64,
    ) -> Result<()> {
        instructions::init_vault::handle_init_vault(ctx, performance_fee_bps, management_fee_bps, deposit_cap)
    }

    pub fn deposit(ctx: Context<Deposit>, amount: u64) -> Result<()> {
        instructions::deposit::handle_deposit(ctx, amount)
    }

    pub fn withdraw(ctx: Context<Withdraw>, shares: u64) -> Result<()> {
        instructions::withdraw::handle_withdraw(ctx, shares)
    }

    pub fn allocate(ctx: Context<Allocate>, amount: u64) -> Result<()> {
        instructions::allocate::handle_allocate(ctx, amount)
    }

    pub fn deallocate(ctx: Context<Deallocate>, klend_shares: u64) -> Result<()> {
        instructions::deallocate::handle_deallocate(ctx, klend_shares)
    }

    pub fn harvest(ctx: Context<Harvest>) -> Result<()> {
        instructions::harvest::handle_harvest(ctx)
    }

    pub fn set_halt(ctx: Context<SetHalt>, halted: bool) -> Result<()> {
        instructions::set_halt::handle_set_halt(ctx, halted)
    }
}
