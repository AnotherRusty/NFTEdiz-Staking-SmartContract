import { Program, web3 } from '@project-serum/anchor';
import * as anchor from '@project-serum/anchor';
import {
    PublicKey,
    SystemProgram,
    SYSVAR_RENT_PUBKEY,
} from '@solana/web3.js';
import {
    TOKEN_PROGRAM_ID,
    ASSOCIATED_TOKEN_PROGRAM_ID,
} from "@solana/spl-token";
import fs from 'fs';

export const METAPLEX = new web3.PublicKey('metaqbxxUerdq28cj1RbAWkYQm3ybzjb6a8bt518x1s');

//Type
import { GlobalPool, UserPool } from './type';

//Account Size
const USER_POOL_SIZE = 2712;     // 8 + 2704

//Seeds
const GLOBAL_AUTHORITY_SEED = "global-authority";
const USER_POOL_SEED = "user-pool";

//Collection
const BEAR_COLLECTION_ADDRESS = "Etw6Z82sU98kjHcDCyByJzBkRTjjTG5nNcJQ6JizQUkN";
const BOX_COLLECTION_ADDRESS = "DguaYzhpoH2Fxxr3zhRMpZpfcKRn5PLVXWMuTz3YGp9s";
const MEDAL_TOKEN_ADDRESS = new PublicKey("3BAfTyeyPkykQuC5g1FejbebcphhWTBgEwJ75XXBW6CW");

//Program ID
const PROGRAM_ID = "GqVfxjhCXWvhQtMg9x2K2BqhRDdC35MXxDjbLVdhaDv2";

// Address of the deployed program.
const programId = new anchor.web3.PublicKey(PROGRAM_ID);

let program: Program = null;
let provider: anchor.Provider = null;
let rewardVault: PublicKey = null;

//Connection & Provider
const idl = JSON.parse(
    fs.readFileSync(__dirname + "/../target/idl/armory_staking.json", "utf8")
);

anchor.setProvider(anchor.AnchorProvider.local(web3.clusterApiUrl("devnet")));
provider = anchor.getProvider();
const solConnection = anchor.getProvider().connection;
const payer = anchor.AnchorProvider.local().wallet;
const adminAddress = payer.publicKey;

// Generate the program client from IDL.
program = new anchor.Program(idl, programId);
console.log('ProgramId: ', program.programId.toBase58());

//Define Main function
const main = async () => {
    const [globalAuthority, bump] = await PublicKey.findProgramAddress(
        [Buffer.from(GLOBAL_AUTHORITY_SEED)],
        program.programId
    );
    console.log('GlobalAuthority: ', globalAuthority.toBase58());

    rewardVault = await getAssociatedTokenAccount(globalAuthority, MEDAL_TOKEN_ADDRESS);
    console.log('RewardVault: ', rewardVault.toBase58());

    /*
    functions to be executable
    */
    // await initGlobalPool();

    // await initUserPool(new PublicKey("Am9xhPPVCfDZFDabcGgmQ8GTMdsbqEt1qVXbyhTxybAp"));
    
    // await stakeNft(
    //     new PublicKey("Am9xhPPVCfDZFDabcGgmQ8GTMdsbqEt1qVXbyhTxybAp"),
    //     new PublicKey("3MYkAHwuy7JsCd96nqyAJMgij3qtqRXmDJ3pPycF4azQ"),
    //     new PublicKey("CCZcsGGdfskJWZoZ55h6BkwqC6GdxfYDuZeQxrSm31D9"),
    //     1
    // )

    // await unstakeNft(
    //     new PublicKey("Am9xhPPVCfDZFDabcGgmQ8GTMdsbqEt1qVXbyhTxybAp"),
    //     new PublicKey("3MYkAHwuy7JsCd96nqyAJMgij3qtqRXmDJ3pPycF4azQ"),
    //     new PublicKey("CCZcsGGdfskJWZoZ55h6BkwqC6GdxfYDuZeQxrSm31D9"),
    //     1
    // )

    // await claimReward(new PublicKey("Am9xhPPVCfDZFDabcGgmQ8GTMdsbqEt1qVXbyhTxybAp"));
}

