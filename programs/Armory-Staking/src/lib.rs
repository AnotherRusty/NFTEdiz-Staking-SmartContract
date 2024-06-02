use anchor_lang::{prelude::*, AccountSerialize};
use anchor_spl::token::{self, Mint, Token, TokenAccount, Transfer};
use mpl_token_metadata::state::{Metadata, TokenMetadataAccount};
use solana_program::pubkey::Pubkey;

pub mod account;
pub mod constant;
pub mod error;

use account::*;
use constant::*;
use error::*;

declare_id!("GqVfxjhCXWvhQtMg9x2K2BqhRDdC35MXxDjbLVdhaDv2");

#[program]
pub mod armory_staking {
    use super::*;

    pub fn initialize(ctx: Context<Initialize>, _global_bump: u8) -> Result<()> {
        let global_authority = &mut ctx.accounts.global_authority;
        global_authority.super_admin = ctx.accounts.admin.key();

        Ok(())
    }

    pub fn init_user_pool(ctx: Context<InitUserPool>) -> Result<()> {
        let mut user_pool = ctx.accounts.user_pool.load_init()?;
        user_pool.owner = ctx.accounts.owner.key();
        msg!("Owner: {:?}", user_pool.owner.to_string());

        Ok(())
    }

    #[access_control(user(&ctx.accounts.user_pool, &ctx.accounts.owner))]
    pub fn stake_nft(ctx: Context<StakeNft>, _global_bump: u8, _box_id: u64) -> Result<()> {
        let global_authority = &mut ctx.accounts.global_authority;
        let mut user_pool = ctx.accounts.user_pool.load_mut()?;
        let mint_metadata = &mut &ctx.accounts.mint_metadata;

        msg!("Metadata Account: {:?}", ctx.accounts.mint_metadata.key());
        let (metadata, _) = Pubkey::find_program_address(
            &[
                mpl_token_metadata::state::PREFIX.as_bytes(),
                mpl_token_metadata::id().as_ref(),
                ctx.accounts.nft_mint.key().as_ref(),
            ],
            &mpl_token_metadata::id(),
        );
        require!(
            metadata == mint_metadata.key(),
            StakingError::InvalidMetadata
        );

        let nft_metadata = Metadata::from_account_info(mint_metadata)?;

        if let Some(creators) = nft_metadata.data.creators {
            let mut valid: u8 = 0;
            let mut collection: Pubkey = Pubkey::default();
            for creator in creators {
                if creator.address.to_string() == BEAR_COLLECTION_ADDRESS && creator.verified {
                    valid = 1;
                    collection = creator.address;
                    break;
                }
            }
            require!(valid == 1, StakingError::UnkownOrNotAllowedNFTCollection);
            msg!("Collection= {:?}", collection);
        } else {
            return Err(Error::from(StakingError::MetadataCreatorParseError));
        }

        let char_vec: Vec<char> = nft_metadata.data.name.chars().collect();
        let mut num_array = vec![];
        let mut idx = 0;
        let mut index = 10000;
        while idx < char_vec.len() - 1 {
            if char_vec[idx] == '#' {
                index = idx;
            }
            idx += 1;
            if index != 10000 {
                if u32::from(char_vec[idx]) == 0 {
                    break;
                }
                num_array.push(char_vec[idx]);
            }
        }

        let nft_id: String = num_array
            .into_iter()
            .map(|i| i.to_string())
            .collect::<String>();
        let id = nft_id.parse::<u64>().unwrap();
        msg!("NFT ID: {}", id);

        let timestamp = Clock::get()?.unix_timestamp;
        let user_bear_account = &mut &ctx.accounts.user_bear_account;
        let dest_bear_account = &mut &ctx.accounts.dest_bear_account;
        let token_program = &mut &ctx.accounts.token_program;

        let cpi_accounts = Transfer {
            from: user_bear_account.to_account_info().clone(),
            to: dest_bear_account.to_account_info().clone(),
            authority: ctx.accounts.owner.to_account_info().clone(),
        };
        token::transfer(
            CpiContext::new(token_program.clone().to_account_info(), cpi_accounts),
            1,
        )?;

        if _box_id != 0 {
            let user_box_account = &mut &ctx.accounts.user_box_account;
            let dest_box_account = &mut &ctx.accounts.dest_box_account;

            let cpi_accounts = Transfer {
                from: user_box_account.to_account_info().clone(),
                to: dest_box_account.to_account_info().clone(),
                authority: ctx.accounts.owner.to_account_info().clone(),
            };
            token::transfer(
                CpiContext::new(token_program.clone().to_account_info(), cpi_accounts),
                1,
            )?;
        }

        if user_pool.mission_completed == false {
            let seeds = &[GLOBAL_AUTHORITY_SEED.as_bytes(), &[_global_bump]];
            let signer = &[&seeds[..]];
            let token_program = ctx.accounts.token_program.to_account_info();
            let cpi_accounts = Transfer {
                from: ctx.accounts.reward_vault.to_account_info(),
                to: ctx.accounts.user_reward_account.to_account_info(),
                authority: global_authority.to_account_info(),
            };
            token::transfer(
                CpiContext::new_with_signer(token_program.clone(), cpi_accounts, signer),
                25_000_000_000,
            )?;

            user_pool.mission_completed = true;
        }

        user_pool.add_nft(
            ctx.accounts.nft_mint.key(),
            id,
            ctx.accounts.nft_box_mint.key(),
            _box_id,
            timestamp,
        );
        global_authority.total_staked_count += 1;

        Ok(())
    }

