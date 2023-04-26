use anchor_lang::prelude::*;

#[account]
#[derive(Default)]
pub struct GlobalState {
    // to avoid reinitialization attack
    pub is_initialized: u8,
    // admin
    pub authority: Pubkey,
    // vault
    pub vault: Pubkey,
    // dev_account
    pub dev_account: Pubkey,
    // marketing_account
    pub marketing_account: Pubkey,
    // giveaway_account
    pub giveaway_account: Pubkey,
    // ceo_account
    pub ceo_account: Pubkey,

    pub total_bakers: u64
}