export const initGlobalPool = async () => {
    const [globalAuthority, bump] = await PublicKey.findProgramAddress(
        [Buffer.from(GLOBAL_AUTHORITY_SEED)],
        program.programId
    );

    const tx = await program.rpc.initialize(
        bump, {
        accounts: {
            admin: adminAddress,
            globalAuthority,
            systemProgram: SystemProgram.programId,
            rent: SYSVAR_RENT_PUBKEY,
        },
        instructions: [],
        signers: [],
    });

    await solConnection.confirmTransaction(tx, "confirmed");
    console.log("txHash = ", tx);
}

//Initialize Userpool according to the user
export const initUserPool = async (user: PublicKey) => {
    let userPoolKey = await PublicKey.createWithSeed(
        user,
        USER_POOL_SEED,
        program.programId,
    );
    console.log('Your Address: ', user.toBase58());
    console.log("User Pool Address:", userPoolKey.toBase58());

    let ix = SystemProgram.createAccountWithSeed({
        fromPubkey: user,
        basePubkey: user,
        seed: USER_POOL_SEED,
        newAccountPubkey: userPoolKey,
        lamports: await solConnection.getMinimumBalanceForRentExemption(USER_POOL_SIZE),
        space: USER_POOL_SIZE,
        programId: program.programId,
    });

    const tx = await program.rpc.initUserPool(
        {
            accounts: {
                userPool: userPoolKey,
                owner: user
            },
            instructions: [
                ix
            ],
            signers: []
        }
    );
    await solConnection.confirmTransaction(tx, "finalized");

    console.log("txHash = ", tx);
}

export const stakeNft = async (userAddress: PublicKey, mint: PublicKey, boxMint: PublicKey, boxId: number) => {
    const [globalAuthority, bump] = await PublicKey.findProgramAddress(
        [Buffer.from(GLOBAL_AUTHORITY_SEED)],
        program.programId
    );

    let userTokenAccount = await getAssociatedTokenAccount(userAddress, mint);
    let destIx1 = await getATokenAccountsNeedCreate(
        solConnection,
        userAddress,
        globalAuthority,
        [mint]
    );
    console.log("Dest Bear Account = ", destIx1.destinationAccounts[0].toBase58());

    let userBoxAccount = await getAssociatedTokenAccount(userAddress, boxMint);
    let destIx = await getATokenAccountsNeedCreate(
        solConnection,
        userAddress,
        globalAuthority,
        [boxMint]
    );
    console.log("Dest Box Account = ", destIx.destinationAccounts[0].toBase58());

    let { instructions, destinationAccounts } = await getATokenAccountsNeedCreate(
        solConnection,
        userAddress,
        userAddress,
        [MEDAL_TOKEN_ADDRESS]
    );
    console.log("Dest Token Account = ", destinationAccounts[0].toBase58());

    let userPoolKey = await PublicKey.createWithSeed(
        userAddress,
        USER_POOL_SEED,
        program.programId,
    );

    let poolAccount = await solConnection.getAccountInfo(userPoolKey);
    if (poolAccount === null || poolAccount.data === null) {
        await initUserPool(userAddress);
    }

    const metadata = await getMetadata(mint);
    console.log("Metadata=", metadata.toBase58());

    const tx = await program.rpc.stakeNft(
        bump, new anchor.BN(boxId), {
        accounts: {
            owner: userAddress,
            globalAuthority,
            userPool: userPoolKey,
            nftMint: mint,
            nftBoxMint: boxMint,
            userBearAccount: userTokenAccount,
            destBearAccount: destIx1.destinationAccounts[0],
            userBoxAccount,
            destBoxAccount: destIx.destinationAccounts[0],
            rewardVault,
            userRewardAccount: destinationAccounts[0],
            mintMetadata: metadata,
            tokenProgram: TOKEN_PROGRAM_ID,
            tokenMetadataProgram: METAPLEX,
        },
        instructions: [
            ...destIx1.instructions,
            ...destIx.instructions,
            ...instructions
        ],
        signers: [],
    }
    );
    await solConnection.confirmTransaction(tx, "confirmed");
    console.log("txHash = ", tx);
}

