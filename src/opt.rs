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
    /// Get SPL Token Mints
    #[structopt(name = "spl")]
    SPL {
        #[structopt(short, long)]
        full_json: bool,

        #[structopt(short, long, default_value = "./output.json")]
        output: String,
    },

    /// Get metadata stuff from file
    #[structopt(name = "get-metadata")]
    GET_METADATA {
        #[structopt(short, long, default_value = "./better_output.json")]
        output: String,
    },

    /// Process stuff
    #[structopt(name = "process-stuff")]
    PROCESS_STUFF,
}
