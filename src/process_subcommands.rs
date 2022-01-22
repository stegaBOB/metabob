use anyhow::Result;
use solana_client::rpc_client::RpcClient;

use crate::opt::{Command};
use crate::spl::{get_spl_accounts, get_metadata, do_stuff};

pub fn process_spl(client: &RpcClient, full_json: bool, output: String) -> Result<()> {
        get_spl_accounts(&client, full_json, output)
}

pub fn process_get_metadata(client: &RpcClient, output: String) -> Result<()> {
        get_metadata(&client, output)
}

pub fn process_stuff(client: &RpcClient) -> Result<()> {
        do_stuff(&client)
}
