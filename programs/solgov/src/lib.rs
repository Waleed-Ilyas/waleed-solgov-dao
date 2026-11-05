use anchor_lang::prelude::*;
use anchor_lang::system_program::{self, Transfer as SystemTransfer};
use anchor_spl::token::{Mint, TokenAccount};

declare_id!("EPhYz9toHQfnd5L61WjvYa9M3fPQ1BwyeVAEYY1XbepC");

const MAX_VOTING_PERIOD_SECONDS: i64 = 90 * 24 * 60 * 60;

#[program]
pub mod solgov {
    use super::*;

    pub fn create_dao(
        ctx: Context<CreateDao>,
        nonce: u64,
        name: String,
        quorum: u64,
    ) -> Result<()> {
        require!(name.len() <= 32, GovernanceError::NameTooLong);
        require!(quorum > 0, GovernanceError::InvalidQuorum);

        let dao = &mut ctx.accounts.dao;
        dao.creator = ctx.accounts.creator.key();
        dao.governance_mint = ctx.accounts.governance_mint.key();
        dao.nonce = nonce;
        dao.name = name;
        dao.quorum = quorum;
        dao.next_proposal_id = 0;
        dao.treasury_bump = ctx.bumps.treasury;
        dao.bump = ctx.bumps.dao;
        Ok(())
    }

    pub fn create_proposal(
        ctx: Context<CreateProposal>,
        title: String,
        description: String,
        recipient: Pubkey,
        amount_lamports: u64,
        voting_period_seconds: i64,
    ) -> Result<()> {
        require!(title.len() <= 64, GovernanceError::TitleTooLong);
        require!(
            description.len() <= 256,
            GovernanceError::DescriptionTooLong
        );
        require!(amount_lamports > 0, GovernanceError::InvalidAmount);
        require!(
            voting_period_seconds > 0 && voting_period_seconds <= MAX_VOTING_PERIOD_SECONDS,
            GovernanceError::InvalidVotingPeriod
        );

        let now = Clock::get()?.unix_timestamp;
        let closes_at = now
            .checked_add(voting_period_seconds)
            .ok_or(GovernanceError::MathOverflow)?;
        let proposal_id = ctx.accounts.dao.next_proposal_id;
        let proposal = &mut ctx.accounts.proposal;
        proposal.dao = ctx.accounts.dao.key();
        proposal.proposer = ctx.accounts.proposer.key();
        proposal.id = proposal_id;
        proposal.title = title;
        proposal.description = description;
        proposal.recipient = recipient;
        proposal.amount_lamports = amount_lamports;
        proposal.yes_votes = 0;
        proposal.no_votes = 0;
        proposal.abstain_votes = 0;
        proposal.quorum = ctx.accounts.dao.quorum;
        proposal.closes_at = closes_at;
        proposal.executed = false;
        proposal.bump = ctx.bumps.proposal;

        ctx.accounts.dao.next_proposal_id = proposal_id
            .checked_add(1)
            .ok_or(GovernanceError::MathOverflow)?;
        Ok(())
    }

    pub fn cast_vote(ctx: Context<CastVote>, choice: u8) -> Result<()> {
        let now = Clock::get()?.unix_timestamp;
        require!(
            now <= ctx.accounts.proposal.closes_at,
            GovernanceError::VotingClosed
        );
        require!(
            ctx.accounts.voter_tokens.amount > 0,
            GovernanceError::NoVotingPower
        );
        require!(choice <= 2, GovernanceError::InvalidVoteChoice);

        let weight = ctx.accounts.voter_tokens.amount;
        let proposal = &mut ctx.accounts.proposal;
        match choice {
            0 => {
                proposal.yes_votes = proposal
                    .yes_votes
                    .checked_add(weight)
                    .ok_or(GovernanceError::MathOverflow)?
            }
            1 => {
                proposal.no_votes = proposal
                    .no_votes
                    .checked_add(weight)
                    .ok_or(GovernanceError::MathOverflow)?
            }
            2 => {
                proposal.abstain_votes = proposal
                    .abstain_votes
                    .checked_add(weight)
                    .ok_or(GovernanceError::MathOverflow)?
            }
            _ => return err!(GovernanceError::InvalidVoteChoice),
        }

        let record = &mut ctx.accounts.vote_record;
        record.proposal = proposal.key();
        record.voter = ctx.accounts.voter.key();
        record.weight = weight;
        record.choice = choice;
        record.bump = ctx.bumps.vote_record;
        Ok(())
    }

