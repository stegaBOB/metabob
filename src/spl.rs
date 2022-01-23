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
use std::{
    fs::File,
    sync::{Arc, Mutex},
};
use std::{
    fs::{self, OpenOptions},
    time::Instant,
};

use crate::decode::{decode_mint_account, get_metadata_pda, get_metadata_struct};
use solana_sdk::commitment_config::{CommitmentConfig, CommitmentLevel};
use solana_sdk::{account::Account, pubkey::Pubkey};
use spl_token::ID as TOKEN_PROGRAM_ID;

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct MintInfo {
    mint_pubkey: String,
    mint_account: Account,
}

impl From<(Pubkey, Account)> for MintInfo {
    fn from(tuple: (Pubkey, Account)) -> Self {
        MintInfo {
            mint_pubkey: tuple.0.to_string(),
            mint_account: tuple.1,
        }
    }
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct MetadataInfo {
    metadata_pubkey: String,
    metadata_account: Account,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct AccountStruct {
    mint: MintInfo,
    metadata: MetadataInfo,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TokenListEntry {
    chainId: u8,
    address: String,
    symbol: String,
    name: String,
    decimals: u8,
    logoURI: String,
}

impl TokenListEntry {
    pub fn new(
        address: String,
        symbol: String,
        name: String,
        decimals: u8,
        logoURI: String,
    ) -> TokenListEntry {
        let symbol = symbol.trim_matches(char::from(0)).to_string();
        let name = name.trim_matches(char::from(0)).to_string();
        let logoURI = logoURI.trim_matches(char::from(0)).to_string();

        TokenListEntry {
            chainId: 101,
            address,
            symbol,
            name,
            decimals,
            logoURI,
        }
    }
}

pub fn do_everything(client: &RpcClient) -> Result<Vec<TokenListEntry>> {
    let mint_accounts = get_mint_accounts(client, false)?;
    let account_vec = parse_mint_accounts(client, Some(mint_accounts), false)?;
    get_token_entries(client, Some(account_vec))
}

pub fn get_mint_accounts(client: &RpcClient, no_save: bool) -> Result<Vec<MintInfo>> {
    let mut mint_accounts_file = OpenOptions::new()
        .write(true)
        .read(true)
        .open("./mint_accounts.json")?;
    let filter1 = RpcFilterType::DataSize(82);
    let commitment = CommitmentConfig {
        commitment: CommitmentLevel::Confirmed,
    };

    let account_config = RpcAccountInfoConfig {
        encoding: Some(UiAccountEncoding::Base64),
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
    println!("Reformatting mint accounts");
    let mint_accounts = accounts_to_mint_info(mint_tuple);
    println!(
        "Found {} total fungible token mint accounts",
        mint_accounts.len()
    );
    if !no_save {
        println!("Saving fungible mint accounts file");
        serde_json::to_writer(&mut mint_accounts_file, &mint_accounts)?;
        println!("Saved fungible mint accounts file");
    };
    Ok(mint_accounts)
}

pub fn parse_mint_accounts(
    client: &RpcClient,
    mint_accounts: Option<Vec<MintInfo>>,
    no_save: bool,
) -> Result<Vec<AccountStruct>> {
    let commitment = CommitmentConfig {
        commitment: CommitmentLevel::Confirmed,
    };

    let mint_accounts = match mint_accounts {
        Some(m) => m,
        None => {
            println!("Reading mint accounts from file");
            let mut mint_accounts_file =
                File::open("./mint_accounts.json").expect("Error opening mint accounts file");
            serde_json::from_reader(&mut mint_accounts_file)
                .expect("Error parsing mint accounts file")
        }
    };

    println!("Adding metadata files!");
    let account_vec = add_metadata(client, commitment, mint_accounts);

    println!(
        "Total fungible mint accounts with metadata: {}",
        account_vec.len()
    );
    if !no_save {
        let mut file = OpenOptions::new()
            .write(true)
            .create_new(true)
            .open("./full_accounts.json")
            .expect("error creating full accounts file");
        println!("Saving full accounts file");
        serde_json::to_writer(&mut file, &account_vec)?;
    };

    Ok(account_vec)
}

pub fn get_token_entries(
    client: &RpcClient,
    full_accounts: Option<Vec<AccountStruct>>,
) -> Result<Vec<TokenListEntry>> {
    let account_vec = match full_accounts {
        Some(a) => a,
        None => {
            println!("Reading full accounts from file");
            let mut full_accounts_file = OpenOptions::new()
                .write(true)
                .read(true)
                .open("./full_accounts.json")?;
            serde_json::from_reader(&full_accounts_file).expect("Error parsing json file")
        }
    };
    let account_vec = filter_accounts(account_vec).expect("Error filtering accounts");

    println!(
        "Total fungible mint accounts with metadata: {}",
        account_vec.len()
    );

    let token_entries = get_metadata_vec(account_vec);

    fs::create_dir("./draft");
    let mut file = File::create("./draft/tokenlist.json")?;
    println!("Saving tokenlist file");
    serde_json::to_writer(&mut file, &token_entries)?;
    Ok(token_entries)
}

fn accounts_to_mint_info(accounts: Vec<(Pubkey, Account)>) -> Vec<MintInfo> {
    let mint_info: Arc<Mutex<Vec<MintInfo>>> = Arc::new(Mutex::new(Vec::new()));
    accounts
        .par_iter()
        .progress()
        .for_each(|(mint_pubkey, account)| {
            let mint_data = decode_mint_account(account);
            match mint_data {
                Ok(m) => {
                    let base: u64 = 10;
                    if m.supply > base.pow(m.decimals as u32) {
                        let mint_info = mint_info.clone();
                        let new_data: MintInfo = (*mint_pubkey, account.clone()).into();
                        mint_info.lock().unwrap().push(new_data);
                    }
                }
                Err(_) => {}
            }
        });
    Arc::try_unwrap(mint_info).unwrap().into_inner().unwrap()
}

fn add_metadata(
    client: &RpcClient,
    commitment: CommitmentConfig,
    mint_accounts: Vec<MintInfo>,
) -> Vec<AccountStruct> {
    let account_vec: Arc<Mutex<Vec<AccountStruct>>> = Arc::new(Mutex::new(Vec::new()));
    mint_accounts.par_iter().progress().for_each(|mint_info| {
        let account_vec = account_vec.clone();
        let metadata_pda_pubkey = get_metadata_pda(
            Pubkey::try_from(mint_info.mint_pubkey.as_str()).expect("Failed to parse mint pubkey"),
        );

        let metadata_account_info = client
            .get_account_with_commitment(&metadata_pda_pubkey, commitment)
            .expect("RPC error")
            .value;

        if let Some(metadata_account_data) = metadata_account_info {
            let metadata_info = MetadataInfo {
                metadata_pubkey: metadata_pda_pubkey.to_string(),
                metadata_account: metadata_account_data,
            };
            account_vec.lock().unwrap().push(AccountStruct {
                mint: mint_info.clone(),
                metadata: metadata_info,
            });
        };
    });
    Arc::try_unwrap(account_vec).unwrap().into_inner().unwrap()
}

fn filter_accounts(account_vec: Vec<AccountStruct>) -> Result<Vec<AccountStruct>> {
    let mut full_accounts_file = OpenOptions::new()
        .write(true)
        .read(true)
        .open("./full_accounts.json")?;
    let new_account_vec: Arc<Mutex<Vec<AccountStruct>>> = Arc::new(Mutex::new(Vec::new()));
    println!("Filtering accounts...");
    account_vec
        .par_iter()
        .progress()
        .for_each(|account_struct| {
            let account = &account_struct.mint.mint_account;
            let mint_data = decode_mint_account(account);
            match mint_data {
                Ok(m) => {
                    let base: u64 = 10;
                    if m.supply > base.pow(m.decimals as u32) {
                        let new_account_vec = new_account_vec.clone();
                        let mint_info = account_struct.mint.clone();
                        let metadata_info = account_struct.metadata.clone();
                        new_account_vec.lock().unwrap().push(AccountStruct {
                            metadata: metadata_info,
                            mint: mint_info,
                        });
                    }
                }
                Err(_) => {}
            }
        });
    let account_vec = Arc::try_unwrap(new_account_vec)
        .unwrap()
        .into_inner()
        .unwrap();
    println!("Saving new json");
    serde_json::to_writer(&mut full_accounts_file, &account_vec)?;
    println!("Saved file of {} fungible token mints", account_vec.len());
    Ok(account_vec)
}

fn get_metadata_vec(account_vec: Vec<AccountStruct>) -> Vec<TokenListEntry> {
    let token_entries: Arc<Mutex<Vec<TokenListEntry>>> = Arc::new(Mutex::new(Vec::new()));
    account_vec
        .par_iter()
        .progress()
        .for_each(|account_struct| {
            let metadata_data = get_metadata_struct(&account_struct.metadata.metadata_account);
            match metadata_data {
                Ok(m) => {
                    if let Ok(mint) = decode_mint_account(&account_struct.mint.mint_account) {
                        let decimals = mint.decimals;
                        let data = m.data;
                        let token_entry = TokenListEntry::new(
                            account_struct.mint.mint_pubkey.clone(),
                            data.symbol,
                            data.name,
                            decimals,
                            data.uri,
                        );
                        let token_entries = token_entries.clone();
                        token_entries.lock().unwrap().push(token_entry);
                    }
                }
                Err(_) => {}
            };
        });
    Arc::try_unwrap(token_entries)
        .unwrap()
        .into_inner()
        .unwrap()
}

pub fn do_stuff(client: &RpcClient) -> Result<()> {
    println!("Getting accounts");
    let pubkey = Pubkey::try_from("Am6CfPUwtUkmJopSzAAFAdrKb8ykrdXUmipPaaY5RfQJ")?;
    let account = client.get_account(&pubkey)?;

    // let mint_info = accounts_to_mint_info(vec![(pubkey, account)]);
    let mint_info = decode_mint_account(&account)?;
    let supply = mint_info.supply;
    println!("Total supply from account is {:?}", supply);

    let supply = client.get_token_supply(&pubkey)?;
    println!("Total supply from rpc is {:?}", supply);

    Ok(())
}

// impl From<Account> for TokenListEntry {
//     fn from(account: Account) -> Self {

//     }
// }

// {
// "chainId": 101,
// "address": "6TgvYd7eApfcZ7K5Mur7MaUQ2xT7THB4cLHWuMkQdU5Z",
// "symbol": "OTR",
// "name": "Otter Finance",
// "decimals": 9,
// "logoURI": "https://raw.githubusercontent.com/solana-labs/token-list/main/assets/mainnet/6TgvYd7eApfcZ7K5Mur7MaUQ2xT7THB4cLHWuMkQdU5Z.png",
// }