    #[access_control(user(&ctx.accounts.user_pool, &ctx.accounts.owner))]
    pub fn unstake_nft(ctx: Context<UnstakeNft>, _global_bump: u8, _box_id: u64) -> Result<()> {
        let global_authority = &mut ctx.accounts.global_authority;
        let mut user_pool = ctx.accounts.user_pool.load_mut()?;

        let timestamp = Clock::get()?.unix_timestamp;
        let user_bear_account = &mut &ctx.accounts.user_bear_account;
        let dest_bear_account = &mut &ctx.accounts.dest_bear_account;
        let token_program = &mut &ctx.accounts.token_program;
        let seeds = &[GLOBAL_AUTHORITY_SEED.as_bytes(), &[_global_bump]];
        let signer = &[&seeds[..]];

        let cpi_accounts = Transfer {
            from: dest_bear_account.to_account_info().clone(),
            to: user_bear_account.to_account_info().clone(),
            authority: global_authority.to_account_info(),
        };
        token::transfer(
            CpiContext::new_with_signer(
                token_program.clone().to_account_info(),
                cpi_accounts,
                signer,
            ),
            1,
        )?;

        if _box_id != 0 {
            let user_box_account = &mut &ctx.accounts.user_box_account;
            let dest_box_account = &mut &ctx.accounts.dest_box_account;

            let cpi_accounts = Transfer {
                from: dest_box_account.to_account_info().clone(),
                to: user_box_account.to_account_info().clone(),
                authority: global_authority.to_account_info(),
            };
            token::transfer(
                CpiContext::new_with_signer(
                    token_program.clone().to_account_info(),
                    cpi_accounts,
                    signer,
                ),
                1,
            )?;
        }

        user_pool.remove_nft(ctx.accounts.nft_mint.key(), timestamp)?;
        global_authority.total_staked_count -= 1;

        Ok(())
    }

