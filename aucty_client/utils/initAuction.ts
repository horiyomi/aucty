import { AccountLayout, Token, TOKEN_PROGRAM_ID } from "@solana/spl-token";
import { Account, Connection, PublicKey, SystemProgram, 
    SYSVAR_RENT_PUBKEY, Transaction, TransactionInstruction } from "@solana/web3.js";
import BN from "bn.js";
import { AUCTION_ACCOUNT_DATA_LAYOUT, AuctionLayout } from "./layout";

const connection = new Connection("http://localhost:8899", 'singleGossip');

export const initAuction = async (
    privateKeyByteArray: string,
    initializerXTokenAccountPubkeyString: string,
    amountXTokensToSendToEscrow: number,
    initializerReceivingTokenAccountPubkeyString: string,
    bidAmount: number,
    auctionProgramIdString: string) => {
    const initializerXTokenAccountPubkey = new PublicKey(initializerXTokenAccountPubkeyString);

    //@ts-expect-error
    const XTokenMintAccountPubkey = new PublicKey((await connection.getParsedAccountInfo(initializerXTokenAccountPubkey, 'singleGossip')).value!.data.parsed.info.mint);

    const privateKeyDecoded = privateKeyByteArray.split(',').map(s => parseInt(s));
    const initializerAccount = new Account(privateKeyDecoded);

    const tempTokenAccount = new Account();
    const createTempTokenAccountIx = SystemProgram.createAccount({
        programId: TOKEN_PROGRAM_ID,
        space: AccountLayout.span,
        lamports: await connection.getMinimumBalanceForRentExemption(AccountLayout.span, 'singleGossip'),
        fromPubkey: initializerAccount.publicKey,
        newAccountPubkey: tempTokenAccount.publicKey
    });

    const initTempAccountIx = Token.createInitAccountInstruction(TOKEN_PROGRAM_ID, XTokenMintAccountPubkey, 
        tempTokenAccount.publicKey, initializerAccount.publicKey);

    const transferXTokensToTempAccIx = Token
        .createTransferInstruction(TOKEN_PROGRAM_ID, initializerXTokenAccountPubkey, 
            tempTokenAccount.publicKey, initializerAccount.publicKey, [], amountXTokensToSendToEscrow);
    
    const auctionAccount = new Account();
    const auctionProgramId = new PublicKey(auctionProgramIdString);

    const createAcutionAccountIx = SystemProgram.createAccount({
        space: AUCTION_ACCOUNT_DATA_LAYOUT.span,
        lamports: await connection.getMinimumBalanceForRentExemption(AUCTION_ACCOUNT_DATA_LAYOUT.span, 'singleGossip'),
        fromPubkey: initializerAccount.publicKey,
        newAccountPubkey: auctionAccount.publicKey,
        programId: auctionProgramId
    });

    const initAuctionIx = new TransactionInstruction({
        programId: auctionProgramId,
        keys: [
            { pubkey: initializerAccount.publicKey, isSigner: true, isWritable: false },
            { pubkey: tempTokenAccount.publicKey, isSigner: false, isWritable: true },
            { pubkey: new PublicKey(initializerReceivingTokenAccountPubkeyString), isSigner: false, isWritable: false },
            { pubkey: auctionAccount.publicKey, isSigner: false, isWritable: true },
            { pubkey: SYSVAR_RENT_PUBKEY, isSigner: false, isWritable: false},
            { pubkey: TOKEN_PROGRAM_ID, isSigner: false, isWritable: false },
        ],
        data: Buffer.from(Uint8Array.of(0, ...new BN(bidAmount).toArray("le", 8)))
    })

    const tx = new Transaction()
        .add(createTempTokenAccountIx, initTempAccountIx, transferXTokensToTempAccIx, createAcutionAccountIx, initAuctionIx);
    await connection.sendTransaction(tx, [initializerAccount, tempTokenAccount, auctionAccount], {skipPreflight: false, preflightCommitment: 'singleGossip'});

    await new Promise((resolve) => setTimeout(resolve, 1000));

    const encodedAuctionState = (await connection.getAccountInfo(auctionAccount.publicKey, 'singleGossip'))!.data;
    const decodedAuctionState = AUCTION_ACCOUNT_DATA_LAYOUT.decode(encodedAuctionState) as AuctionLayout;
    return {
        auctionAccountPubkey: auctionAccount.publicKey.toBase58(),
        isInitialized: !!decodedAuctionState.isInitialized,
        initializerAccountPubkey: new PublicKey(decodedAuctionState.initializerPubkey).toBase58(),
        XTokenTempAccountPubkey: new PublicKey(decodedAuctionState.initializerTempTokenAccountPubkey).toBase58(),
        initializerYTokenAccount: new PublicKey(decodedAuctionState.initializerReceivingTokenAccountPubkey).toBase58(),
        bidAmount: new BN(decodedAuctionState.bidAmount, 10, "le").toNumber()
    };
}