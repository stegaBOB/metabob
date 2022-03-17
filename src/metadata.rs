use crate::limiter::create_rate_limiter;
use crate::{
    constants::{MAX_NAME_LENGTH, MAX_SYMBOL_LENGTH, MAX_URI_LENGTH, USE_RATE_LIMIT},
    parse::parse_solana_config,
};
use anyhow::{Result};
use indicatif::ParallelProgressIterator;
use log::{error, info};
use mpl_token_metadata::{
    instruction::sign_metadata, state::Metadata, ID as TOKEN_METADATA_PROGRAM_ID,
};
use rayon::iter::{IntoParallelRefIterator, ParallelIterator};
use retry::{delay::Exponential, retry};
use solana_account_decoder::UiAccountEncoding;
use solana_client::{
    rpc_client::RpcClient,
    rpc_config::{RpcAccountInfoConfig, RpcProgramAccountsConfig},
    rpc_filter::{Memcmp, MemcmpEncodedBytes, RpcFilterType},
};
use solana_program::account_info::AccountInfo;
use solana_sdk::{
    account::Account,
    commitment_config::{CommitmentConfig, CommitmentLevel},
    pubkey::Pubkey,
    signature::{read_keypair_file, Signature},
    signer::{keypair::Keypair, Signer},
    transaction::Transaction,
};
use std::str::FromStr;
use std::{
    fs::OpenOptions,
    sync::{Arc, Mutex},
};

pub fn count_creators(client: &RpcClient, creator: String) -> Result<Vec<Pubkey>> {
    let creator_pubkey = Pubkey::from_str(&creator).expect("Couldn't parse a pubkey from your option");
    let creator_string = creator_pubkey.to_string();
    let accounts_vec: Arc<Mutex<Vec<Pubkey>>> = Arc::new(Mutex::new(Vec::new()));

    let index_vec: Vec<usize> = vec![0, 1, 2, 3, 4];
    println!("Looking for metadata accounts that the following address can sign: {}", creator_string);
    index_vec.par_iter().for_each(|i| {
        let accounts_vec = accounts_vec.clone();
        let next_accounts = get_metadata_creator_accounts(client, &creator_string, i.clone())
            .expect(&format!("Couldn't finish the GPA for creator index {}", i));
        let total = next_accounts.len();
        let unsigned_mints: Arc<Mutex<Vec<Pubkey>>> = Arc::new(Mutex::new(Vec::new()));
        next_accounts.par_iter().for_each(|(pubkey, account)| {
            let mut account = account.clone();
            let account_info = AccountInfo::from((pubkey, &mut account));
            let metadata_account =
                Metadata::from_account_info(&account_info).expect("Couldn't deserialize metadata");
            let creator = &metadata_account.data.creators.unwrap()[*i];
            if !creator.verified {
                unsigned_mints.lock().unwrap().push(pubkey.clone());
            }
        });
        let mut unsigned_mints = Arc::try_unwrap(unsigned_mints)
            .unwrap()
            .into_inner()
            .unwrap();
        println!("In position {}:\n  Found {} unverified of {} total", i,
            unsigned_mints.len(),
            total
        );
        accounts_vec.lock().unwrap().append(&mut unsigned_mints);
    });

    let accounts_vec = Arc::try_unwrap(accounts_vec).unwrap().into_inner().unwrap();
    let accounts_strings_vec: Vec<String> =
        accounts_vec.iter().map(|key| key.to_string()).collect();

    println!(
        "Found {} total metadata accounts that still need to be signed",
        accounts_vec.len()
    );

    if accounts_vec.len() > 0 {
        let file1 = OpenOptions::new()
            .write(true)
            .create(true)
            .open("./metadata_list.json");
        match file1 {
            Err(_) => println!("Error opening metadata_keys file"),
            Ok(mut f) => {
                println!("Saving metadata list info file...");
                serde_json::to_writer(&mut f, &accounts_strings_vec);
            }
        }

        let file2 = OpenOptions::new()
            .write(true)
            .create(true)
            .open("./metadata_pubkeys.json");
        match file2 {
            Err(_) => println!("Error opening metadata pubkeys file"),
            Ok(mut f) => {
                println!("Saving metadata pubkeys info file...");
                serde_json::to_writer(&mut f, &accounts_vec);
            }
        }
    }
    Ok(accounts_vec)
}

