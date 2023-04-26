use crate::{constants::*, error::*, states::*, utils::*, events::*};
use anchor_lang::prelude::*;
use solana_program::{program::invoke, system_instruction};

#[derive(Accounts)]
#[instruction(ref_user: Pubkey)]
pub struct BuyBeans<'info> {
    #[account(mut)]
    pub user: Signer<'info>,

    #[account(
      mut,
      seeds = [GLOBAL_STATE_SEED],
      bump,
    )]
    pub global_state: Account<'info, GlobalState>,

    #[account(mut, address = global_state.dev_account)]
    /// CHECK: this should be set by admin
    pub dev_account: AccountInfo<'info>,

    #[account(mut, address = global_state.marketing_account)]
    /// CHECK: this should be set by admin
    pub marketing_account: AccountInfo<'info>,

    #[account(mut, address = global_state.ceo_account)]
    /// CHECK: this should be set by admin
    pub ceo_account: AccountInfo<'info>,

    #[account(
        mut,
        address = global_state.vault
    )]
    /// CHECK: this should be set by admin
    pub vault: AccountInfo<'info>,

    #[account(
        mut,
        seeds = [USER_STATE_SEED, user.key().as_ref()],
        bump,
    )]
    pub user_state: Account<'info, UserState>,

    #[account(
      mut,
      seeds = [USER_STATE_SEED, ref_user.key().as_ref()],
      bump,
      constraint = user_state.user != ref_user_state.user
    )]
    pub ref_user_state: Account<'info, UserState>,

    pub system_program: Program<'info, System>,
    pub rent: Sysvar<'info, Rent>,
}

pub fn handle(ctx: Context<BuyBeans>, ref_user: Pubkey, sol_amount: u64) -> Result<()> {
    let accts = ctx.accounts;
    let user_key = accts.user.key();
    require!(sol_amount >= MIN_DEPOSIT, BeanError::InsufficientDeposit);
    require!(accts.user_state.total_deposit <= MAX_WALLET_TVL_IN_SOL, BeanError::TotalDepositReached);
    require!(ref_user.eq(&accts.global_state.authority) || accts.ref_user_state.total_deposit > 0, BeanError::ReferrerShouldInvest);

    let cur_timestamp = Clock::get()?.unix_timestamp as u64;

    let beans_from = accts.user_state.beans;
    let total_sol_fee = percent_from_amount(sol_amount, DEPOSIT_FEE);
    let sol_value = sol_amount - total_sol_fee;
    let beans_bought = sol_to_beans(sol_value);

    let total_beans_bought = add_beans(&accts.user_state, beans_bought);
    accts.user_state.beans = total_beans_bought;

    // referrer
    if accts.user_state.has_referred == 0 {
      accts.user_state.has_referred = 1;
      accts.user_state.upline = ref_user;
      accts.ref_user_state.referrals.push(user_key);
      if accts.user_state.total_deposit == 0 {
        let ref_bonus = percent_from_amount(sol_to_beans(sol_amount), FIRST_DEPOSIT_REF_BONUS);
        accts.ref_user_state.beans = add_beans(&accts.ref_user_state, ref_bonus);
      }
    }
    
    if accts.user_state.total_deposit == 0 {
      accts.user_state.first_deposit_time = cur_timestamp;
      accts.global_state.total_bakers = accts.global_state.total_bakers + 1;
    }

    accts.user_state.total_deposit = accts.user_state.total_deposit + sol_amount;

    if 
      accts.user_state.has_referred == 1 &&
      accts.user_state.total_deposit  >= MIN_REF_DEPOSIT_FOR_BONUS &&
      ref_exists(&accts.ref_user_state, user_key) == false
    {
        accts.ref_user_state.bonus_eligible_referrals.push(user_key);
    }

    // fee distribution
    let dev_fee = percent_from_amount(total_sol_fee, DEV_FEE);
    let market_fee = percent_from_amount(total_sol_fee, MARKETING_FEE);
    let ceo_fee = percent_from_amount(total_sol_fee, CEO_FEE);

    let remained_fee = total_sol_fee - dev_fee - market_fee - ceo_fee;

    // send dev_fee
    invoke(
        &system_instruction::transfer(&user_key, &accts.dev_account.key(), dev_fee),
        &[
            accts.user.to_account_info().clone(),
            accts.dev_account.clone(),
            accts.system_program.to_account_info().clone(),
        ],
    )?;
    // send marketing_fee
    invoke(
          &system_instruction::transfer(&user_key, &accts.marketing_account.key(), market_fee),
          &[
              accts.user.to_account_info().clone(),
              accts.marketing_account.clone(),
              accts.system_program.to_account_info().clone(),
          ],
      )?;
    // send ceo_fee
    invoke(
        &system_instruction::transfer(&user_key, &accts.ceo_account.key(), ceo_fee),
        &[
            accts.user.to_account_info().clone(),
            accts.ceo_account.clone(),
            accts.system_program.to_account_info().clone(),
        ],
    )?;

    // add vault <- sol_amount - fee
    invoke(
      &system_instruction::transfer(&user_key, &accts.vault.key(), sol_value + remained_fee),
      &[
          accts.user.to_account_info().clone(),
          accts.vault.clone(),
          accts.system_program.to_account_info().clone(),
      ],
    )?;

    // todo: handleBake, emit Event
    emit!(EventBoughtBeans {
      user_address: accts.user.key(),
      ref_address: ref_user,
      sol_amount,
      beans_from,
      beans_to: accts.user_state.beans
    });
    Ok(())
}