    #[access_control(user(&ctx.accounts.user_pool, &ctx.accounts.owner))]
    pub fn claim_reward(ctx: Context<ClaimReward>, _global_bump: u8) -> Result<()> {
        let mut user_pool = ctx.accounts.user_pool.load_mut()?;
        let timestamp = Clock::get()?.unix_timestamp;
        require!(
            timestamp - user_pool.last_claimed_time >= ONE_DAY,
            StakingError::InvalidClaimRequest
        );
        let mut total_reward: u64 = 0;

        for i in 0..user_pool.staked_count {
            let index = i as usize;
            let mut last_claimed_time = user_pool.last_claimed_time;
            if last_claimed_time < user_pool.staked_nfts[index].staked_time {
                last_claimed_time = user_pool.staked_nfts[index].staked_time;
            }

            let reward: u64;
            let mut reward_rate: u64;

            match user_pool.staked_nfts[index].bear_id {
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

            match user_pool.staked_nfts[index].box_id {
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

            reward = (((timestamp - last_claimed_time) / ONE_DAY) as u64) * reward_rate;
            total_reward += reward;
        }
        total_reward += user_pool.pending_reward;
        user_pool.last_claimed_time = timestamp;
        user_pool.pending_reward = 0;

        require!(
            ctx.accounts.reward_vault.amount >= total_reward,
            StakingError::InsufficientRewardVault
        );

        let seeds = &[GLOBAL_AUTHORITY_SEED.as_bytes(), &[_global_bump]];
        let signer = &[&seeds[..]];
        let token_program = ctx.accounts.token_program.to_account_info();
        let cpi_accounts = Transfer {
            from: ctx.accounts.reward_vault.to_account_info(),
            to: ctx.accounts.user_reward_account.to_account_info(),
            authority: ctx.accounts.global_authority.to_account_info(),
        };
        token::transfer(
            CpiContext::new_with_signer(token_program.clone(), cpi_accounts, signer),
            total_reward,
        )?;

        Ok(())
    }
}

#[derive(Accounts)]
#[instruction(bump: u8)]
pub struct Initialize<'info> {
    #[account(mut)]
    pub admin: Signer<'info>,

    #[account(init, seeds = [GLOBAL_AUTHORITY_SEED.as_ref()], bump, space = 48, payer = admin)]
    pub global_authority: Account<'info, GlobalPool>,

    pub system_program: Program<'info, System>,
    pub rent: Sysvar<'info, Rent>,
}

#[derive(Accounts)]
pub struct InitUserPool<'info> {
    #[account(zero)]
    pub user_pool: AccountLoader<'info, UserPool>,

    #[account(mut)]
    pub owner: Signer<'info>,
}

#[derive(Accounts)]
#[instruction(bump: u8)]
pub struct StakeNft<'info> {
    #[account(mut)]
    pub owner: Signer<'info>,

    #[account(
        mut,
        seeds = [GLOBAL_AUTHORITY_SEED.as_ref()],
        bump,
    )]
    pub global_authority: Box<Account<'info, GlobalPool>>,

    #[account(mut)]
    pub user_pool: AccountLoader<'info, UserPool>,

    /// CHECK: This is not dangerous because we don't read or write from this account
    pub nft_mint: Account<'info, Mint>,

    /// CHECK: This is not dangerous because we don't read or write from this account
    pub nft_box_mint: Account<'info, Mint>,

    #[account(
        mut,
        constraint = user_bear_account.mint == *nft_mint.to_account_info().key,
        constraint = user_bear_account.owner == *owner.key,
        constraint = user_bear_account.amount == 1,
    )]
    pub user_bear_account: Box<Account<'info, TokenAccount>>,

    #[account(
        mut,
        constraint = dest_bear_account.mint == *nft_mint.to_account_info().key,
        constraint = dest_bear_account.owner == *global_authority.to_account_info().key,
    )]
    pub dest_bear_account: Box<Account<'info, TokenAccount>>,

    #[account(mut)]
    /// CHECK: This is not dangerous because we don't read or write from this account
    pub user_box_account: AccountInfo<'info>,

    #[account(mut)]
    /// CHECK: This is not dangerous because we don't read or write from this account
    pub dest_box_account: AccountInfo<'info>,

    #[account(
        mut,
        constraint = reward_vault.mint == MEDAL_TOKEN_ADDRESS.parse::<Pubkey>().unwrap(),
        constraint = reward_vault.owner == global_authority.key(),
    )]
    pub reward_vault: Box<Account<'info, TokenAccount>>,

    #[account(
        mut,
        constraint = user_reward_account.mint == MEDAL_TOKEN_ADDRESS.parse::<Pubkey>().unwrap(),
        constraint = user_reward_account.owner == owner.key(),
    )]
    pub user_reward_account: Box<Account<'info, TokenAccount>>,

    #[account(mut)]
    /// CHECK: This is not dangerous because we don't read or write from this account
    pub mint_metadata: AccountInfo<'info>,

    pub token_program: Program<'info, Token>,
    #[account(constraint = token_metadata_program.key == &mpl_token_metadata::ID)]
    /// CHECK: This is not dangerous because we don't read or write from this account
    pub token_metadata_program: AccountInfo<'info>,
}

