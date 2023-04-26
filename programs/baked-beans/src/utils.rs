use crate::{constants::*, states::*};
use anchor_lang::prelude::*;

pub fn percent_from_amount(amount: u64, fee: u64) -> u64 {
  amount * fee / 100
}

pub fn sol_to_beans(sol_amount: u64) -> u64 {
  sol_amount / SOL_PER_BEAN
}

pub fn beans_to_sol(beans_amount: u64) -> u64 {
  beans_amount * SOL_PER_BEAN
}

pub fn add_beans(user_state: &UserState, beans_to_add: u64) -> u64 {
  let total_beans = user_state.beans + beans_to_add;
  let max_beans = sol_to_beans(MAX_WALLET_TVL_IN_SOL);
  if total_beans > max_beans { max_beans }
  else { total_beans }
}

pub fn max_tvl_reached(user_state: &UserState) -> bool {
  user_state.beans >= sol_to_beans(MAX_WALLET_TVL_IN_SOL)
}

pub fn max_payout_reached(user_state: &UserState) -> bool {
  user_state.total_payout >= max_payout(user_state)
}

pub fn max_payout(user_state: &UserState) -> u64 {
  user_state.total_deposit * 3
}

pub fn calc_giveaway_amount(user_state: &UserState, sol_withdrawal_amount: u64) -> u64 {
  percent_from_amount(sol_withdrawal_amount, has_bean_taxed(user_state)) / 2
}

pub fn seconds_since_last_eat(user_state: &UserState) -> u64 {
  let mut last_ate_or_first_deposit = user_state.ate_at;
  if last_ate_or_first_deposit == 0 {
    last_ate_or_first_deposit = user_state.first_deposit_time;
  }

  let cur_timestamp = Clock::get().unwrap().unix_timestamp as u64;
  let seconds_passed = cur_timestamp - last_ate_or_first_deposit;
  seconds_passed
}

pub fn days_since_last_eat(user_state: &UserState) -> u64 {
  let seconds_passed = seconds_since_last_eat(user_state);
  seconds_passed / SECONDS_PER_DAY
}

pub fn has_bean_taxed(user_state: &UserState) -> u64 {
  let days_passed: u64 = days_since_last_eat(user_state);
  let last_digit = days_passed % 10;

  match last_digit {
    0 => 90,
    1 => 80,
    2 => 70,
    3 => 60,
    4 => 50,
    5 => 40,
    6 => 30,
    7 => 20,
    8 => 10,
    _ => 0
  }
}

pub fn rewarded_beans(user_state: &UserState) -> u64 {
  let seconds_passed = seconds_since_last_action(user_state);
  let daily_reward_factor = daily_reward(user_state);
  let beans_rewarded = calc_beans_reward(user_state, seconds_passed, daily_reward_factor);

  if beans_rewarded >= sol_to_beans(MAX_DAILY_REWARDS_IN_SOL) {
    sol_to_beans(MAX_DAILY_REWARDS_IN_SOL)
  } else {
    beans_rewarded
  }
}

pub fn seconds_since_last_action(user_state: &UserState) -> u64 {
  let cur_timestamp = Clock::get().unwrap().unix_timestamp as u64;
  let mut last_timestamp = user_state.baked_at;
  
  if last_timestamp == 0 {
    last_timestamp = user_state.ate_at;
  }

  if last_timestamp == 0 {
    last_timestamp = user_state.first_deposit_time;
  }

  cur_timestamp - last_timestamp
}

pub fn daily_reward(user_state: &UserState) -> u64 {
  let ref_count = user_state.bonus_eligible_referrals.len();
  if ref_count < 10 { 30000 }
  else if ref_count < 25 { 35000 }
  else if ref_count < 50 { 40000 }
  else if ref_count < 100 { 45000 }
  else if ref_count < 150 { 50000 }
  else if ref_count < 250 { 55000 }
  else { 60000 }
}

pub fn calc_beans_reward(user_state: &UserState, seconds_passed: u64, daily_reward_factor: u64) -> u64 {
  let reward_per_day = percent_from_amount(user_state.beans, daily_reward_factor);
  let rewards_per_second = reward_per_day * 1000 / SECONDS_PER_DAY;
  let beans_rewarded = rewards_per_second * seconds_passed / 10_000_000;
  beans_rewarded
}

pub fn add_withdrawal_taxes(user_state: &UserState, sol_withdrawal_amt: u64) -> u64{
  percent_from_amount(sol_withdrawal_amt, 100u64 - has_bean_taxed(user_state))
}

pub fn ref_exists(ref_user_state: &UserState, user_key: Pubkey) -> bool {
  let res = ref_user_state.bonus_eligible_referrals.iter().find(|&referral| referral.eq(&user_key));
  res.is_some()
}

