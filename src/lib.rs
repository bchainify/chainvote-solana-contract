pub mod processor;
pub mod state;

use solana_program::{
    account_info::{AccountInfo},
    entrypoint,
    entrypoint::ProgramResult,
    msg,
    pubkey::Pubkey,
    // program_error::PrintProgramError
};

use crate::{processor::Processor};

entrypoint!(process_instruction);

fn process_instruction(
    program_id: &Pubkey,     
    accounts: &[AccountInfo],
    instruction_data: &[u8], 
) -> ProgramResult {
    if let Err(err) = Processor::process(program_id, accounts, instruction_data){
        msg!("Error occured: {:?}", err);
        // err.print::<VoteError>();
        return Err(err)
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    // use std::mem;
    use solana_program::{
            clock::Epoch,
            program_pack::{Pack},
        };
        use crate::{state::Vote};

    #[test]
    fn test_sanity() {
        let program_id = Pubkey::default();
        let key = Pubkey::new_unique();
        // msg!("pub key {}", key);
        let mut lamports = 0;
        // let mut data = vec![0; 2 * mem::size_of::<u32>()];
        let mut data = vec![0; Vote::get_packed_len()];
        let title = b"absfheksbg";
        let vote = Vote{
            yes: 0, no: 1, is_initialized: true, title: *title,
        };
        Vote::pack(vote, &mut data).unwrap();
        // msg!("Packed vote {:?}", vote);
        let owner = Pubkey::default();
        let instruction_data: Vec<u8> = vec![1];
        let vote_data_account = AccountInfo::new(
            &key,             // account pubkey
            false,            // is_signer
            true,             // is_writable
            &mut lamports,    // balance in lamports
            &mut data,        // storage
            &owner,           // owner pubkey
            false,            // is_executable
            Epoch::default(), // rent_epoch
        );
        let mut account_vote_owner = vote_data_account.clone();
        let voter_data_account = vote_data_account.clone();
        let voter_account = vote_data_account.clone();
        let acct = vote_data_account.clone();
        account_vote_owner.is_signer = true;
        account_vote_owner.is_writable = false;
        let accounts = vec![vote_data_account, account_vote_owner, voter_account, voter_data_account, acct];

        let result = process_instruction(&program_id, &accounts, &instruction_data).unwrap_or_else(
            |err|{
                msg!("Program error: {:?}", err);
            } 
        );
        msg!("Result: {:?}", result);
    }

    #[test]
    fn test_sanity2() {
        let program_id = Pubkey::default();
        let key = Pubkey::new_unique();
        let mut lamports = 0;
        let mut data = vec![0; Vote::get_packed_len()];
        let title = b"absfheksbg";
        let vote = Vote{
            yes: 11, no: 2, is_initialized: false, title: *title,
        };
        Vote::pack(vote, &mut data).unwrap();

        let owner = Pubkey::default();
        let instruction_data: Vec<u8> = vec![0];
        let account = AccountInfo::new(
            &key,             // account pubkey
            false,            // is_signer
            true,             // is_writable
            &mut lamports,    // balance in lamports
            &mut data,        // storage
            &owner,           // owner pubkey
            false,            // is_executable
            Epoch::default(), // rent_epoch
        );
        let accounts = vec![account];

        let _result = process_instruction(&program_id, &accounts, &instruction_data).unwrap_or_else(
            |err|{
                msg!("Program error: {:?}", err);
            } 
        );
        
        let result = process_instruction(&program_id, &accounts, &instruction_data).unwrap_or_else(
            |err|{
                msg!("Program error: {:?}", err);
            } 
        );
        msg!("Result: {:?}", result);
    }
}
