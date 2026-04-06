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
}