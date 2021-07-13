use thiserror::Error;
use solana_program::program_error::ProgramError;

#[derive(Error, Debug, Copy, Clone)]
pub enum AuctionError {

  /// Invalid instruction error
  #[error("Invalid Instruction")]
  InvalidInstruction,
  
  #[error("Not rent exempt")]
  NotRentExempt,

  #[error("Amount expected mismatch")]
  ExpectedAmountMismatch,

  #[error("Amount specified below highest bid")]
  BelowExpectedAmount,

  #[error("Amount overflow")]
  AmountOverflow,
}

impl From<AuctionError> for ProgramError {
  fn from(e: AuctionError) -> Self {
    ProgramError::Custom(e as u32)
  }
}