    pub fn execute_proposal(ctx: Context<ExecuteProposal>) -> Result<()> {
        let proposal = &ctx.accounts.proposal;
        let now = Clock::get()?.unix_timestamp;
        require!(now > proposal.closes_at, GovernanceError::VotingStillOpen);
        require!(!proposal.executed, GovernanceError::AlreadyExecuted);
        require_keys_eq!(
            ctx.accounts.recipient.key(),
            proposal.recipient,
            GovernanceError::WrongRecipient
        );
        require!(
            proposal_passes(
                proposal.yes_votes,
                proposal.no_votes,
                proposal.abstain_votes,
                proposal.quorum
            )?,
            GovernanceError::ProposalRejected
        );
        require!(
            ctx.accounts.treasury.lamports() >= proposal.amount_lamports,
            GovernanceError::InsufficientTreasury
        );

        let dao_key = ctx.accounts.dao.key();
        let bump = [ctx.accounts.dao.treasury_bump];
        let signer_seeds: &[&[u8]] = &[b"treasury", dao_key.as_ref(), &bump];
        system_program::transfer(
            CpiContext::new_with_signer(
                ctx.accounts.system_program.key(),
                SystemTransfer {
                    from: ctx.accounts.treasury.to_account_info(),
                    to: ctx.accounts.recipient.to_account_info(),
                },
                &[signer_seeds],
            ),
            proposal.amount_lamports,
        )?;

        ctx.accounts.proposal.executed = true;
        Ok(())
    }
}

fn proposal_passes(yes: u64, no: u64, abstain: u64, quorum: u64) -> Result<bool> {
    let participation = yes
        .checked_add(no)
        .and_then(|votes| votes.checked_add(abstain))
        .ok_or(GovernanceError::MathOverflow)?;
    require!(participation >= quorum, GovernanceError::QuorumNotMet);
    Ok(yes > no)
}

#[derive(Accounts)]
#[instruction(nonce: u64)]
pub struct CreateDao<'info> {
    #[account(mut)]
    pub creator: Signer<'info>,
    pub governance_mint: Account<'info, Mint>,
    #[account(
        init,
        payer = creator,
        space = 8 + Dao::INIT_SPACE,
        seeds = [b"dao", creator.key().as_ref(), &nonce.to_le_bytes()],
        bump
    )]
    pub dao: Account<'info, Dao>,
    #[account(mut, seeds = [b"treasury", dao.key().as_ref()], bump)]
    pub treasury: SystemAccount<'info>,
    pub system_program: Program<'info, System>,
}

#[derive(Accounts)]
pub struct CreateProposal<'info> {
    #[account(mut)]
    pub proposer: Signer<'info>,
    #[account(
        mut,
        seeds = [b"dao", dao.creator.as_ref(), &dao.nonce.to_le_bytes()],
        bump = dao.bump
    )]
    pub dao: Account<'info, Dao>,
    #[account(
        init,
        payer = proposer,
        space = 8 + Proposal::INIT_SPACE,
        seeds = [b"proposal", dao.key().as_ref(), &dao.next_proposal_id.to_le_bytes()],
        bump
    )]
    pub proposal: Account<'info, Proposal>,
    pub system_program: Program<'info, System>,
}

#[derive(Accounts)]
pub struct CastVote<'info> {
    #[account(mut)]
    pub voter: Signer<'info>,
    #[account(
        seeds = [b"dao", dao.creator.as_ref(), &dao.nonce.to_le_bytes()],
        bump = dao.bump
    )]
    pub dao: Account<'info, Dao>,
    #[account(
        mut,
        seeds = [b"proposal", dao.key().as_ref(), &proposal.id.to_le_bytes()],
        bump = proposal.bump,
        has_one = dao
    )]
    pub proposal: Account<'info, Proposal>,
    #[account(token::mint = dao.governance_mint, token::authority = voter)]
    pub voter_tokens: Account<'info, TokenAccount>,
    #[account(
        init,
        payer = voter,
        space = 8 + VoteRecord::INIT_SPACE,
        seeds = [b"vote", proposal.key().as_ref(), voter.key().as_ref()],
        bump
    )]
    pub vote_record: Account<'info, VoteRecord>,
    pub system_program: Program<'info, System>,
}

