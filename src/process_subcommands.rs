use crate::{
    opt::SplSubcommands,
    spl::{
        do_everything, do_stuff, get_metadata_accounts, get_token_entries,
        parse_token_uri, get_mint_accounts,
    },
};
use anyhow::Result;
use solana_client::rpc_client::RpcClient;

pub fn process_spl(client: &RpcClient, heavy_client: &RpcClient, subcommands: SplSubcommands) -> Result<()> {
    match subcommands {
        SplSubcommands::DoEverything { no_save } => {
            do_everything(client, heavy_client, no_save)?;
        }
        SplSubcommands::GetMints { no_save } => {
            get_mint_accounts(client, no_save)?;
        }
        SplSubcommands::GetMetadataAccounts { no_save} => {
            get_metadata_accounts(client, None, no_save )?;
        }
        SplSubcommands::GetTokenList { no_save } => {
            get_token_entries(None, no_save)?;
        }
        SplSubcommands::ParseTokenList { no_save } => {
            parse_token_uri(None, no_save)?;
        }
        SplSubcommands::DoStuff => {
            do_stuff(client)?;
        }
    }

    Ok(())
}