export const unstakeNft = async (userAddress: PublicKey, mint: PublicKey, boxMint: PublicKey, boxId: number) => {
    const [globalAuthority, bump] = await PublicKey.findProgramAddress(
        [Buffer.from(GLOBAL_AUTHORITY_SEED)],
        program.programId
    );

    let userBearAccount = await getAssociatedTokenAccount(userAddress, mint);
    let destTokenAccount = await getATokenAccountsNeedCreate(
        solConnection,
        userAddress,
        globalAuthority,
        [mint]
    );
    let userBoxAccount = await getAssociatedTokenAccount(userAddress, boxMint);
    let destBoxAccount = await getATokenAccountsNeedCreate(
        solConnection,
        userAddress,
        globalAuthority,
        [boxMint]
    );

    let userPoolKey = await PublicKey.createWithSeed(
        userAddress,
        "user-pool",
        program.programId,
    );

    const tx = await program.rpc.unstakeNft(
        bump, new anchor.BN(boxId), {
        accounts: {
            owner: userAddress,
            userPool: userPoolKey,
            globalAuthority,
            nftMint: mint,
            nftBoxMint: boxMint,
            userBearAccount,
            destBearAccount: destTokenAccount.destinationAccounts[0],
            userBoxAccount,
            destBoxAccount: destBoxAccount.destinationAccounts[0],
            tokenProgram: TOKEN_PROGRAM_ID,
        },
        instructions: [],
        signers: [],
    }
    );
    await solConnection.confirmTransaction(tx, "confirmed");
    console.log("txHash = ", tx);
}

export const claimReward = async (userAddress: PublicKey) => {
    const [globalAuthority, bump] = await PublicKey.findProgramAddress(
        [Buffer.from(GLOBAL_AUTHORITY_SEED)],
        program.programId
    );

    let userPoolKey = await PublicKey.createWithSeed(
        userAddress,
        USER_POOL_SEED,
        program.programId,
    );

    let { instructions, destinationAccounts } = await getATokenAccountsNeedCreate(
        solConnection,
        userAddress,
        userAddress,
        [MEDAL_TOKEN_ADDRESS]
    );
    console.log("Dest Token Account = ", destinationAccounts[0].toBase58());

    const tx = await program.rpc.claimReward(
        bump, {
        accounts: {
            owner: userAddress,
            userPool: userPoolKey,
            globalAuthority,
            rewardVault,
            userRewardAccount: destinationAccounts[0],
            tokenProgram: TOKEN_PROGRAM_ID,
        },
        instructions: [
            ...instructions,
        ],
        signers: []
    }
    );

    await solConnection.confirmTransaction(tx, "confirmed");
    console.log("txHash = ", tx);
}

/*
 *Define affiliated functions
 */
export const getGlobalState = async (
): Promise<GlobalPool | null> => {
    const [globalAuthority, bump] = await PublicKey.findProgramAddress(
        [Buffer.from(GLOBAL_AUTHORITY_SEED)],
        program.programId
    );
    try {
        let globalState = await program.account.globalPool.fetch(globalAuthority);
        return globalState as unknown as GlobalPool;
    } catch {
        return null;
    }
}

export const getUserPoolState = async (
    userAddress: PublicKey
): Promise<UserPool | null> => {
    if (!userAddress) return null;

    let userPoolKey = await PublicKey.createWithSeed(
        userAddress,
        USER_POOL_SEED,
        program.programId,
    );
    console.log('User Pool: ', userPoolKey.toBase58());
    try {
        let poolState = await program.account.userPool.fetch(userPoolKey);
        return poolState as unknown as UserPool;
    } catch {
        return null;
    }
}

