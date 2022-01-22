use std::io::Error;
use solana_sdk::{pubkey::Pubkey, account::Account};
use crate::{errors::DecodeError};
use metaplex_token_metadata::ID as TOKEN_METADATA_PROGRAM_ID;
use solana_program::{program_option::COption, borsh::try_from_slice_unchecked, program_pack::Pack, stake::state::Meta};
use spl_token::state::Mint;
use metaplex_token_metadata::state::Metadata;


pub fn get_metadata_pda(mint_pubkey: Pubkey) -> Pubkey {
    let program_id = Pubkey::try_from(TOKEN_METADATA_PROGRAM_ID).expect("Failed to parse Token Metadata Program Id");
    let seeds = &["metadata".as_bytes(), program_id.as_ref(), mint_pubkey.as_ref()];
    let (pubkey,_) = Pubkey::find_program_address(seeds, &program_id);
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

pub fn get_metadata_struct(account: &Account) -> Result<Metadata, DecodeError>{
    let account_data = account.data.as_slice();
    let metadata: Result<Metadata, Error> = try_from_slice_unchecked(account_data);
    let token_metadata = match metadata {
        Ok(m) => m,
        Err(err)=> return Err(DecodeError::DecodeMetadataDataFailed(err.to_string())),
    };
    Ok(token_metadata)
}