use anchor_lang::prelude::*;

use crate::constants::*;
use crate::state::LendingMarket;

#[derive(Accounts)]
pub struct InitMarket<'info> {
    #[account(mut)]
    pub admin: Signer<'info>,

    #[account(
        init,
        payer = admin,
        space = 8 + 32 + 1,
        seeds = [LENDING_MARKET_SEED, admin.key().as_ref()],
        bump,
    )]
    pub lending_market: Account<'info, LendingMarket>,

    pub system_program: Program<'info, System>,
}

pub fn handle_init_market(ctx: Context<InitMarket>) -> Result<()> {
    let market = &mut ctx.accounts.lending_market;
    market.admin = ctx.accounts.admin.key();
    market.bump = ctx.bumps.lending_market;
    Ok(())
}
