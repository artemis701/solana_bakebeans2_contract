use anchor_lang::prelude::*;

pub mod constants;
pub mod error;
pub mod instructions;
pub mod states;
pub mod utils;
pub mod events;

use instructions::*;

declare_id!("CzBzTMfRhJViNwXC6fZTLHcfEDsn6xEM7dPjeCZ2HU1f");
#[program]
pub mod baked_beans {
    use super::*;

    pub fn initialize(ctx: Context<Initialize>, new_authority: Pubkey) -> Result<()> {
        initialize::handle(ctx, new_authority)
    }

    pub fn buy_beans(ctx: Context<BuyBeans>, ref_user: Pubkey, amount: u64) -> Result<()> {
        buy_beans::handle(ctx, ref_user, amount)
    }

    pub fn eat_beans(ctx: Context<EatBeans>) -> Result<()> {
        eat_beans::handle(ctx)
    }

    pub fn bake_beans(ctx: Context<BakeBeans>, only_rebaking: u8) -> Result<()> {
        bake_beans::handle(ctx, only_rebaking)
    }

    pub fn init_user_state(ctx: Context<InitUserState>, user_key: Pubkey) -> Result<()> {
      init_user_state::handle(ctx, user_key)
    }
}
