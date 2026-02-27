use anchor_lang::prelude::*;

use crate::constants::*;
use crate::state::LeveragedVault;

#[derive(Accounts)]
pub struct SetHalt<'info> {
    #[account(
        mut,
        seeds = [LEVERAGED_VAULT_SEED, vault.collateral_mint.as_ref(), vault.debt_mint.as_ref()],
        bump = vault.vault_bump,
        has_one = admin,
    )]
    pub vault: Account<'info, LeveragedVault>,

    pub admin: Signer<'info>,
}

pub fn handle_set_halt(ctx: Context<SetHalt>, halted: bool) -> Result<()> {
    ctx.accounts.vault.halted = halted;
    Ok(())
}
