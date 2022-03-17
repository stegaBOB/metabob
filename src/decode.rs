use crate::errors::DecodeError;
use metaplex_token_metadata::state::Metadata;
use metaplex_token_metadata::ID as TOKEN_METADATA_PROGRAM_ID;
use solana_account_decoder;
use solana_program::{borsh::try_from_slice_unchecked, program_pack::Pack};
use solana_sdk::{account::Account, pubkey::Pubkey};
use spl_token::state::Mint;
use std::io::Error;

pub fn get_metadata_pda(mint_pubkey: &Pubkey) -> Pubkey {
    let seeds = &[
        "metadata".as_bytes(),
        TOKEN_METADATA_PROGRAM_ID.as_ref(),
        mint_pubkey.as_ref(),
    ];
    let (pubkey, _) = Pubkey::find_program_address(seeds, &TOKEN_METADATA_PROGRAM_ID);
    pubkey
}

pub fn decode_mint_account(mint_account: &Account) -> Result<Mint, DecodeError> {
    let account_data = mint_account.data.as_slice();
    let mint_data: Mint = match spl_token::state::Mint::unpack(account_data) {
        Ok(m) => m,
        Err(err) => return Err(DecodeError::DecodeMintFailed(err.to_string())),
    };
    Ok(mint_data)
}

// pub fn decode_candy_account(candy_v1: &Account){
//     debug_account_data(data, f);
// }

pub fn decode_metadata_account(account: &Account) -> Result<Metadata, DecodeError> {
    let account_data = account.data.as_slice();
    let metadata: Result<Metadata, Error> = try_from_slice_unchecked(account_data);
    let token_metadata = match metadata {
        Ok(m) => m,
        Err(err) => return Err(DecodeError::DecodeMetadataDataFailed(err.to_string())),
    };
    Ok(token_metadata)
}
