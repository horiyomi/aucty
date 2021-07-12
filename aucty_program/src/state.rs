use solana_program::{
  program_pack::{ IsInitialized, Sealed, Pack},
  pubkey::Pubkey,
  program_error::ProgramError,
};
use arrayref::{array_mut_ref, array_ref, array_refs, mut_array_refs};


pub struct Auction {
  pub is_initialized: bool,
  pub initializer_pubkey: Pubkey,
  pub temp_token_account_pubkey: Pubkey,
  pub initializer_token_to_receive_account_pubkey: Pubkey,
  pub bid_amount: u64,

}



impl Sealed for Auction {}

impl IsInitialized for Auction {

  fn is_initialized(&self) -> bool {
    self.is_initialized
  }
}


impl Pack for Auction {
  const LEN: usize = 105;

  fn unpack_from_slice(src: &[u8]) -> Result<Self, ProgramError> {
    let src = array_ref![src, 0, Auction::LEN];
    let (
        is_initialized,
        initializer_pubkey,
        temp_token_account_pubkey,
        initializer_token_to_receive_account_pubkey,
        bid_amount,
      ) = array_refs![src, 1, 32, 32, 32, 8];
      let is_initialized = match is_initialized {
          [0] => false,
          [1] => true,
          _ => return Err(ProgramError::InvalidAccountData),
      };

      Ok(Auction {
          is_initialized,
          initializer_pubkey: Pubkey::new_from_array(*initializer_pubkey),
          temp_token_account_pubkey: Pubkey::new_from_array(*temp_token_account_pubkey),
          initializer_token_to_receive_account_pubkey: Pubkey::new_from_array(*initializer_token_to_receive_account_pubkey),
          bid_amount: u64::from_le_bytes(*bid_amount),
      })
  }

  fn pack_into_slice(&self, dst: &mut [u8]) {
    let dst = array_mut_ref![dst, 0, Auction::LEN];
    let (
        is_initialized_dst,
        initializer_pubkey_dst,
        temp_token_account_pubkey_dst,
        initializer_token_to_receive_account_pubkey_dst,
        bid_amount_dest,
    ) = mut_array_refs![dst, 1, 32, 32, 32, 8];

    let Auction {
      is_initialized,
      initializer_pubkey,
      temp_token_account_pubkey,
      initializer_token_to_receive_account_pubkey,
      bid_amount,
    } = self;

    is_initialized_dst[0] = *is_initialized as u8;
    initializer_pubkey_dst.copy_from_slice(initializer_pubkey.as_ref());
    temp_token_account_pubkey_dst.copy_from_slice(temp_token_account_pubkey.as_ref());
    initializer_token_to_receive_account_pubkey_dst.copy_from_slice(initializer_token_to_receive_account_pubkey.as_ref());
    *bid_amount_dest = bid_amount.to_le_bytes();
  }

}