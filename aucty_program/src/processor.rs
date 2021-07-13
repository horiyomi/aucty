use solana_program::{
  account_info::{ next_account_info, AccountInfo},
  entrypoint::ProgramResult,
  msg,
  pubkey::Pubkey,
  program_pack::{Pack, IsInitialized},
  sysvar::{rent::Rent, Sysvar},
  program::{invoke, invoke_signed},
  program_error::ProgramError
};
use spl_token::state::Account as TokenAccount;
use crate::{instruction::AuctionInstruction, error::AuctionError, state::Auction};

pub struct Processor;


impl Processor {
  pub fn process(program_id: &Pubkey, accounts: &[AccountInfo], instruction_data: &[u8]) -> ProgramResult {
    let instruction = AuctionInstruction::unpack(instruction_data)?;
    match instruction {
      AuctionInstruction::InitAuction { amount } => {
        msg!("Instruction: InitAuction");
        Self::process_init_auction(accounts, amount, program_id)
      }
      AuctionInstruction::Bid { amount } => {
        msg!("Instruction: Bid");
        Self::process_place_bid(accounts, amount, program_id)
      }
      AuctionInstruction::BidWinner { amount } => {
        msg!("Instruction: Bid Winner");
        Self::process_bid_winner(accounts, amount, program_id)
      }
    }
  }


  /// Initiate auction, by creating a temporary account and 
  /// assigning owner authority to the tempory account which bidders 
  /// transfer spl tokens to.
  fn process_init_auction(accounts: &[AccountInfo], amount: u64, program_id: &Pubkey) -> ProgramResult {
    let account_info_iter = &mut accounts.iter();
    let initializer = next_account_info(account_info_iter)?;

    if !initializer.is_signer {
      return Err(ProgramError::MissingRequiredSignature);
    }

    let temp_token_account = next_account_info(account_info_iter)?;

    let token_to_recieve_account = next_account_info(account_info_iter)?;
    if *token_to_recieve_account.owner != spl_token::id() {
      return Err(ProgramError::IncorrectProgramId);
    }


    let auction_account = next_account_info(account_info_iter)?;
    let rent = &Rent::from_account_info(next_account_info(account_info_iter)?)?;

    if !rent.is_exempt(auction_account.lamports(), auction_account.data_len()) {
      return Err(AuctionError::NotRentExempt.into());
    }

    let mut auction_info = Auction::unpack_unchecked(&auction_account.data.borrow())?;

    if auction_info.is_initialized() {
      return Err(ProgramError::AccountAlreadyInitialized);
    }

    auction_info.is_initialized = true;
    auction_info.initializer_pubkey = *initializer.key;
    auction_info.temp_token_account_pubkey = *temp_token_account.key;
    auction_info.initializer_token_to_receive_account_pubkey = *token_to_recieve_account.key;
    auction_info.bid_amount = amount;

    Auction::pack(auction_info, &mut auction_account.data.borrow_mut())?;    
    
    let (pda, _bump_seed) = Pubkey::find_program_address(&[b"auction"], program_id);

    let token_program = next_account_info(account_info_iter)?;
    let owner_change_ix = spl_token::instruction::set_authority(
        token_program.key,
        temp_token_account.key,
        Some(&pda),
        spl_token::instruction::AuthorityType::AccountOwner,
        initializer.key,
        &[&initializer.key],
    )?;

    msg!("Calling the token program to transfer token account ownership...");
    invoke(
      &owner_change_ix,
      &[
        temp_token_account.clone(),
        initializer.clone(),
        token_program.clone(),
      ],
    )?;


    Ok(())
  }


  // Place bid to the auction temporary account
  fn process_place_bid(accounts: &[AccountInfo], amount_expected_by_bidder: u64, _program_id: &Pubkey) -> ProgramResult {
    let account_info_iter = &mut accounts.iter();
    let bidder = next_account_info(account_info_iter)?;

    if !bidder.is_signer {
      return Err(ProgramError::MissingRequiredSignature);
    }

    let bidder_sending_token_account = next_account_info(account_info_iter)?;

    // let bidder_token_to_receive_account = next_account_info(account_info_iter)?;
    
    let pdas_temp_token_account = next_account_info(account_info_iter)?;
    let pdas_temp_token_account_info = TokenAccount::unpack(&pdas_temp_token_account.data.borrow())?;
    // let (pda, bump_seed) = Pubkey::find_program_address(&[b"auction"], program_id);

    if amount_expected_by_bidder < pdas_temp_token_account_info.amount {
      return Err(AuctionError::BelowExpectedAmount.into());
    }

    let initializers_main_account = next_account_info(account_info_iter)?;
    let initializers_token_to_receive_account = next_account_info(account_info_iter)?;
    let auction_account = next_account_info(account_info_iter)?;

    let auction_info = Auction::unpack(&auction_account.data.borrow())?;

    if auction_info.temp_token_account_pubkey != *pdas_temp_token_account.key {
      return Err(ProgramError::InvalidAccountData);
    }

    if auction_info.initializer_pubkey != *initializers_main_account.key {
      return Err(ProgramError::InvalidAccountData);
    }

    if auction_info.initializer_token_to_receive_account_pubkey != *initializers_token_to_receive_account.key {
      return Err(ProgramError::InvalidAccountData);
    }

    let token_program = next_account_info(account_info_iter)?;

    let transfer_to_initializer_ix = spl_token::instruction::transfer(
      token_program.key,
      bidder_sending_token_account.key,
      initializers_token_to_receive_account.key,
      bidder.key,
      &[&bidder.key],
      auction_info.bid_amount,
    )?;

    msg!("Calling the token program to transfer tokens to the auction's initializer...");
    invoke(
      &transfer_to_initializer_ix,
      &[
        bidder_sending_token_account.clone(),
        initializers_token_to_receive_account.clone(),
        bidder.clone(),
        token_program.clone(),
      ],
    )?;

    Ok(())
  }

