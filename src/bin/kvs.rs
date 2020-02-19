use structopt::StructOpt;

use kvs::Result;

#[derive(Debug, StructOpt)]
#[structopt(name = "kvs", about = "A command-line key-value store client")]
struct Opt {
    /// Activate debug mode
    #[structopt(short, long)]
    debug: bool,
    /// Verbose mode (-v, -vv, -vvv, etc.)
    #[structopt(short, long, parse(from_occurrences))]
    verbose: u8,
    #[structopt(subcommand)]
    cmd: Cmd,
}

#[derive(Debug, StructOpt)]
enum Cmd {
    /// Set the value of a string key to a string
    Set {
        #[structopt(name = "KEY")]
        key: String,
        #[structopt(name = "VALUE")]
        value: String,
    },
    /// Get the string value of a given string key
    Get {
        #[structopt(name = "KEY")]
        key: String,
    },
    /// Remove a given key
    Rm {
        #[structopt(name = "KEY")]
        key: String,
    },
}

fn main() -> Result<()> {
    let opt = Opt::from_args();
    match opt.cmd {
        Cmd::Get { .. } => panic!("unimplemented"),
        Cmd::Set { .. } => panic!("unimplemented"),
        Cmd::Rm { .. } => panic!("unimplemented"),
    }
}
