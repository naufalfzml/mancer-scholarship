pub mod constants;
pub mod error;
pub mod instructions;
pub mod state;

use anchor_lang::prelude::*;

pub use constants::*;
pub use instructions::*;
pub use state::*;

declare_id!("8RjcjbiGt6FxemB4mG3yHvvmnKN6yEQPiwt4a9NDtUH");

#[program]
pub mod solana_crowdfunding {
    use super::*;

    pub fn create_campaign(
        ctx: Context<CreateCampaign>,
        goal: u64,
        deadline: i64,
        title: String,
        description: String,
    ) -> Result<()> {
        create_campaign::create_campaign_handler(ctx, goal, deadline, title, description)
    }

    pub fn contribute_campaign(ctx: Context<Contribute>, amount: u64) -> Result<()> {
        contribute::contribute_handler(ctx, amount)
    }

    pub fn withdraw(ctx: Context<Withdraw>) -> Result<()> {
        withdraw::withdraw_handler(ctx)
    }

    pub fn refund(ctx: Context<Refund>) -> Result<()> {
        refund::refund_handler(ctx)
    }

    pub fn cancel_campaign(ctx: Context<CancelCampaign>) -> Result<()> {
        cancel_campaign::cancel_campaign_handler(ctx)
    }
}
