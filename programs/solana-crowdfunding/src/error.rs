use anchor_lang::prelude::*;

#[error_code]
pub enum CrowdfundingError {
    #[msg("Deadline must be in the future")]
    DeadlineInPast,
    #[msg("Goal must be greater than zero")]
    GoalZero,
    #[msg("Campaign goal not reached")]
    GoalNotReached,
    #[msg("Campaign goal already reached, no refund")]
    GoalReached,
    #[msg("Deadline not yet passed")]
    DeadlineNotPassed,
    #[msg("Not the campaign creator")]
    NotCreator,
    #[msg("Campaign already claimed")]
    AlreadyClaimed,
    #[msg("Deadline Passed")]
    DeadlinePassed,
    #[msg("Campaign is cancelled")]
    CampaignCancelled,
    #[msg("Title too long (max 50 chars)")]
    TitleTooLong,
    #[msg("Description too long (max 200 chars)")]
    DescriptionTooLong,
    #[msg("Campaign is not cancelled and deadline not passed or goal reached")]
    RefundNotAllowed,
}