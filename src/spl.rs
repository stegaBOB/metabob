use anyhow::Result;
use indicatif::ParallelProgressIterator;
use rayon::iter::{IntoParallelRefIterator, ParallelIterator};
use serde::{Deserialize, Serialize};
use solana_account_decoder::UiAccountEncoding;
use solana_client::{
    rpc_client::RpcClient,
    rpc_config::{RpcAccountInfoConfig, RpcProgramAccountsConfig},
    rpc_filter::RpcFilterType,
};
use spl_token::state::Mint;
use std::{
    fs::{self, OpenOptions},
    time::Instant,
};
use std::{
    io::BufReader,
    sync::{Arc, Mutex},
};

use crate::decode::{decode_metadata_account, decode_mint_account, get_metadata_pda};
use metaplex_token_metadata::state::Metadata;
use spl_token::ID as TOKEN_PROGRAM_ID;

use solana_sdk::commitment_config::{CommitmentConfig, CommitmentLevel};
use solana_sdk::{account::Account, pubkey::Pubkey};

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct MintStruct {
    pub supply: u64,
    pub decimals: u8,
}
impl From<Mint> for MintStruct {
    fn from(mint: Mint) -> Self {
        MintStruct {
            supply: mint.supply,
            decimals: mint.decimals,
        }
    }
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct MetadataStruct {
    pub mint: Pubkey,
    pub name: String,
    pub symbol: String,
    pub uri: String,
}

impl From<Metadata> for MetadataStruct {
    fn from(metadata: Metadata) -> Self {
        MetadataStruct {
            mint: metadata.mint,
            name: metadata.data.name,
            symbol: metadata.data.symbol,
            uri: metadata.data.uri,
        }
    }
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct MintInfo {
    mint_pubkey: Pubkey,
    mint: MintStruct,
}
impl MintInfo {
    fn is_fungible(&self) -> bool {
        let base: u64 = 10;
        self.mint.supply > base.pow(self.mint.decimals as u32)
    }
}

impl TryFrom<(Pubkey, Account)> for MintInfo {
    type Error = &'static str;
    fn try_from(tuple: (Pubkey, Account)) -> Result<Self, Self::Error> {
        let mint = decode_mint_account(&tuple.1);
        match mint {
            Ok(m) => Ok(MintInfo {
                mint_pubkey: tuple.0,
                mint: m.into(),
            }),
            Err(_) => Err("Error decoding mint account"),
        }
    }
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct MetadataInfo {
    metadata_pubkey: Pubkey,
    metadata: MetadataStruct,
}

impl TryFrom<(Pubkey, Account)> for MetadataInfo {
    type Error = &'static str;

