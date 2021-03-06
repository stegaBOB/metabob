use structopt::StructOpt;

#[derive(Debug, StructOpt)]
#[structopt(
    name = "Metabob",
    about = "The Metaplex NFT-standard assortment of tools for very specific tasks that are unrelated. Inspired HEAVILY by Metaboss."
)]
pub struct Opt {
    // RPC endpoint url
    #[structopt(short, long)]
    pub rpc: Option<String>,

    // Heavy RPC endpoint url
    #[structopt(long)]
    pub heavy_rpc: Option<String>,

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

    /// Stuff directly with Token Metadata
    #[structopt(name = "metadata")]
    Metadata {
        #[structopt(subcommand)]
        metadata_subcommands: MetadataSubcommands,
    },

    /// Stuff with gumdrop
    #[structopt(name = "gumdrop")]
    Gumdrop {
        #[structopt(subcommand)]
        gumdrop_subcommands: GumdropSubcommands,
    },
}

#[derive(Debug, StructOpt)]
pub enum SplSubcommands {
    /// Get all tokens mints for the SPL Token List
    #[structopt(name = "do_everything")]
    DoEverything {
        /// Don't save intermediate files
        #[structopt(short, long)]
        no_save: bool,
    },

    /// Get all fungible SPL token mints
    #[structopt(name = "get_mints")]
    GetMints {
        /// Don't save mint accounts to file
        #[structopt(short, long)]
        no_save: bool,
    },

    /// Get all metadata accounts
    #[structopt(name = "get_metadata")]
    GetMetadataAccounts {
        /// Don't save metadata accounts to file
        #[structopt(long)]
        no_save: bool,
    },

    /// Get SPL Token list json
    #[structopt(name = "get_token_list")]
    GetTokenList {
        /// Don't save accounts to file
        #[structopt(short, long)]
        no_save: bool,
    },

    /// Parse SPL Token list json
    #[structopt(name = "parse_token_list")]
    ParseTokenList {
        /// Don't save accounts to file
        #[structopt(long)]
        no_save: bool,
    },

    /// Do stuff?
    #[structopt(name = "do_stuff")]
    DoStuff,
}

#[derive(Debug, StructOpt)]
pub enum MetadataSubcommands {
    /// Signs ALL NFTs that contain the wallet address in the creator array
    #[structopt(name = "sign_all")]
    SignAll {
        /// Path to creator's keypair file
        #[structopt(short, long)]
        keypair: Option<String>,
    },

    /// Counts all NFTs that have the provided creator listed in the creator array
    #[structopt(name = "count_creators")]
    CountCreators {
        /// Base58 creator address
        #[structopt(short, long)]
        creator: String,
    },
}

#[derive(Debug, StructOpt)]
pub enum GumdropSubcommands {
    /// Makes a gumdrop claim list of random wallets
    #[structopt(name = "make_list")]
    MakeList {
        /// Number of wallets to make the list from
        #[structopt(short, long)]
        number: u64,

        /// Amount of tokens that each wallet gets
        #[structopt(short, long)]
        amount: u64,

        /// Number of times to make a list
        #[structopt(short, long)]
        repeat: Option<u64>,

        /// Path to file of base58 pubkeys to chuck in the gumdrop in array form
        #[structopt(short, long)]
        pubkey_path: Option<String>,
    },
}