#[derive(Accounts)]
#[instruction(
    bump: u8,
)]
pub struct UnstakeNft<'info> {
    #[account(mut)]
    pub owner: Signer<'info>,

    #[account(mut)]
    pub user_pool: AccountLoader<'info, UserPool>,

    #[account(
        mut,
        seeds = [GLOBAL_AUTHORITY_SEED.as_ref()],
        bump,
    )]
    pub global_authority: Box<Account<'info, GlobalPool>>,

    /// CHECK: This is not dangerous because we don't read or write from this account
    pub nft_mint: Account<'info, Mint>,

    /// CHECK: This is not dangerous because we don't read or write from this account
    pub nft_box_mint: Account<'info, Mint>,

    #[account(
        mut,
        constraint = user_bear_account.mint == *nft_mint.to_account_info().key,
        constraint = user_bear_account.owner == *owner.key,
    )]
    pub user_bear_account: Box<Account<'info, TokenAccount>>,

    #[account(
        mut,
        constraint = dest_bear_account.mint == *nft_mint.to_account_info().key,
        constraint = dest_bear_account.owner == *global_authority.to_account_info().key,
        constraint = dest_bear_account.amount == 1,
    )]
    pub dest_bear_account: Box<Account<'info, TokenAccount>>,

    #[account(mut)]
    /// CHECK: This is not dangerous because we don't read or write from this account
    pub user_box_account: AccountInfo<'info>,

    #[account(mut)]
    /// CHECK: This is not dangerous because we don't read or write from this account
    pub dest_box_account: AccountInfo<'info>,

    pub token_program: Program<'info, Token>,
}

#[derive(Accounts)]
#[instruction(bump: u8)]
pub struct ClaimReward<'info> {
    #[account(mut)]
    pub owner: Signer<'info>,

    #[account(mut)]
    pub user_pool: AccountLoader<'info, UserPool>,

    #[account(
        mut,
        seeds = [GLOBAL_AUTHORITY_SEED.as_ref()],
        bump,
    )]
    pub global_authority: Account<'info, GlobalPool>,

    #[account(
        mut,
        constraint = reward_vault.mint == MEDAL_TOKEN_ADDRESS.parse::<Pubkey>().unwrap(),
        constraint = reward_vault.owner == global_authority.key(),
    )]
    pub reward_vault: Box<Account<'info, TokenAccount>>,

    #[account(
        mut,
        constraint = user_reward_account.mint == MEDAL_TOKEN_ADDRESS.parse::<Pubkey>().unwrap(),
        constraint = user_reward_account.owner == owner.key(),
    )]
    pub user_reward_account: Box<Account<'info, TokenAccount>>,

    pub token_program: Program<'info, Token>,
}

// Access control modifiers
fn user(pool_loader: &AccountLoader<UserPool>, user: &AccountInfo) -> Result<()> {
    let user_pool = pool_loader.load()?;
    require!(user_pool.owner == *user.key, StakingError::InvalidUserPool);
    Ok(())
}
