use anchor_lang::prelude::*;

#[error_code]
pub enum BeanError {
    #[msg("Not allowed authority")]
    NotAllowedAuthority,

    #[msg("Should be over minimum amount")]
    InsufficientAmount,

    #[msg("Incorrect User State")]
    IncorrectUserState,

    #[msg("Incorrect Referral Pubkey")]
    IncorrectReferral,

    #[msg("Deposit doesn't meet the minimum requirements")]
    InsufficientDeposit,

    #[msg("Max total deposit reached")]
    TotalDepositReached,

    #[msg("invalid address to initialise")]
    ZeroAddressDetected,

    #[msg("Referrer should have invested")]
    ReferrerShouldInvest,

    #[msg("Total wallet TVL reached")]
    WalletTvlReached,

    #[msg("InvalidAction")]
    InvalidAction,

    #[msg("Rewards must be equal or higher than 0.01 BNB to bake")]
    UnderMinBake,

    #[msg("You have reached max payout")]
    MaxPayoutReached,

    
}