pub fn sign_all(client: &RpcClient, keypair_path: Option<String>) -> Result<()> {
    let solana_opts = parse_solana_config();
    let keypair: Keypair = match keypair_path {
        Some(path) => read_keypair_file(path).expect("Uh I cant read that keypair file :cry:"),
        None => {
            let solana_config = solana_opts
                .expect("You didn't pass in a keypair and your Solana config is no bueno");
            let keypair_path = solana_config.keypair_path;
            read_keypair_file(keypair_path)
                .expect("Uh I cant read that keypair file from your Solana config :cry:")
        }
    };

    let creator_pubkey = keypair.pubkey();
    let creator_string = creator_pubkey.to_string();
    let accounts_vec = count_creators(&client, creator_string)?;

    if accounts_vec.len() > 0 {
        println!("Now signing metadata...");

        let use_rate_limit = *USE_RATE_LIMIT.read().unwrap();
        let handle = create_rate_limiter();

        // also basically taken directly from metaboss
        accounts_vec
            .par_iter()
            .progress()
            .for_each(|metadata_pubkey| {
                let mut handle = handle.clone();
                if use_rate_limit {
                    handle.wait();
                }
                // Try to sign all accounts, print any errors that crop up.
                match sign(client, &keypair, *metadata_pubkey) {
                    Ok(sig) => info!("{}", sig),
                    Err(e) => error!("{}", e),
                }
            });
    }
    Ok(())
}

// From metaboss
pub fn get_metadata_creator_accounts(
    client: &RpcClient,
    creator: &String,
    position: usize,
) -> Result<Vec<(Pubkey, Account)>> {
    if position > 4 {
        error!("CM Creator position cannot be greator than 4");
        std::process::exit(1);
    }

    let config = RpcProgramAccountsConfig {
        filters: Some(vec![RpcFilterType::Memcmp(Memcmp {
            offset: 1 + // key
            32 + // update auth
            32 + // mint
            4 + // name string length
            MAX_NAME_LENGTH + // name
            4 + // uri string length
            MAX_URI_LENGTH + // uri*
            4 + // symbol string length
            MAX_SYMBOL_LENGTH + // symbol
            2 + // seller fee basis points
            1 + // whether or not there is a creators vec
            4 + // creators
            position * // index for each creator
            (
                32 + // address
                1 + // verified
                1 // share
            ),
            bytes: MemcmpEncodedBytes::Base58(creator.to_string()),
            encoding: None,
        })]),
        account_config: RpcAccountInfoConfig {
            encoding: Some(UiAccountEncoding::Base64),
            data_slice: None,
            commitment: Some(CommitmentConfig {
                commitment: CommitmentLevel::Confirmed,
            }),
        },
        with_context: None,
    };

    let accounts = client.get_program_accounts_with_config(&TOKEN_METADATA_PROGRAM_ID, config)?;

    Ok(accounts)
}

// From metaboss
pub fn sign(client: &RpcClient, creator: &Keypair, metadata_pubkey: Pubkey) -> Result<Signature> {
    let recent_blockhash = client.get_latest_blockhash()?;
    let ix = sign_metadata(TOKEN_METADATA_PROGRAM_ID, metadata_pubkey, creator.pubkey());
    let tx = Transaction::new_signed_with_payer(
        &[ix],
        Some(&creator.pubkey()),
        &[creator],
        recent_blockhash,
    );

    // Send tx with retries.
    let res = retry(
        Exponential::from_millis_with_factor(250, 2.0).take(3),
        || client.send_and_confirm_transaction(&tx),
    );
    let sig = res?;

    Ok(sig)
}
