use anyhow::Result;
use indicatif::ParallelProgressIterator;
use rand::Rng;
use rayon::iter::{IntoParallelIterator, ParallelIterator};
use serde::{Deserialize, Serialize};
use solana_program::pubkey::Pubkey;
use solana_sdk::signature::Keypair;
use solana_sdk::signer::Signer;
use std::fs::OpenOptions;
use std::str::FromStr;
use std::sync::{Arc, Mutex};
use std::time::Instant;

#[derive(Serialize, Deserialize, Debug, Clone, Default)]
pub struct Whitelist {
    pub handle: String,
    pub amount: u64,
}

impl Whitelist {
    pub fn new(handle: String, amount: u64) -> Whitelist {
        Whitelist { handle, amount }
    }
}

pub fn make_list(
    number: u64,
    amount: u64,
    repeat: Option<u64>,
    pubkey_path: Option<String>,
) -> Result<()> {
    let pubkey_vec: Vec<String> = pubkey_path.map_or_else(Vec::new, |path| {
        let file = OpenOptions::new().read(true).open(path);
        if let Ok(file) = file {
            serde_json::from_reader(file).unwrap()
        } else {
            vec![]
        }
    });

    let mut pubkey_iter = pubkey_vec.into_iter();
    for i in 0..repeat.map_or_else(|| 1, |n| n) {
        let whitelist_list: Arc<Mutex<Vec<Whitelist>>> = Arc::new(Mutex::new(vec![]));
        let pubkey = pubkey_iter.next();
        let new_pubkey = pubkey.map_or_else(
            || Keypair::new().pubkey(),
            |key| Pubkey::from_str(&key).unwrap_or_else(|_| Keypair::new().pubkey()),
        );
        (0..(number as usize - 1))
            .into_par_iter()
            .progress()
            .for_each(|_| {
                let whitelist_list = Arc::clone(&whitelist_list);
                let handle = Keypair::new().pubkey().to_string();
                let new_whitelist = Whitelist::new(handle, amount);
                whitelist_list.lock().unwrap().push(new_whitelist);
            });

        let index = rand::thread_rng().gen_range(0..number as usize);

        whitelist_list
            .lock()
            .unwrap()
            .insert(index, Whitelist::new(new_pubkey.to_string(), amount));
        let whitelist_file = OpenOptions::new()
            .create_new(true)
            .write(true)
            .open(format!("./distribution{}-{}.json", number, i))
            .expect("Couldn't make new file");
        println!("Saving whitelist file #{}...", i);
        let start = Instant::now();
        serde_json::to_writer(whitelist_file, &*whitelist_list.lock().unwrap())?;
        let duration = start.elapsed();
        println!(
            "Saved whitelist file #{} in {} minutes and {} seconds!",
            i,
            duration.as_secs() / 60,
            duration.as_secs() % 60
        );
    }
    Ok(())
}
