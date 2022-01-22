use solana_client::client_error::ClientErrorKind;
use std::io;
use thiserror::Error;

#[derive(Error, Debug)]
pub enum DecodeError {
    #[error("failed to decode token mint data")]
    DecodeMintFailed(String),

    #[error("failed to decode token metadata data")]
    DecodeMetadataDataFailed(String),
}