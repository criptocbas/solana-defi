use anchor_lang::prelude::*;

use crate::constants::*;
use crate::state::Vault;

#[derive(Accounts)]
pub struct SetHalt<'info> {
    #[account(
        mut,
        seeds = [VAULT_SEED, vault.underlying_mint.as_ref()],
        bump = vault.vault_bump,
        has_one = admin,
    )]
    pub vault: Account<'info, Vault>,

    pub admin: Signer<'info>,
}

pub fn handle_set_halt(ctx: Context<SetHalt>, halted: bool) -> Result<()> {
    ctx.accounts.vault.halted = halted;
    Ok(())
}
