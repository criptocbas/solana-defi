use anchor_lang::prelude::*;

pub mod constants;
pub mod errors;
pub mod instructions;
pub mod math;
pub mod state;

use instructions::*;

declare_id!("4niQV7cydNxRakwi4hM2jhSkp6dwg4abuzx5HsAwDz95");

#[program]
pub mod kusd {
    use super::*;

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

    pub fn init_vault(
        ctx: Context<InitVault>,
        max_ltv_bps: u16,
        liquidation_threshold_bps: u16,
        liquidation_bonus_bps: u16,
        stability_fee_bps: u16,
        debt_ceiling: u64,
        oracle_max_staleness: u64,
    ) -> Result<()> {
        instructions::init_vault::handle_init_vault(
            ctx,
            max_ltv_bps,
            liquidation_threshold_bps,
            liquidation_bonus_bps,
            stability_fee_bps,
            debt_ceiling,
            oracle_max_staleness,
        )
    }

    pub fn open_position(ctx: Context<OpenPosition>) -> Result<()> {
        instructions::open_position::handle_open_position(ctx)
    }

    pub fn deposit_collateral(ctx: Context<DepositCollateral>, amount: u64) -> Result<()> {
        instructions::deposit_collateral::handle_deposit_collateral(ctx, amount)
    }

    pub fn mint_kusd(ctx: Context<MintKusd>, amount: u64) -> Result<()> {
        instructions::mint_kusd::handle_mint_kusd(ctx, amount)
    }

    pub fn repay_kusd(ctx: Context<RepayKusd>, amount: u64) -> Result<()> {
        instructions::repay_kusd::handle_repay_kusd(ctx, amount)
    }

    pub fn withdraw_collateral(ctx: Context<WithdrawCollateral>, amount: u64) -> Result<()> {
        instructions::withdraw_collateral::handle_withdraw_collateral(ctx, amount)
    }

    pub fn liquidate(ctx: Context<Liquidate>, repay_amount: u64) -> Result<()> {
        instructions::liquidate::handle_liquidate(ctx, repay_amount)
    }

    pub fn accrue_fees(ctx: Context<AccrueFees>) -> Result<()> {
        instructions::accrue_fees::handle_accrue_fees(ctx)
    }

    pub fn set_halt(ctx: Context<SetHalt>, halted: bool) -> Result<()> {
        instructions::set_halt::handle_set_halt(ctx, halted)
    }
}
