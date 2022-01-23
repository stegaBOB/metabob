use crate::{
    opt::SplSubcommands,
    spl::{do_everything, do_stuff, get_mint_accounts, get_token_entries, parse_mint_accounts},
};
use anyhow::Result;
use solana_client::rpc_client::RpcClient;

pub fn process_spl(client: &RpcClient, subcommands: SplSubcommands) -> Result<()> {
    match subcommands {
        SplSubcommands::DoEverything => {
            do_everything(client)?;
        }
        SplSubcommands::GetMints { no_save } => {
            get_mint_accounts(client, no_save)?;
        }
        SplSubcommands::ProcessMints { no_save } => {
            parse_mint_accounts(client, None, no_save)?;
        }
        SplSubcommands::GetTokenList => {
            get_token_entries(&client, None)?;
        }
        SplSubcommands::DoStuff => {
            do_stuff(client)?;
        }
    }

    Ok(())
}
