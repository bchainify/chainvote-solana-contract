use solana_program::{
    account_info::{AccountInfo, next_account_info},
    entrypoint::ProgramResult,
    pubkey::Pubkey,
    program_error::ProgramError,
    decode_error::DecodeError,
    program_pack::{Pack},
    msg,
};
use num_derive::FromPrimitive;
use thiserror::Error;
use std::mem::size_of;
use std::str::from_utf8;
use crate::{state::{Vote, Voter}};



pub struct Processor {}

impl Processor {
    pub fn process(program_id: &Pubkey, accounts: &[AccountInfo], input: &[u8]) -> ProgramResult {
        let _account = accounts;
        let _program_id = program_id;
        let instruction = VoteInstruction::unpack(input)?; 
        match instruction {
            VoteInstruction::NewVote => {
                msg!("Instruction NewVote");
                Self::process_newvote(program_id, accounts)
            }
            VoteInstruction::AddUser => {
                msg!("Instruction AddUser");
                Self::process_adduser(program_id, accounts)
            }
            VoteInstruction::Vote {is_vote_for} => {
                msg!("Instruction Vote");
                Self::process_vote(program_id, accounts, &is_vote_for)
            }
        }
    }

    pub fn process_newvote(program_id: &Pubkey, accounts: &[AccountInfo]) -> ProgramResult {
        msg!("process new vote+++");
        let _account = accounts;
        let _program_id = program_id;
        let accounts_iter = &mut accounts.iter();
        let vote_data_account = next_account_info(accounts_iter)?;

        if vote_data_account.owner != program_id {
            msg!("Vote data account is not owned by program");
            return Err(ProgramError::InvalidAccountData);
        }

        let mut vote_data = vote_data_account.try_borrow_mut_data()?;

        let mut vote = Vote::unpack_unchecked(&vote_data)?;

        if vote.is_initialized {
            msg!("Vote has already been initialized");
            return Err(VoteError::VoteDataAccountAlreadyInitialized.into());
        }

        vote.is_initialized = true;

        msg!("Creating Vote with title {:?}", from_utf8(&vote.title).unwrap());
        msg!("New Vote: {:?}", vote);
        Vote::pack(vote, &mut vote_data).expect("Failed to write to vote data account");
        msg!("Vote created");

        Ok(())
    }
    
    pub fn process_adduser(program_id: &Pubkey, accounts: &[AccountInfo]) -> ProgramResult {
        msg!("process new user+++");

        let accounts_iter = &mut accounts.iter();

        let vote_data_account = next_account_info(accounts_iter)?; // 1

        if vote_data_account.owner != program_id {
            msg!("Vote data account is not owned by program");
            return Err(ProgramError::InvalidAccountData);
        }
        // msg!("voter_data {:?}", vote_data_account.clone());

        msg!("Vote account data, {:?}", vote_data_account);

        let vote_data = vote_data_account.try_borrow_mut_data()?;

        let vote = Vote::unpack_unchecked(&vote_data)?;

        if !vote.is_initialized {
            msg!("Vote has has not been initialized");
            return Err(ProgramError::InvalidAccountData);
        }


        // check that voter adder is the owner/creator of the vote
        let vote_creator_account = next_account_info(accounts_iter)?; // 2

        if !vote_creator_account.is_signer {
            msg!("Voter account is not signer");
            return Err(ProgramError::MissingRequiredSignature);
        }
        
        let voter_account = next_account_info(accounts_iter)?; // 3
        
        let voter_data_account = next_account_info(accounts_iter)?; //4

        let seed: &str = &(vote_data_account.key.to_string())[30..];

        let expected_voter_data_account = Pubkey::create_with_seed(voter_account.key, seed, program_id)?;
        if expected_voter_data_account != *voter_data_account.key {
            msg!("Voter data account [{:?}] is not valid, not equal to expected [{:?}]", *voter_data_account.key, expected_voter_data_account);
            return Err(ProgramError::InvalidAccountData); 
        }

        

        let mut voter_data_account_data = voter_data_account.try_borrow_mut_data()?;

        let mut voter = Voter::unpack_unchecked(&voter_data_account_data)?;

        if voter.is_initialized {
            msg!("Voter is already added");
            return Err(ProgramError::InvalidInstructionData); 
        }

        voter.is_initialized = true;
        // voter.has_voted = false;

        Voter::pack(voter, &mut voter_data_account_data).expect("Failed to write to voter's data account");
        
        Ok(())
    }

