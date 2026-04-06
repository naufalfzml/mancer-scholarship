use anchor_lang::prelude::*;
use crate::state::Campaign;
use crate::error::CrowdfundingError;

#[derive(Accounts)]
pub struct CreateCampaign<'info> {
    #[account(mut)]
    pub creator: Signer<'info>,

    #[account(
        init,
        payer = creator,
        space = 8 + Campaign::INIT_SPACE,
    )]
    pub campaign: Account<'info, Campaign>,
    pub system_program: Program<'info, System>,
}

pub fn create_campaign_handler(ctx: Context<CreateCampaign>, goal: u64, deadline: i64) -> Result<()> {
    let clock = Clock::get()?;
    let current_time = clock.unix_timestamp;
    let campaign = &mut ctx.accounts.campaign;

    require!(deadline > current_time, CrowdfundingError::DeadlineInPast);
    require!(goal > 0, CrowdfundingError::GoalZero);

    campaign.creator = ctx.accounts.creator.key();
    campaign.goal = goal;
    campaign.raised = 0;
    campaign.deadline = deadline;
    campaign.claimed = false;

    msg!("Campaign created: goal={}, deadline={}", goal, deadline);

    Ok(())
}