const getOwnerOfNFT = async (nftMintPk: PublicKey): Promise<PublicKey> => {
    let tokenAccountPK = await getNFTTokenAccount(nftMintPk);
    let tokenAccountInfo = await solConnection.getAccountInfo(tokenAccountPK);

    console.log("nftMintPk=", nftMintPk.toBase58());
    console.log("tokenAccountInfo =", tokenAccountInfo);

    if (tokenAccountInfo && tokenAccountInfo.data) {
        let ownerPubkey = new PublicKey(tokenAccountInfo.data.slice(32, 64))
        console.log("ownerPubkey=", ownerPubkey.toBase58());
        return ownerPubkey;
    }
    return new PublicKey("");
}

const getNFTTokenAccount = async (nftMintPk: PublicKey): Promise<PublicKey> => {
    console.log("getNFTTokenAccount nftMintPk=", nftMintPk.toBase58());
    let tokenAccount = await solConnection.getProgramAccounts(
        TOKEN_PROGRAM_ID,
        {
            filters: [
                {
                    dataSize: 165
                },
                {
                    memcmp: {
                        offset: 64,
                        bytes: '2'
                    }
                },
                {
                    memcmp: {
                        offset: 0,
                        bytes: nftMintPk.toBase58()
                    }
                },
            ]
        }
    );
    return tokenAccount[0].pubkey;
}

const getAssociatedTokenAccount = async (ownerPubkey: PublicKey, mintPk: PublicKey): Promise<PublicKey> => {
    let associatedTokenAccountPubkey = (await PublicKey.findProgramAddress(
        [
            ownerPubkey.toBuffer(),
            TOKEN_PROGRAM_ID.toBuffer(),
            mintPk.toBuffer(), // mint address
        ],
        ASSOCIATED_TOKEN_PROGRAM_ID
    ))[0];
    return associatedTokenAccountPubkey;
}

export const getATokenAccountsNeedCreate = async (
    connection: anchor.web3.Connection,
    walletAddress: anchor.web3.PublicKey,
    owner: anchor.web3.PublicKey,
    nfts: anchor.web3.PublicKey[],
) => {
    let instructions = [], destinationAccounts = [];
    for (const mint of nfts) {
        const destinationPubkey = await getAssociatedTokenAccount(owner, mint);
        let response = await connection.getAccountInfo(destinationPubkey);
        if (!response) {
            const createATAIx = createAssociatedTokenAccountInstruction(
                destinationPubkey,
                walletAddress,
                owner,
                mint,
            );
            instructions.push(createATAIx);
        }
        destinationAccounts.push(destinationPubkey);

    }
    return {
        instructions,
        destinationAccounts,
    };
}

export const createAssociatedTokenAccountInstruction = (
    associatedTokenAddress: anchor.web3.PublicKey,
    payer: anchor.web3.PublicKey,
    walletAddress: anchor.web3.PublicKey,
    splTokenMintAddress: anchor.web3.PublicKey
) => {
    const keys = [
        { pubkey: payer, isSigner: true, isWritable: true },
        { pubkey: associatedTokenAddress, isSigner: false, isWritable: true },
        { pubkey: walletAddress, isSigner: false, isWritable: false },
        { pubkey: splTokenMintAddress, isSigner: false, isWritable: false },
        {
            pubkey: anchor.web3.SystemProgram.programId,
            isSigner: false,
            isWritable: false,
        },
        { pubkey: TOKEN_PROGRAM_ID, isSigner: false, isWritable: false },
        {
            pubkey: anchor.web3.SYSVAR_RENT_PUBKEY,
            isSigner: false,
            isWritable: false,
        },
    ];
    return new anchor.web3.TransactionInstruction({
        keys,
        programId: ASSOCIATED_TOKEN_PROGRAM_ID,
        data: Buffer.from([]),
    });
}

/** Get metaplex mint metadata account address */
export const getMetadata = async (mint: PublicKey): Promise<PublicKey> => {
    return (
        await PublicKey.findProgramAddress([Buffer.from('metadata'), METAPLEX.toBuffer(), mint.toBuffer()], METAPLEX)
    )[0];
};

main();