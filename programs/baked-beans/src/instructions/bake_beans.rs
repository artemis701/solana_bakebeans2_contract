use crate::{constants::*, error::*, states::*, utils::*, events::*};
use anchor_lang::prelude::*;

#[derive(Accounts)]
pub struct BakeBeans<'info> {
    #[account(mut)]
    pub user: Signer<'info>,
    #[account(
      mut,
      seeds = [GLOBAL_STATE_SEED],
      bump,
    )]
    pub global_state: Account<'info, GlobalState>,

    #[account(
        mut,
        seeds = [USER_STATE_SEED, user.key().as_ref()],
        bump,
        has_one = user,
    )]
    pub user_state: Account<'info, UserState>,

    pub system_program: Program<'info, System>,
    pub rent: Sysvar<'info, Rent>,
}

pub fn handle(ctx: Context<BakeBeans>, only_rebaking: u8) -> Result<()> {
    let cur_timestamp = Clock::get()?.unix_timestamp as u64;
    let accts = ctx.accounts;
    require!(max_tvl_reached(&accts.user_state) == false, BeanError::WalletTvlReached);
    require!(accts.user_state.total_deposit > 0, BeanError::InvalidAction);
    if only_rebaking == 1 {
      require!(
        beans_to_sol(rewarded_beans(&accts.user_state)) > MIN_BAKE, 
        BeanError::UnderMinBake
      );
    }

    let beans_from = accts.user_state.beans;
    let beans_from_rewards = rewarded_beans(&accts.user_state);
    let total_beans = add_beans(&accts.user_state, beans_from_rewards);
    accts.user_state.beans = total_beans;
    accts.user_state.baked_at = cur_timestamp;

    emit!(EventBaked {
      user_address: accts.user.key(),
      ref_address: accts.user_state.upline,
      beans_from,
      beans_to: accts.user_state.beans
    });

    Ok(())
}
