use anchor_lang::prelude::*;

use crate::constants::*;
use crate::state::{CdpPosition, CdpVault};

#[derive(Accounts)]
pub struct OpenPosition<'info> {
    #[account(mut)]
    pub owner: Signer<'info>,

    #[account(
        seeds = [CDP_VAULT_SEED, vault.collateral_mint.as_ref()],
        bump = vault.vault_bump,
    )]
    pub vault: Account<'info, CdpVault>,

    #[account(
        init,
        payer = owner,
        space = 8 + CdpPosition::SPACE,
        seeds = [CDP_POSITION_SEED, vault.key().as_ref(), owner.key().as_ref()],
        bump,
    )]
    pub position: Account<'info, CdpPosition>,

    pub system_program: Program<'info, System>,
}

pub fn handle_open_position(ctx: Context<OpenPosition>) -> Result<()> {
    let position = &mut ctx.accounts.position;
    position.vault = ctx.accounts.vault.key();
    position.owner = ctx.accounts.owner.key();
    position.collateral_amount = 0;
    position.debt_shares = 0;
    position.bump = ctx.bumps.position;
    Ok(())
}
