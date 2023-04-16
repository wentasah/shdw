use clap::{Parser, Subcommand, ValueHint};
use std::path::PathBuf;

fn home_dir() -> PathBuf {
    dirs::home_dir().expect("home directory should be defined")
}

/// Manage symlinks to files in a shadow directory
///
///
#[derive(Parser)]
#[command(author, version, about, long_about = None)]
#[command(propagate_version = true)]
pub struct Cli {
    #[command(subcommand)]
    pub command: Commands,
    /// Shadow directory to use
    #[arg(long, env("SHDW_DIR"), value_hint = ValueHint::DirPath, default_value_t = home_dir().join(PathBuf::from(".config/shdw/dir")).display().to_string())]
    pub shadow_dir: String,
    /// Base directory
    #[arg(long, env("SHDW_BASE_DIR"), value_hint = ValueHint::DirPath, default_value_t = home_dir().display().to_string())]
    pub base_dir: String,
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
    /// Run 'git clean' with given GIT_OPTIONS followed by 'shdw restore'.
    GitClean {
        /// Don't run 'shdw restore' after 'git clean'.
        #[arg(long)]
        no_shdw: bool,
        /// git-clean options
        #[arg(trailing_var_arg = true, allow_hyphen_values = true)]
        git_options: Vec<String>,
    },
}
