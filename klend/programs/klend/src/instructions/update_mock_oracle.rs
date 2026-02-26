use anchor_lang::prelude::*;

use crate::state::MockOracle;

#[derive(Accounts)]
pub struct UpdateMockOracle<'info> {
    pub payer: Signer<'info>,

    #[account(mut)]
    pub oracle: Account<'info, MockOracle>,
}

pub fn handle_update_mock_oracle(ctx: Context<UpdateMockOracle>, price: u64) -> Result<()> {
    let oracle = &mut ctx.accounts.oracle;
    oracle.price = price;
    oracle.timestamp = Clock::get()?.unix_timestamp;
    Ok(())
}
