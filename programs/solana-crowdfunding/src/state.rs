use anchor_lang::prelude::*;

/// On-chain state for a crowdfunding campaign.
#[account]
pub struct Campaign {
    pub creator: Pubkey,
    pub goal: u64,
    pub raised: u64,
    pub deadline: i64,
    pub claimed: bool,
    pub cancelled: bool,
    pub title: String,
    pub description: String,
}

impl Campaign {
    pub const MAX_TITLE_LEN: usize = 50;
    pub const MAX_DESC_LEN: usize = 200;
    pub const INIT_SPACE: usize = 32 + 8 + 8 + 8 + 1 + 1 + (4 + Self::MAX_TITLE_LEN) + (4 + Self::MAX_DESC_LEN);
}

/// Tracks a donor's contribution to a campaign. PDA: [b"contribution", campaign, donor]
#[account]
pub struct Contribution {
    pub donor: Pubkey,
    pub campaign: Pubkey,
    pub amount: u64,
}

impl Contribution {
    pub const INIT_SPACE: usize = 32 + 32 + 8;
}

