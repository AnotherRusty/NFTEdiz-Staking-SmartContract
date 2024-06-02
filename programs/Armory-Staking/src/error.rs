use anchor_lang::prelude::*;

#[error_code]
pub enum StakingError {
    #[msg("Invalid NFT Address")]
    InvalidNftAddress,
    #[msg("Invalid NFT Metadata")]
    InvalidMetadata,
    #[msg("Invalid UserPool")]
    InvalidUserPool,
    #[msg("Unknown or not allowed NFT collection")]
    UnkownOrNotAllowedNFTCollection,
    #[msg("Not allowed NFT ID for staking")]
    NotAllowedNFTID,
    #[msg("Faild to parse metadata information")]
    MetadataCreatorParseError,
    #[msg("You can't claim reward now")]
    InvalidClaimRequest,
    #[msg("There isn't such a amount of token in the vault")]
    InsufficientRewardVault,
}