    fn try_from(tuple: (Pubkey, Account)) -> Result<Self, Self::Error> {
        let metadata = decode_metadata_account(&tuple.1);
        match metadata {
            Ok(m) => Ok(MetadataInfo {
                metadata_pubkey: tuple.0,
                metadata: m.into(),
            }),
            Err(_) => Err("error decoding metadata account"),
        }
    }
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct AccountStruct {
    mint: MintInfo,
    metadata: MetadataInfo,
}
impl AccountStruct {
    fn new(mint: MintInfo, metadata: MetadataInfo) -> AccountStruct {
        AccountStruct { mint, metadata }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct TokenListEntry {
    chain_id: u8,
    address: String,
    symbol: String,
    name: String,
    decimals: u8,
    logo_uri: String,
}

impl TokenListEntry {
    pub fn new(
        address: String,
        symbol: String,
        name: String,
        decimals: u8,
        logo_uri: String,
    ) -> TokenListEntry {
        let symbol = symbol.trim_matches(char::from(0)).to_string();
        let name = name.trim_matches(char::from(0)).to_string();
        let logo_uri = logo_uri.trim_matches(char::from(0)).to_string();

        TokenListEntry {
            chain_id: 101,
            address,
            symbol,
            name,
            decimals,
            logo_uri,
        }
    }
}

impl From<AccountStruct> for TokenListEntry {
    fn from(a: AccountStruct) -> Self {
        let mint = a.mint;
        let metadata = a.metadata.metadata;
        TokenListEntry::new(
            mint.mint_pubkey.to_string(),
            metadata.symbol,
            metadata.name,
            mint.mint.decimals,
            metadata.uri,
        )
    }
}

pub fn do_everything(
    client: &RpcClient,
    heavy_client: &RpcClient,
    no_save: bool,
) -> Result<Vec<TokenListEntry>> {
    let fungible_mint_accounts = get_mint_accounts(heavy_client, no_save)?;
    let account_info = get_metadata_accounts(client, Some(fungible_mint_accounts), no_save)?;
    let token_list = get_token_entries(Some(account_info), no_save)?;
    parse_token_uri(Some(token_list), false)
}

pub fn get_mint_accounts(client: &RpcClient, no_save: bool) -> Result<Vec<MintInfo>> {
    let mut mint_accounts_file = OpenOptions::new()
        .write(true)
        .read(true)
        .create(true)
        .open("./mint_info.json")?;
    let filter1 = RpcFilterType::DataSize(82);
    let commitment = CommitmentConfig {
        commitment: CommitmentLevel::Finalized,
    };

    let account_config = RpcAccountInfoConfig {
        encoding: Some(UiAccountEncoding::Base64),
        // data_slice: Some(UiDataSliceConfig {
        //     length: 0,
        //     offset: 0,
        // }),
        data_slice: None,
        commitment: Some(commitment),
    };

    let config = RpcProgramAccountsConfig {
        filters: Some(vec![filter1]),
        account_config,
        with_context: None,
    };

    println!("Looking for mint accounts. This may take a while...");
    let start = Instant::now();
    let mint_tuple = client.get_program_accounts_with_config(&TOKEN_PROGRAM_ID, config)?;
    let duration = start.elapsed();
    println!(
        "Found {} mint accounts in {} minutes and {} seconds!",
        mint_tuple.len(),
        duration.as_secs() / 60,
        duration.as_secs() % 60
    );

    println!("Filtering mint accounts...");
    let parsed_mint_accounts = filter_mints(mint_tuple);
    println!(
        "Total fungible mint accounts: {}",
        parsed_mint_accounts.len()
    );
    if !no_save {
        println!("Saving fungible mint accounts file...");
        let start = Instant::now();
        serde_json::to_writer(&mut mint_accounts_file, &parsed_mint_accounts)?;
        let duration = start.elapsed();
        println!(
            "Saved fungible mint accounts file in {} minutes and {} seconds!",
            duration.as_secs() / 60,
            duration.as_secs() % 60
        );
    };
    Ok(parsed_mint_accounts)
}

pub fn get_metadata_accounts(
    client: &RpcClient,
    mint_info: Option<Vec<MintInfo>>,
    no_save: bool,
) -> Result<Vec<AccountStruct>> {
    let mint_info = match mint_info {
        Some(t) => t,
        None => {
            let token_list_file = OpenOptions::new()
                .write(true)
                .read(true)
                .open("./mint_info.json")?;
            println!("Reading fungible mint info from file...");
            let reader = BufReader::new(&token_list_file);
            let to_return: Vec<MintInfo> =
                serde_json::from_reader(reader).expect("Error parsing mint info file");
            println!("Read {} fungible mint accounts.", to_return.len());
            to_return
        }
    };

    let account_info: Arc<Mutex<Vec<AccountStruct>>> = Arc::new(Mutex::new(Vec::new()));
    let commitment = CommitmentConfig {
        commitment: CommitmentLevel::Finalized,
    };
    mint_info.par_iter().progress().for_each(|mint_info| {
        let metadata_pubkey = get_metadata_pda(&mint_info.mint_pubkey);
        let metadata_account =
            RpcClient::get_account_with_commitment(client, &metadata_pubkey, commitment);
        if let Ok(account) = metadata_account {
            let account = account.value;
            if let Some(account) = account {
                let account_info = account_info.clone();
                let metadata_info = MetadataInfo::try_from((metadata_pubkey, account));
                if let Ok(..) = metadata_info {
                    account_info.lock().unwrap().push(AccountStruct::new(
                        mint_info.clone(),
                        metadata_info.unwrap(),
                    ));
                }
            }
        }
    });
    let account_info = Arc::try_unwrap(account_info).unwrap().into_inner().unwrap();

    println!(
        "Found {} fungible accounts with metadata.",
        account_info.len()
    );

    if !no_save {
        let file = OpenOptions::new()
            .write(true)
            .create(true)
            .open("./account_info.json");
        match file {
            Err(_) => println!("Error opening account info file"),
            Ok(mut f) => {
                println!("Saving accounts info file...");
                let start = Instant::now();
                serde_json::to_writer(&mut f, &account_info)?;
                let duration = start.elapsed();
                println!(
                    "Saved accounts info file in {} minutes and {} seconds!",
                    duration.as_secs() / 60,
                    duration.as_secs() % 60
                );
            }
        }
    };

    Ok(account_info)
}

pub fn get_token_entries(
    full_accounts: Option<Vec<AccountStruct>>,
    no_save: bool,
) -> Result<Vec<TokenListEntry>> {
    let account_vec = match full_accounts {
        Some(a) => a,
        None => {
            println!("Reading accounts info from file...");
            let full_accounts_file = OpenOptions::new()
                .write(true)
                .read(true)
                .open("./account_info.json")?;
            let reader = BufReader::new(&full_accounts_file);
            let to_return = serde_json::from_reader(reader).expect("Error parsing json file");
            println!("Read full accounts file.");
            to_return
        }
    };
    // let account_vec = filter_accounts(account_vec).expect("Error filtering accounts");
    println!(
        "Total fungible mint accounts with metadata: {}",
        account_vec.len()
    );
    let token_entries = get_token_entry_vec(account_vec);

    if !no_save {
        let _create_dir = fs::create_dir("./draft");
        let start = Instant::now();
        let mut token_list_file = OpenOptions::new()
            .write(true)
            .create(true)
            .open("./draft/tokenlist.json")?;
        println!("Saving tokenlist file...");
        serde_json::to_writer(&mut token_list_file, &token_entries)?;
        let duration = start.elapsed();
        println!(
            "Saved tokenlist file in {} minutes and {} seconds!",
            duration.as_secs() / 60,
            duration.as_secs() % 60
        );
    }
    Ok(token_entries)
}

pub fn parse_token_uri(
    token_entries: Option<Vec<TokenListEntry>>,
    no_save: bool,
) -> Result<Vec<TokenListEntry>> {
    let token_entries = match token_entries {
        Some(t) => t,
        None => {
            println!("Reading token entries from file...");
            let token_list_file = OpenOptions::new()
                .write(true)
                .read(true)
                .open("./draft/tokenlist.json")?;
            let reader = BufReader::new(&token_list_file);

            let to_return: Vec<TokenListEntry> =
                serde_json::from_reader(reader).expect("Error parsing token list file");
            println!(
                "Read token list file of {} token list entries.",
                to_return.len()
            );
            to_return
        }
    };

    let _create_dir = fs::create_dir("./draft");
    let mut no_uri_file = OpenOptions::new()
        .write(true)
        .read(true)
        .create(true)
        .open("./draft/no_uri_tokenlist.json")?;
    let mut uri_file = OpenOptions::new()
        .write(true)
        .read(true)
        .create(true)
        .open("./draft/uri_tokenlist.json")?;

    let uri_list: Arc<Mutex<Vec<TokenListEntry>>> = Arc::new(Mutex::new(Vec::new()));
    let no_uri_list: Arc<Mutex<Vec<TokenListEntry>>> = Arc::new(Mutex::new(Vec::new()));

    println!("Parsing token list...");
    token_entries.par_iter().progress().for_each(|entry| {
        if entry.logo_uri.is_empty() {
            let no_uri_list = no_uri_list.clone();
            no_uri_list.lock().unwrap().push(entry.clone());
        } else {
            let uri_list = uri_list.clone();
            uri_list.lock().unwrap().push(entry.clone());
        }
    });

    let uri_list = Arc::try_unwrap(uri_list).unwrap().into_inner().unwrap();
    let no_uri_list = Arc::try_unwrap(no_uri_list).unwrap().into_inner().unwrap();
    println!(
        "Found {} items with a URI and {} items with no URI.",
        uri_list.len(),
        no_uri_list.len()
    );
    if !no_save {
        println!("Saving pretty printed parsed token list files...");
        serde_json::to_writer_pretty(&mut uri_file, &uri_list)
            .expect("Error writing uri token list");
        serde_json::to_writer_pretty(&mut no_uri_file, &no_uri_list)
            .expect("Error writing no uri token list");
        println!("Saved parsed token list files!");
    }

    Ok(uri_list)
}

fn filter_mints(mint_accounts: Vec<(Pubkey, Account)>) -> Vec<MintInfo> {
    let mint_vec: Arc<Mutex<Vec<MintInfo>>> = Arc::new(Mutex::new(Vec::new()));
    mint_accounts
        .par_iter()
        .progress()
        .for_each(|(mint_pubkey, mint_account)| {
            let mint_vec = mint_vec.clone();

            let mint_info = MintInfo::try_from((*mint_pubkey, mint_account.clone()));

            if let Ok(mint) = mint_info {
                if mint.is_fungible() {
                    mint_vec.lock().unwrap().push(mint);
                }
            }
        });
    Arc::try_unwrap(mint_vec).unwrap().into_inner().unwrap()
}

// fn filter_accounts(account_vec: Vec<AccountStruct>) -> Result<Vec<AccountStruct>> {
//     let mut full_accounts_file = OpenOptions::new()
//         .write(true)
//         .read(true)
//         .open("./full_accounts.json")?;
//     let new_account_vec: Arc<Mutex<Vec<AccountStruct>>> = Arc::new(Mutex::new(Vec::new()));
//     println!("Filtering accounts...");
//     account_vec
//         .par_iter()
//         .progress()
//         .for_each(|account_struct| {
//             let account = &account_struct.mint.mint_account;
//             let mint_data = decode_mint_account(account);
//             match mint_data {
//                 Ok(m) => {
//                     let base: u64 = 10;
//                     if m.supply > base.pow(m.decimals as u32) {
//                         let new_account_vec = new_account_vec.clone();
//                         let mint_info = account_struct.mint.clone();
//                         let metadata_info = account_struct.metadata.clone();
//                         new_account_vec.lock().unwrap().push(AccountStruct {
//                             metadata: metadata_info,
//                             mint: mint_info,
//                         });
//                     }
//                 }
//                 Err(_) => {}
//             }
//         });
//     let account_vec = Arc::try_unwrap(new_account_vec)
//         .unwrap()
//         .into_inner()
//         .unwrap();
//     println!("Saving new filtered accounts json...");
//     serde_json::to_writer(&mut full_accounts_file, &account_vec)?;
//     println!("Saved file of {} fungible token mints", account_vec.len());
//     Ok(account_vec)
// }

fn get_token_entry_vec(account_vec: Vec<AccountStruct>) -> Vec<TokenListEntry> {
    let token_entries: Arc<Mutex<Vec<TokenListEntry>>> = Arc::new(Mutex::new(Vec::new()));
    account_vec
        .par_iter()
        .progress()
        .for_each(|account_struct| {
            let token_entries = token_entries.clone();
            token_entries
                .lock()
                .unwrap()
                .push(TokenListEntry::from(account_struct.clone()));
        });
    Arc::try_unwrap(token_entries)
        .unwrap()
        .into_inner()
        .unwrap()
}
pub fn do_stuff() -> Result<()> {
    Ok(())
}

// fn accounts_to_metadata_info(accounts: Vec<(Pubkey, Account)>) -> Vec<MintInfo> {
//     let mint_info: Arc<Mutex<Vec<MintInfo>>> = Arc::new(Mutex::new(Vec::new()));
//     accounts
//         .par_iter()
//         .progress()
//         .for_each(|(mint_pubkey, account)| {
//             let mint_data = MintInfo::try_from((*mint_pubkey, account.clone())).unwrap();
//             let mint = &mint_data.mint;
//             let base: u64 = 10;
//             if mint.supply > base.pow(mint.decimals as u32) {
//                 let mint_info = mint_info.clone();
//                 mint_info.lock().unwrap().push(mint_data);
//             }
//         });
//     Arc::try_unwrap(mint_info).unwrap().into_inner().unwrap()
// }
