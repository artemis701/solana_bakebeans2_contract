use crate::{constants::*, error::*, states::*, utils::*, events::*};
use anchor_lang::prelude::*;
use solana_program::{program::invoke_signed, system_instruction};
#[derive(Accounts)]
pub struct EatBeans<'info> {
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
        seeds = [VAULT_SEED],
        bump
    )]
    /// CHECK: this should be checked with address in global_state
    pub vault: AccountInfo<'info>,

    #[account(mut, address = global_state.dev_account)]
    /// CHECK: this should be set by admin
    pub dev_account: AccountInfo<'info>,

    #[account(mut, address = global_state.marketing_account)]
    /// CHECK: this should be set by admin
    pub marketing_account: AccountInfo<'info>,

    #[account(mut, address = global_state.ceo_account)]
    /// CHECK: this should be set by admin
    pub ceo_account: AccountInfo<'info>,

    #[account(mut, address = global_state.giveaway_account)]
    /// CHECK: this should be set by admin
    pub giveaway_account: AccountInfo<'info>,

    #[account(
        mut,
        seeds = [USER_STATE_SEED, user.key().as_ref()],
        bump,
        has_one = user
    )]
    pub user_state: Account<'info, UserState>,

    pub system_program: Program<'info, System>,
}

impl<'info> EatBeans<'info> {
    pub fn validate(&self) -> Result<()> {
        Ok(())
    }
}

#[access_control(ctx.accounts.validate())]
pub fn handle(ctx: Context<EatBeans>) -> Result<()> {
    let cur_timestamp = Clock::get()?.unix_timestamp as u64;
    let accts = ctx.accounts;

    require!(accts.user_state.total_deposit > 0, BeanError::InvalidAction);
    require!(max_payout_reached(&accts.user_state) == false, BeanError::MaxPayoutReached);

    let beans_before_fee = rewarded_beans(&accts.user_state);
    let beans_in_sol_before_fee = beans_to_sol(beans_before_fee);
    let total_sol_fee = percent_from_amount(beans_in_sol_before_fee, WITHDRAWAL_FEE);
    
    let mut sol_to_eat = beans_in_sol_before_fee - total_sol_fee;
    let for_giveway = calc_giveaway_amount(&accts.user_state, sol_to_eat);
    sol_to_eat = add_withdrawal_taxes(&accts.user_state, sol_to_eat);

    if beans_in_sol_before_fee + accts.user_state.total_payout >= max_payout(&accts.user_state) {
      sol_to_eat = max_payout(&accts.user_state) - accts.user_state.total_payout;
      accts.user_state.total_payout = max_payout(&accts.user_state);
    } else {
      let after_tax = add_withdrawal_taxes(
        &accts.user_state,
        beans_in_sol_before_fee
      );
      accts.user_state.total_payout = accts.user_state.total_payout + after_tax;
    }

    accts.user_state.ate_at = cur_timestamp;
    accts.user_state.baked_at = cur_timestamp;
    
    let bump = ctx.bumps.get("vault").unwrap();
    // send giveaway
    invoke_signed(
      &system_instruction::transfer(&accts.vault.key(), &accts.giveaway_account.key(), for_giveway),
      &[
          accts.vault.to_account_info().clone(),
          accts.giveaway_account.clone(),
          accts.system_program.to_account_info().clone(),
      ],
      &[&[VAULT_SEED, &[*bump]]],
    )?;
    
    // fee distribution
    let dev_fee = percent_from_amount(total_sol_fee, DEV_FEE);
    let market_fee = percent_from_amount(total_sol_fee, MARKETING_FEE);
    let ceo_fee = percent_from_amount(total_sol_fee, CEO_FEE);

    // send dev_fee
    invoke_signed(
        &system_instruction::transfer(&accts.vault.key(), &accts.dev_account.key(), dev_fee),
        &[
            accts.vault.to_account_info().clone(),
            accts.dev_account.clone(),
            accts.system_program.to_account_info().clone(),
        ],
        &[&[VAULT_SEED, &[*bump]]],
    )?;
    // send marketing_fee
    invoke_signed(
          &system_instruction::transfer(&accts.vault.key(), &accts.marketing_account.key(), market_fee),
          &[
              accts.vault.to_account_info().clone(),
              accts.marketing_account.clone(),
              accts.system_program.to_account_info().clone(),
          ],
          &[&[VAULT_SEED, &[*bump]]],
      )?;
    // send ceo_fee
    invoke_signed(
        &system_instruction::transfer(&accts.vault.key(), &accts.ceo_account.key(), ceo_fee),
        &[
            accts.vault.to_account_info().clone(),
            accts.ceo_account.clone(),
            accts.system_program.to_account_info().clone(),
        ],
        &[&[VAULT_SEED, &[*bump]]],
    )?;

    // send to user
    invoke_signed(
        &system_instruction::transfer(&accts.vault.key(), &accts.user.key(), sol_to_eat),
        &[
            accts.vault.to_account_info().clone(),
            accts.user.to_account_info().clone(),
            accts.system_program.to_account_info().clone(),
        ],
        &[&[VAULT_SEED, &[*bump]]],
    )?;

    // lamports should be bigger than zero to prevent rent exemption
    let rent = Rent::default();
    let required_lamports = rent
        .minimum_balance(0)
        .max(1)
        .saturating_sub(accts.vault.to_account_info().lamports());
    require!(
        **accts.vault.lamports.borrow() > required_lamports,
        BeanError::InsufficientAmount
    );

    emit!(EventAte {
      user_address: accts.user.key(),
      sol_to_eat,
      beans_before_fee
    });
    Ok(())
}
