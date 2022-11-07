use clap::{Parser, Subcommand, ValueHint};

/// Manage symlinks to files in a shadow directory
///
///
#[derive(Parser)]
#[command(author, version, about, long_about = None)]
#[command(propagate_version = true)]
pub struct Cli {
    #[command(subcommand)]
    pub command: Commands,
}

#[derive(Subcommand)]
pub enum Commands {
    /// Move given files to the shadow directory and create symlinks
    /// to them in their original location.
    Add {
        #[arg(required(true), value_hint = ValueHint::FilePath)]
        files: Vec<String>,
    },
    /// List files in the shadow directory.
    Ls {},
    /// (Re)create symlinks to all files in the shadow directory.
    Restore {
        /// Remove existing files in place of created symlinks
        #[arg(short, long)]
        force: bool,
    },
    /// Move given files out of the shadow directory to the place of
    /// their symlinks at or under the current directory.
    Rm {
        #[arg(required(true), value_hint = ValueHint::FilePath)]
        files: Vec<String>,
    },
}
