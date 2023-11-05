use std::path::PathBuf;

use clap::{Parser, Subcommand};

#[derive(Parser, Debug)]
#[command(author, version, about, long_about = None)]
pub struct Args {
    #[command(subcommand)]
    pub command: Command,
}

#[derive(Subcommand, Debug)]
#[clap(rename_all = "snake_case")]
pub enum Command {
    Decode {
        value: String,
    },
    Info {
        filename: PathBuf,
    },
    Peers {
        filename: PathBuf,
    },
    Handshake {
        filename: PathBuf,
        peer: String,
    },
    DownloadPiece {
        #[arg(short)]
        output: PathBuf,
        torrent: String,
        piece_index: usize,
    },
    Download {
        #[arg(short)]
        output: PathBuf,
        torrent: String,
    },
}
