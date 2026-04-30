use clap::Parser;
use std::path::PathBuf;

#[derive(Parser, Debug)]
#[command(author, version, about, long_about = None)]
pub struct Args {
    #[arg(long)]
    pub i: PathBuf,

    #[arg(long)]
    pub o: PathBuf,
}
