import * as anchor from '@project-serum/anchor';
import { PublicKey } from '@solana/web3.js';

export interface GlobalPool {
    superAdmin: PublicKey,
    totalStakedCount: anchor.BN,
}

export interface StakedData {
    bearMint: PublicKey,
    bearId: anchor.BN,
    boxMint: PublicKey,
    boxId: anchor.BN,
    stakedTime: anchor.BN,
}

export interface UserPool {
    owner: PublicKey,
    lastClaimedTime: anchor.BN,
    pendingReward: anchor.BN,
    stakedCount: anchor.BN,
    missionCompleted: boolean,
    stakedNfts: StakedData[],
}