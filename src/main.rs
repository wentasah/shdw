mod cli;

use anyhow::{bail, Context};
use clap::Parser;
use cli::{Cli, Commands};
use dirs::home_dir;
use std::env::current_dir;
use std::fs::remove_file;
use std::{
    fs::{self, create_dir_all},
    path::Path,
};
use walkdir::{DirEntry, WalkDir};

fn walk_shadow_files<F>(shadow_dir: &Path, f: F) -> anyhow::Result<()>
where
    F: Fn(&DirEntry) -> anyhow::Result<()>,
{
    for entry in WalkDir::new(&shadow_dir) {
        let e = entry?;
        if e.file_type().is_file() {
            f(&e)?;
        }
    }
    Ok(())
}

fn main() -> anyhow::Result<()> {
    let cli = Cli::parse();

    let home_dir = home_dir().expect("home directory should be defined");
    let env_base = home_dir.join(Path::new("nix/conf/env"));
    let shadow_dir = env_base.join(current_dir()?.strip_prefix(home_dir)?);

    match &cli.command {
        Commands::Add { files } => {
            for f in files {
                let shadow = shadow_dir.join(f);
                create_dir_all(shadow.parent().unwrap())?;
                fs::rename(f, &shadow)?; // FIXME: Don't require same filesystem
                std::os::unix::fs::symlink(&shadow, f)?;
            }
        }
        Commands::Ls {} => walk_shadow_files(&shadow_dir, |e| {
            println!("{}", e.path().strip_prefix(&shadow_dir)?.display());
            Ok(())
        })?,
        Commands::Restore { force } => restore(&shadow_dir, *force)?,
        Commands::Rm { files } => {
            for f in files {
                let shadow = shadow_dir.join(f);
                let f = Path::new(f);
                if !shadow.is_file() {
                    bail!("`{}` is not a file", shadow.display());
                }
                if f.is_symlink() && f.is_file() && fs::read_link(f)? == shadow {
                    remove_file(f).with_context(|| format!("Removing `{}`", f.display()))?;
                    fs::rename(&shadow, f).with_context(|| {
                        format!("Moving `{}` to `{}`", shadow.display(), f.display())
                    })?;
                } else {
                    bail!(
                        "Not removing `{}` because it's not a symlink pointing to file `{}`",
                        f.display(),
                        shadow.display()
                    );
                }
            }
        }
    }
    Ok(())
}

fn restore(shadow_dir: &std::path::PathBuf, force: bool) -> Result<(), anyhow::Error> {
    walk_shadow_files(shadow_dir, |e| {
        let link = e.path().strip_prefix(shadow_dir)?;
        if let Some(parent) = link.parent() {
            if !parent.as_os_str().is_empty() && !parent.exists() {
                fs::create_dir(parent)
                    .with_context(|| format!("Creating directory `{}`", parent.display()))?;
            }
        }
        if link.exists() {
            if link.is_symlink() && fs::read_link(link)? == e.path() {
                return Ok(());
            }
            if force {
                remove_file(link).with_context(|| format!("Removing `{}`", link.display()))?;
            } else {
                bail!("File exists: `{}`", link.display());
            }
        }
        std::os::unix::fs::symlink(e.path(), link)
            .with_context(|| format!("Creating link `{}`", link.display()))?;
        Ok(())
    })
}
