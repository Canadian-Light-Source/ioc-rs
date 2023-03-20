use clap::{Parser, Subcommand};

// CLI =================================================
#[derive(Parser)]
#[command(name = "ioc")]
#[command(author = "Niko Kivel <niko.kivel@lightsource.ca>")]
#[command(version = "0.1.0")]
#[command(about = "install ioc definitions", long_about = None)]
pub struct Cli {
    // /// Path to the deployment destination
    // #[arg(short, long)]
    // path: Option<String>,

    /// Path to the tempalte directory
    #[arg(short, long)]
    pub template_dir: Option<String>,

    /// The name of the command
    #[command(subcommand)]
    pub command: Option<Commands>,
}

#[derive(Subcommand, Debug)]
pub enum Commands {
    /// delpoyment command
    Install {
        /// perform dryrun
        #[arg(short, long, action)]
        dryrun: bool,
        /// force install
        #[arg(short, long, action)]
        force: bool,
        /// list of IOCs to deploy, comma separated
        // #[clap(short, long, default_value = "./", value_parser, num_args = 1.., value_delimiter = ' ')]
        #[clap(default_value = "", value_parser, num_args = 1.., value_delimiter = ' ')]
        // #[clap(value_parser, num_args = 0.., value_delimiter = ' ')]
        iocs: Option<Vec<String>>,
    },
}