    pub fn process_vote(program_id: &Pubkey, accounts: &[AccountInfo], is_vote_for: &bool) -> ProgramResult {

        let accounts_iter = &mut accounts.iter();

        let vote_data_account = next_account_info(accounts_iter)?; // 1
        
        if vote_data_account.owner != program_id {
            msg!("Vote data account is not owned by program");
            return Err(ProgramError::InvalidAccountData);
        }
        let mut vote_data = vote_data_account.try_borrow_mut_data()?;
        let mut vote = Vote::unpack_unchecked(&vote_data)?;

        let voter_account = next_account_info(accounts_iter)?; // 2
        if !voter_account.is_signer {
            msg!("Voter account is not signer");
            return Err(ProgramError::MissingRequiredSignature);
        }

        let voter_data_account = next_account_info(accounts_iter)?; //3

        let seed: &str = &(vote_data_account.key.to_string())[30..];

        let expected_voter_data_account = Pubkey::create_with_seed(voter_account.key, seed, program_id)?;
        if expected_voter_data_account != *voter_data_account.key {
            msg!("Voter data account is not valid");
            return Err(ProgramError::InvalidAccountData); 
        }

        let mut voter_data_account_data = voter_data_account.try_borrow_mut_data()?;

        let mut voter = Voter::unpack_unchecked(&voter_data_account_data)?;

        if !voter.is_initialized || voter.has_voted {
            msg!("Invalid voter: {} or voter has voted: {}", voter.is_initialized, voter.has_voted);
            return Err(ProgramError::InvalidInstructionData); 
        }
        if *is_vote_for {
            vote.yes += 1;
        }else {
            vote.no += 1;
        }
        voter.has_voted = true;
        
        Vote::pack(vote, &mut vote_data).expect("Failed to write to vote data account");
        Voter::pack(voter, &mut voter_data_account_data).expect("Failed to write to voter's data account");

        Ok(())
    }
}

#[repr(C)]
#[derive(Clone, Debug, PartialEq)]
pub enum VoteInstruction {
    NewVote,
    AddUser,
    Vote {is_vote_for: bool},
}

impl VoteInstruction {
    pub fn unpack(input: &[u8]) -> Result<Self, ProgramError> {
        let (&tag, rest) = input.split_first().ok_or(VoteError::InvalidInstruction)?;
        Ok(match tag {
            0 => Self::NewVote,
            1 => Self::AddUser,
            2 => {
                let (&is_vote_for, _rest) = rest.split_first().ok_or(VoteError::InvalidInstruction)?;
                let is_vote_for = match is_vote_for {
                    0 => false,
                    1 => true,
                    _ => false,
                };
                Self::Vote {is_vote_for}
            },
            _ => return Err(VoteError::InvalidInstruction.into()),
        })
    }
    pub fn pack(&self) -> Vec<u8> {
        let mut buf = Vec::with_capacity(size_of::<Self>());
        match self {
            Self::NewVote => buf.push(0),
            Self::AddUser => buf.push(1),
            Self::Vote{is_vote_for} => {
                buf.push(3);
                let is_vote_for = *is_vote_for as u8;
                buf.push(is_vote_for);
            },
        };
        buf
    }
}

#[derive(Clone, Debug, Eq, Error, FromPrimitive, PartialEq)]
pub enum VoteError {
    /// Invalid instruction
    #[error("Invalid instruction")]
    InvalidInstruction,
    
    /// VoteDataAccountAlreadyInitialized
    #[error("Vote data account has already been initialized")]
    VoteDataAccountAlreadyInitialized,
}

impl From<VoteError> for ProgramError {
    fn from(e: VoteError) -> Self {
        ProgramError::Custom(e as u32)
    }
}
impl<T> DecodeError<T> for VoteError {
    fn type_of() -> &'static str {
        "VoteError"
    }
}
