use anchor_lang::prelude::*;

use crate::constants::*;
use crate::state::CdpVault;

#[derive(Accounts)]
pub struct SetHalt<'info> {
    pub admin: Signer<'info>,

    #[account(
        mut,
        seeds = [CDP_VAULT_SEED, vault.collateral_mint.as_ref()],
        bump = vault.vault_bump,
        has_one = admin,
    )]
    pub vault: Account<'info, CdpVault>,
}

pub fn handle_set_halt(ctx: Context<SetHalt>, halted: bool) -> Result<()> {
    ctx.accounts.vault.halted = halted;
    Ok(())
}
