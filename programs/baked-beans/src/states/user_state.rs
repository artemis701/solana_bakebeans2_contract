use anchor_lang::prelude::*;

#[account]
#[derive(Default)]
pub struct UserState {
    pub bump: u8,

    // user
    pub user: Pubkey,

    pub total_deposit: u64,
    pub total_payout: u64,

    // first deposit time
    pub first_deposit_time: u64,
    pub ate_at: u64,
    pub baked_at: u64,
    
    pub beans: u64,
    pub upline: Pubkey,
    
    pub has_referred: u8,
    
    pub referrals: Vec<Pubkey>,
    pub bonus_eligible_referrals: Vec<Pubkey>,
}
