use anchor_lang::prelude::*;

use crate::constants::*;
use crate::instructions::common::accrue_vault_fees;
use crate::state::CdpVault;

#[derive(Accounts)]
pub struct AccrueFees<'info> {
    #[account(
        mut,
        seeds = [CDP_VAULT_SEED, vault.collateral_mint.as_ref()],
        bump = vault.vault_bump,
    )]
    pub vault: Account<'info, CdpVault>,
}

pub fn handle_accrue_fees(ctx: Context<AccrueFees>) -> Result<()> {
    let vault = &mut ctx.accounts.vault;
    let clock = Clock::get()?;
    accrue_vault_fees(vault, clock.unix_timestamp)?;
    Ok(())
}
