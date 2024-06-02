use anchor_lang::prelude::*;
use std::clone::Clone;

use crate::constant::*;
use crate::error::*;

#[account]
#[derive(Default)]
pub struct GlobalPool {
    //Total Size: 8 + 40 = 48
    pub super_admin: Pubkey,     //32
    pub total_staked_count: u64, //8
}

#[zero_copy]
#[derive(Default)]
pub struct StakedData {
    pub bear_mint: Pubkey, // 32
    pub bear_id: u64,      // 8
    pub box_mint: Pubkey,  // 32
    pub box_id: u64,       // 8
    pub staked_time: i64,  // 8
}

#[account(zero_copy)]
pub struct UserPool {
    //Total Size: 8 + 2704
    pub owner: Pubkey,                              // 32
    pub last_claimed_time: i64,                     // 8
    pub pending_reward: u64,                        // 8
    pub staked_count: u64,                          // 8
    pub mission_completed: bool,                    // 8
    pub staked_nfts: [StakedData; STAKE_MAX_COUNT], // 88 * 30
}

impl Default for UserPool {
    #[inline]
    fn default() -> UserPool {
        UserPool {
            owner: Pubkey::default(),
            last_claimed_time: 0,
            pending_reward: 0,
            staked_count: 0,
            mission_completed: false,
            staked_nfts: [StakedData {
                ..Default::default()
            }; STAKE_MAX_COUNT],
        }
    }
}

impl UserPool {
    pub fn add_nft(
        &mut self,
        bear_nft: Pubkey,
        bear_id: u64,
        box_nft: Pubkey,
        box_id: u64,
        now: i64,
    ) {
        let idx = self.staked_count as usize;
        self.staked_nfts[idx].bear_mint = bear_nft;
        self.staked_nfts[idx].bear_id = bear_id;
        self.staked_nfts[idx].box_mint = box_nft;
        self.staked_nfts[idx].box_id = box_id;
        self.staked_nfts[idx].staked_time = now;
        self.staked_count += 1;
    }

    pub fn remove_nft(&mut self, bear_nft: Pubkey, now: i64) -> Result<()> {
        let mut withdrawn: u8 = 0;
        let mut index: usize = 0;
        // Find NFT in pool
        for i in 0..self.staked_count {
            let idx = i as usize;
            if self.staked_nfts[idx].bear_mint.eq(&bear_nft) {
                index = idx;
                withdrawn = 1;
                break;
            }
        }
        require!(withdrawn == 1, StakingError::InvalidNftAddress);

        let reward: u64;
        let mut reward_rate: u64;

        match self.staked_nfts[index].bear_id {
            1..=1920 | 8001..=8480 => {
                reward_rate = 10;
            }
            1921..=3520 | 8481..=8880 => {
                reward_rate = 14;
            }
            3521..=4880 | 8881..=9220 => {
                reward_rate = 17;
            }
            4881..=6000 | 9221..=9500 => {
                reward_rate = 20;
            }
            6001..=6880 | 9501..=9720 => {
                reward_rate = 22;
            }
            6881..=7520 | 9721..=9880 => {
                reward_rate = 24;
            }
            7521..=7920 | 9881..=9980 => {
                reward_rate = 26;
            }
            7921..=7992 | 9981..=9998 => {
                reward_rate = 30;
            }
            7993..=8000 | 9999..=10000 => {
                reward_rate = 100;
            }
            _ => {
                return Err(StakingError::NotAllowedNFTID.into());
            }
        }

        match self.staked_nfts[index].box_id {
            1..=5000 => {
                reward_rate *= 1_100_000_000;
            }
            5001..=8000 => {
                reward_rate *= 1_200_000_000;
            }
            8001..=9500 => {
                reward_rate *= 1_300_000_000;
            }
            9501..=10000 => {
                reward_rate *= 1_500_000_000;
            }
            _ => {
                reward_rate *= 1_000_000_000;
            }
        }

        let mut last_claimed_time: i64 = self.last_claimed_time;
        if last_claimed_time < self.staked_nfts[index].staked_time {
            last_claimed_time = self.staked_nfts[index].staked_time;
        }

        reward = (((now - last_claimed_time) / ONE_DAY) as u64) * reward_rate;
        self.pending_reward += reward;

        // Remove NFT from pool
        let last_idx: usize = (self.staked_count - 1) as usize;
        if index != last_idx {
            self.staked_nfts[index] = self.staked_nfts[last_idx];
        }
        self.staked_count -= 1;

        Ok(())
    }
}