#[derive(Accounts)]
pub struct ExecuteProposal<'info> {
    pub executor: Signer<'info>,
    #[account(
        seeds = [b"dao", dao.creator.as_ref(), &dao.nonce.to_le_bytes()],
        bump = dao.bump
    )]
    pub dao: Account<'info, Dao>,
    #[account(
        mut,
        seeds = [b"proposal", dao.key().as_ref(), &proposal.id.to_le_bytes()],
        bump = proposal.bump,
        has_one = dao
    )]
    pub proposal: Account<'info, Proposal>,
    #[account(
        mut,
        seeds = [b"treasury", dao.key().as_ref()],
        bump = dao.treasury_bump
    )]
    pub treasury: SystemAccount<'info>,
    /// CHECK: The proposal stores and validates the exact recipient public key.
    #[account(mut)]
    pub recipient: UncheckedAccount<'info>,
    pub system_program: Program<'info, System>,
}

#[account]
#[derive(InitSpace)]
pub struct Dao {
    pub creator: Pubkey,
    pub governance_mint: Pubkey,
    pub nonce: u64,
    #[max_len(32)]
    pub name: String,
    pub quorum: u64,
    pub next_proposal_id: u64,
    pub treasury_bump: u8,
    pub bump: u8,
}

#[account]
#[derive(InitSpace)]
pub struct Proposal {
    pub dao: Pubkey,
    pub proposer: Pubkey,
    pub id: u64,
    #[max_len(64)]
    pub title: String,
    #[max_len(256)]
    pub description: String,
    pub recipient: Pubkey,
    pub amount_lamports: u64,
    pub yes_votes: u64,
    pub no_votes: u64,
    pub abstain_votes: u64,
    pub quorum: u64,
    pub closes_at: i64,
    pub executed: bool,
    pub bump: u8,
}

#[account]
#[derive(InitSpace)]
pub struct VoteRecord {
    pub proposal: Pubkey,
    pub voter: Pubkey,
    pub weight: u64,
    pub choice: u8,
    pub bump: u8,
}

#[error_code]
pub enum GovernanceError {
    #[msg("The DAO name exceeds 32 bytes.")]
    NameTooLong,
    #[msg("The quorum must be greater than zero.")]
    InvalidQuorum,
    #[msg("The proposal title exceeds 64 bytes.")]
    TitleTooLong,
    #[msg("The proposal description exceeds 256 bytes.")]
    DescriptionTooLong,
    #[msg("The transfer amount must be greater than zero.")]
    InvalidAmount,
    #[msg("The voting period must be between one second and 90 days.")]
    InvalidVotingPeriod,
    #[msg("The proposal voting period has ended.")]
    VotingClosed,
    #[msg("The voter token account has no governance tokens.")]
    NoVotingPower,
    #[msg("Use 0 for yes, 1 for no, or 2 for abstain.")]
    InvalidVoteChoice,
    #[msg("The voting period has not ended yet.")]
    VotingStillOpen,
    #[msg("This proposal has already executed.")]
    AlreadyExecuted,
    #[msg("The execution recipient does not match the approved proposal.")]
    WrongRecipient,
    #[msg("The proposal did not reach quorum.")]
    QuorumNotMet,
    #[msg("The proposal did not receive more yes votes than no votes.")]
    ProposalRejected,
    #[msg("The treasury does not hold enough lamports.")]
    InsufficientTreasury,
    #[msg("A checked arithmetic operation overflowed.")]
    MathOverflow,
}

#[cfg(test)]
mod tests {
    use super::{proposal_passes, GovernanceError};

    #[test]
    fn proposal_requires_quorum_and_yes_majority() {
        assert!(proposal_passes(6, 2, 2, 10).unwrap());
        assert!(!proposal_passes(5, 5, 0, 10).unwrap());
        assert!(!proposal_passes(4, 5, 1, 10).unwrap());
        assert_eq!(
            proposal_passes(5, 3, 1, 10).unwrap_err(),
            GovernanceError::QuorumNotMet.into()
        );
    }

    #[test]
    fn participation_overflow_is_rejected() {
        assert!(proposal_passes(u64::MAX, 1, 0, 1).is_err());
    }
}
