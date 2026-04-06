use anchor_lang::prelude::*;
use crate::state::Campaign;
use crate::error::CrowdfundingError;

#[derive(Accounts)]
pub struct CancelCampaign<'info> {
    #[account(mut)]
    pub campaign: Account<'info, Campaign>,

    pub creator: Signer<'info>,
}

/// Creator cancels a campaign, enabling immediate refunds for donors.
pub fn cancel_campaign_handler(ctx: Context<CancelCampaign>) -> Result<()> {
    let campaign = &mut ctx.accounts.campaign;

    require!(ctx.accounts.creator.key() == campaign.creator, CrowdfundingError::NotCreator);
    require!(!campaign.claimed, CrowdfundingError::AlreadyClaimed);
    require!(!campaign.cancelled, CrowdfundingError::CampaignCancelled);

    campaign.cancelled = true;

    msg!("Campaign cancelled by creator");

    Ok(())
}