  /// bid winner takes value from auction temporary account 
  fn process_bid_winner(accounts: &[AccountInfo], amount_expected_by_bidder: u64, program_id: &Pubkey) -> ProgramResult {
    let account_info_iter = &mut accounts.iter();
    let bidder = next_account_info(account_info_iter)?;

    if !bidder.is_signer {
      return Err(ProgramError::MissingRequiredSignature);
    }

    let bidder_sending_token_account = next_account_info(account_info_iter)?;

    let bidder_token_to_receive_account = next_account_info(account_info_iter)?;
    
    let pdas_temp_token_account = next_account_info(account_info_iter)?;
    let pdas_temp_token_account_info = TokenAccount::unpack(&pdas_temp_token_account.data.borrow())?;
    let (pda, bump_seed) = Pubkey::find_program_address(&[b"auction"], program_id);

    if amount_expected_by_bidder != pdas_temp_token_account_info.amount {
      return Err(AuctionError::ExpectedAmountMismatch.into());
    }

    let initializers_main_account = next_account_info(account_info_iter)?;
    let initializers_token_to_receive_account = next_account_info(account_info_iter)?;
    let auction_account = next_account_info(account_info_iter)?;

    let auction_info = Auction::unpack(&auction_account.data.borrow())?;

    if auction_info.temp_token_account_pubkey != *pdas_temp_token_account.key {
      return Err(ProgramError::InvalidAccountData);
    }

    if auction_info.initializer_pubkey != *initializers_main_account.key {
      return Err(ProgramError::InvalidAccountData);
    }

    if auction_info.initializer_token_to_receive_account_pubkey != *initializers_token_to_receive_account.key {
      return Err(ProgramError::InvalidAccountData);
    }

    let token_program = next_account_info(account_info_iter)?;

    let transfer_to_initializer_ix = spl_token::instruction::transfer(
      token_program.key,
      bidder_sending_token_account.key,
      initializers_token_to_receive_account.key,
      bidder.key,
      &[&bidder.key],
      auction_info.bid_amount,
    )?;

    msg!("Calling the token program to transfer tokens to the auction's initializer...");

    invoke(
      &transfer_to_initializer_ix,
      &[
        bidder_sending_token_account.clone(),
        initializers_token_to_receive_account.clone(),
        bidder.clone(),
        token_program.clone(),
      ],
    )?;

    let pda_account = next_account_info(account_info_iter)?;

    let transfer_to_taker_ix = spl_token::instruction::transfer(
      token_program.key,
      pdas_temp_token_account.key,
      bidder_token_to_receive_account.key,
      &pda,
      &[&pda],
      pdas_temp_token_account_info.amount,
    )?;

    msg!("Calling the token program to transfer tokens to the highest bidder...");
    invoke_signed(
        &transfer_to_taker_ix,
        &[
            pdas_temp_token_account.clone(),
            bidder_token_to_receive_account.clone(),
            pda_account.clone(),
            token_program.clone(),
        ],
        &[&[&b"auction"[..], &[bump_seed]]],
    )?;

    let close_pdas_temp_acc_ix = spl_token::instruction::close_account(
      token_program.key,
      pdas_temp_token_account.key,
      initializers_main_account.key,
      &pda,
      &[&pda]
    )?;
    msg!("Calling the token program to close pda's temp account...");
    invoke_signed(
      &close_pdas_temp_acc_ix,
      &[
        pdas_temp_token_account.clone(),
        initializers_main_account.clone(),
        pda_account.clone(),
        token_program.clone(),
      ],
      &[&[&b"auction"[..], &[bump_seed]]],
    )?;

    msg!("Closing the auction account...");
    **initializers_main_account.lamports.borrow_mut() = initializers_main_account.lamports()
      .checked_add(auction_account.lamports())
      .ok_or(AuctionError::AmountOverflow)?;
      **auction_account.lamports.borrow_mut() = 0;
      *auction_account.data.borrow_mut() = &mut [];

      Ok(())
  }


}