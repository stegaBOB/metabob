#[macro_use]
extern crate log;

use anyhow::Result;
use metabob::opt::*;
use metabob::parse::*;
use metabob::process_subcommands::*;
use solana_client::rpc_client::RpcClient;
use solana_sdk::commitment_config::CommitmentConfig;
use std::str::FromStr;
use std::time::Duration;
use structopt::StructOpt;

fn main() -> Result<()> {
    let options = Opt::from_args();
    let sol_config = parse_solana_config();

    let (rpc, commitment) = if let Some(cli_rpc) = options.rpc {
        (cli_rpc.clone(), String::from("confirmed"))
    } else {
        if let Some(config) = sol_config {
            (config.json_rpc_url, config.commitment)
        } else {
            error!(
            "Could not find a valid Solana-CLI config file. Please specify a RPC manually with '-r' or set up your Solana-CLI config file."
        );
            std::process::exit(1);
        }
    };

    let commitment = CommitmentConfig::from_str(&commitment)?;
    let timeout = Duration::from_secs(options.timeout);

    let client = RpcClient::new_with_timeout_and_commitment(rpc.clone(), timeout, commitment);
    println!("RPC: {}", &rpc);
    println!("Timeout: {}", options.timeout);
    match options.command {
        Command::SPL { spl_subcommands } => process_spl(&client, spl_subcommands)?,
    };
    println!("FINISHED!");
    Ok(())
}
