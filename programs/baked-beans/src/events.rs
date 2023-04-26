use anchor_lang::prelude::*;

#[event]
pub struct EventBoughtBeans {
    pub user_address: Pubkey,
    pub ref_address: Pubkey,
    pub sol_amount: u64,
    pub beans_from: u64,
    pub beans_to: u64
}


#[event]
pub struct EventBaked {
    pub user_address: Pubkey,
    pub ref_address: Pubkey,
    pub beans_from: u64,
    pub beans_to: u64
}


#[event]
pub struct EventAte {
    pub user_address: Pubkey,
    pub sol_to_eat: u64,
    pub beans_before_fee: u64
}