use anchor_lang::prelude::*;

#[account]
pub struct Campaign {
    pub creator: Pubkey,
    pub goal: u64,
    pub raised: u64,
    pub deadline: i64,
    pub claimed: bool,
}

impl Campaign {
    pub const INIT_SPACE: usize = 32 + 8 + 8 + 8 + 1;
}

#[account]
pub struct Contribution {
    pub donor: Pubkey,
    pub campaign: Pubkey,
    pub amount: u64,
}

impl Contribution {
    pub const INIT_SPACE: usize = 32 + 32 + 8;
}

