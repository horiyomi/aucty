import { TOKEN_PROGRAM_ID } from "@solana/spl-token";
import { Account, Connection, PublicKey, Transaction, TransactionInstruction } from "@solana/web3.js";
import BN from "bn.js";
import { AUCTION_ACCOUNT_DATA_LAYOUT, AuctionLayout } from "./layout";

const connection = new Connection("http://localhost:8899", 'singleGossip');

export const takeBid = async (
  privateKeyByteArray: string,
  auctionAccountAddressString: string,
  takerXTokenAccountAddressString: string,
  takerYTokenAccountAddressString: string,
  takerExpectedXTokenAmount: number,
  programIdString: string,
) => {
  const takerAccount = new Account(privateKeyByteArray.split(',').map(s => parseInt(s)));
  const auctionAccountPubkey = new PublicKey(auctionAccountAddressString);
  const takerXTokenAccountPubkey = new PublicKey(takerXTokenAccountAddressString);
  const takerYTokenAccountPubkey = new PublicKey(takerYTokenAccountAddressString);
  const programId = new PublicKey(programIdString);

  let encodedAuctionState;

  try {
    encodedAuctionState = (await connection.getAccountInfo(auctionAccountPubkey, 'singleGossip'))!.data;
  } catch (err) {
    throw new Error("Could not find auction at given address!")
  }
  
  const decodedAuctionLayout = AUCTION_ACCOUNT_DATA_LAYOUT.decode(encodedAuctionState) as AuctionLayout;
  const auctionState = {
    auctionAccountPubkey: auctionAccountPubkey,
    isInitialized: !!decodedAuctionLayout.isInitialized,
    initializerAccountPubkey: new PublicKey(decodedAuctionLayout.initializerPubkey),
    XTokenTempAccountPubkey: new PublicKey(decodedAuctionLayout.initializerTempTokenAccountPubkey),
    initializerYTokenAccount: new PublicKey(decodedAuctionLayout.initializerReceivingTokenAccountPubkey),
    bidAmount: new BN(decodedAuctionLayout.bidAmount, 10, "le")
  };

  const PDA = await PublicKey.findProgramAddress([Buffer.from("auction")], programId);

  const auctionInstruction = new TransactionInstruction({
    programId,
    data: Buffer.from(Uint8Array.of(1, ...new BN(takerExpectedXTokenAmount).toArray("le", 8))),
    keys: [
      { pubkey: takerAccount.publicKey, isSigner: true, isWritable: false },
      { pubkey: takerYTokenAccountPubkey, isSigner: false, isWritable: true },
      { pubkey: takerXTokenAccountPubkey, isSigner: false, isWritable: true },
      { pubkey: auctionState.XTokenTempAccountPubkey, isSigner: false, isWritable: true },
      { pubkey: auctionState.initializerAccountPubkey, isSigner: false, isWritable: true },
      { pubkey: auctionState.initializerYTokenAccount, isSigner: false, isWritable: true },
      { pubkey: auctionAccountPubkey, isSigner: false, isWritable: true },
      { pubkey: TOKEN_PROGRAM_ID, isSigner: false, isWritable: false },
      { pubkey: PDA[0], isSigner: false, isWritable: false }
    ]
  })

  await connection.sendTransaction(new Transaction().add(auctionInstruction), [takerAccount], { skipPreflight: false, preflightCommitment: 'singleGossip' });
}