use structopt::StructOpt;

#[derive(Debug, StructOpt)]
#[structopt(
    name = "Metabob",
    about = "The Metaplex NFT-standard assortment of tools for very specific tasks that are unrelated. Inspired by Metaboss"
)]
pub struct Opt {
    // RPC endpoint url
    #[structopt(short, long)]
    pub rpc: Option<String>,

    /// Timeout to override default value of 60 seconds
    #[structopt(short, long, default_value = "60")]
    pub timeout: u64,

    #[structopt(subcommand)]
    pub command: Command,
}

#[derive(Debug, StructOpt)]
pub enum Command {
    /// Stuff with SPL mints and Metadata
    #[structopt(name = "spl")]
    SPL {
        #[structopt(subcommand)]
        spl_subcommands: SplSubcommands,
    },
}

#[derive(Debug, StructOpt)]
pub enum SplSubcommands {
    /// Get all tokens mints for the SPL Token List
    #[structopt(name = "do_everything")]
    DoEverything,

    /// Get all fungible SPL token mints
    #[structopt(name = "get_mints")]
    GetMints {
        /// Don't save mint accounts to file
        #[structopt(short, long)]
        no_save: bool,
    },

    /// Parse mint accounts into account struct
    #[structopt(name = "parse_mints")]
    ProcessMints {
        /// Don't save accounts to file
        #[structopt(short, long)]
        no_save: bool,
    },

    /// Get SPL Token list json
    #[structopt(name = "get_token_list")]
    GetTokenList,

    /// Do stuff?
    #[structopt(name = "do_stuff")]
    DoStuff,
}
