use anyhow::Result;
use indicatif::ParallelProgressIterator;
use rayon::iter::{IntoParallelIterator, ParallelIterator};
use serde::{Deserialize, Serialize};
use solana_sdk::signature::Keypair;
use solana_sdk::signer::Signer;
use std::fs::OpenOptions;
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

const WHITELIST_AMOUNT: u64 = 66;

pub fn make_list(number: u64) -> Result<()> {
    let whitelist_list: Arc<Mutex<Vec<Whitelist>>> = Arc::new(Mutex::new(vec![Whitelist::new(
        "BLiwnygjXUBUNtzBLwD6162hSjxYceZDG3TNH8M4oAw3".to_string(),
        WHITELIST_AMOUNT,
    )]));
    (0..(number as usize - 2))
        .into_par_iter()
        .progress()
        .for_each(|_| {
            let whitelist_list = Arc::clone(&whitelist_list);
            let handle = Keypair::new().pubkey().to_string();
            let new_whitelist = Whitelist::new(handle, WHITELIST_AMOUNT);
            whitelist_list.lock().unwrap().push(new_whitelist);
        });
    whitelist_list.lock().unwrap().push(Whitelist::new(
        "8EiTXjobHT9qst1esCU7vYWKWjUPHdtLqF2tstnP5UwV".to_string(),
        WHITELIST_AMOUNT,
    ));
    let whitelist_file = OpenOptions::new()
        .create_new(true)
        .write(true)
        .open(format!("./test-whitelist-{}.json", number))
        .expect("Couldn't make new file");

    let start = Instant::now();
    println!("Saving whitelist file...");
    serde_json::to_writer(whitelist_file, &*whitelist_list.lock().unwrap())?;
    let duration = start.elapsed();
    println!(
        "Saved whitelist file in {} minutes and {} seconds!",
        duration.as_secs() / 60,
        duration.as_secs() % 60
    );
    Ok(())
}
