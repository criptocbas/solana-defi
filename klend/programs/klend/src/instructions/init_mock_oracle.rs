use anchor_lang::prelude::*;

use crate::constants::*;
use crate::state::MockOracle;

#[derive(Accounts)]
pub struct InitMockOracle<'info> {
    #[account(mut)]
    pub payer: Signer<'info>,

    /// CHECK: Any SPL token mint; validated by PDA seed derivation
    pub token_mint: AccountInfo<'info>,

    #[account(
        init,
        payer = payer,
        space = 8 + 32 + 8 + 1 + 8 + 1,
        seeds = [MOCK_ORACLE_SEED, token_mint.key().as_ref()],
        bump,
    )]
    pub oracle: Account<'info, MockOracle>,

    pub system_program: Program<'info, System>,
}

pub fn handle_init_mock_oracle(
    ctx: Context<InitMockOracle>,
    price: u64,
    decimals: u8,
) -> Result<()> {
    let oracle = &mut ctx.accounts.oracle;
    oracle.token_mint = ctx.accounts.token_mint.key();
    oracle.price = price;
    oracle.decimals = decimals;
    oracle.timestamp = Clock::get()?.unix_timestamp;
    oracle.bump = ctx.bumps.oracle;
    Ok(())
}
