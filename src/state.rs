use solana_program::{
    program_pack::{Pack, Sealed, IsInitialized},
    program_error::ProgramError,
    // pubkey::Pubkey,

};
use byteorder::{ByteOrder, LittleEndian};
use arrayref::{array_ref, array_mut_ref, mut_array_refs, array_refs};


pub const MAX_LEN: usize = 10;

pub struct VoteManager {
    // pub title: u8,
}
impl Sealed for VoteManager {}

impl Pack for VoteManager{
    const LEN: usize = 0;

    fn unpack_from_slice(src: &[u8]) -> Result<Self, ProgramError> {
        let _src = src;
        Ok(VoteManager{})
    }
    fn pack_into_slice(&self, dst: &mut [u8]) {
        let _dst = dst;
    }
}

#[repr(C)]
#[derive(Clone, Copy, Debug, Default, PartialEq)]
pub struct Vote {
    pub yes: u32, // 4
    pub no: u32, // 4
    pub is_initialized: bool,
    pub title: [u8; MAX_LEN], // 4*10
    // pub owner: Pubkey, // 32
    // pub end_time: u64
}

impl Sealed for Vote {}

impl IsInitialized for Vote {
    fn is_initialized(&self) -> bool {
        self.is_initialized
    }
}

impl Pack for Vote { 
    const LEN: usize = 19; // 19+32

    fn unpack_from_slice(src: &[u8]) -> Result<Self, ProgramError> {
        let yes = LittleEndian::read_u32(&src[0..4]);
        let no = LittleEndian::read_u32(&src[4..8]);
        let is_initialized = *array_ref![src, 8, 1];
        let is_initialized = match is_initialized {
            [0] => false,
            [1] => true,
            _ => return Err(ProgramError::InvalidAccountData),
        };
        let title = *array_ref![src, 9, 10];
        Ok(Vote{
            yes,
            no,
            is_initialized,
            title,
        })
    }
    fn pack_into_slice(&self, dst: &mut [u8]) {
        let dst = array_mut_ref![dst, 0, 19];
        let (
            yes_dst,
            no_dst,
            is_initialized_dst,
            title_dst,
        ) = mut_array_refs![dst, 4, 4, 1, 10];
        let &Vote {
            yes,
            no,
            is_initialized,
            title
        } = self;
        *yes_dst = yes.to_le_bytes();
        *no_dst = no.to_le_bytes();
        is_initialized_dst[0] = is_initialized as u8;
        *title_dst = title;
    }
}

#[repr(C)]
#[derive(Clone, Copy, Debug, Default, PartialEq)]
pub struct Voter {
    pub is_initialized: bool, // 1
    pub has_voted: bool, // 1
}

impl Sealed for Voter {}

impl IsInitialized for Voter {
    fn is_initialized(&self) -> bool {
        self.is_initialized
    }
}

impl Pack for Voter { 
    const LEN: usize = 2;

    fn unpack_from_slice(src: &[u8]) -> Result<Self, ProgramError> {
        let src = array_ref![src, 0, 2];
        let (is_initialized, has_voted) = array_refs![src, 1, 1];
        let is_initialized = match is_initialized {
            [0] => false,
            [1] => true,
            _ => return Err(ProgramError::InvalidAccountData),
        };
        let has_voted = match has_voted {
            [0] => false,
            [1] => true,
            _ => return Err(ProgramError::InvalidAccountData),
        };
        Ok(Voter{
            is_initialized,
            has_voted,
        })
    }
    fn pack_into_slice(&self, dst: &mut [u8]) {
        let dst = array_mut_ref![dst, 0, 2];
        let (
            is_initialized_dst,
            has_voted_dst,
        ) = mut_array_refs![dst, 1, 1];
        let &Voter {
            is_initialized,
            has_voted,
        } = self;
        is_initialized_dst[0] = is_initialized as u8;
        has_voted_dst[0] = has_voted as u8;
    }
}



/* 
    create vote : title, users, 
    create an agenda
    add users to the vote
    users should be able to vote
    vote should be announced

    Vote -> Agenda1,Agenda2, name, expTime, users[], voted[map]
    Agenda: title, users[], exp
*/
