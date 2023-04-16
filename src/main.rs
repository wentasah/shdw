mod cli;

use anyhow::{bail, Context};
use clap::Parser;
use cli::{Cli, Commands};
use dirs::home_dir;
use std::env::current_dir;
use std::fs::remove_file;
use std::os::unix::fs::symlink;
use std::path::{Component, PathBuf};
use std::{
    fs::{self, create_dir_all},
    path::Path,
};
use walkdir::{DirEntry, WalkDir};

/// Converts absolute `path` to be relative to absolute `to` path.
// From: https://github.com/uutils/coreutils/blob/084510e499ce8a8d9e7da96731e33f671070baab/src/uucore/src/lib/features/fs.rs#L562
// License: MIT
pub fn make_path_relative_to<P1: AsRef<Path>, P2: AsRef<Path>>(path: P1, to: P2) -> PathBuf {
    let path = path.as_ref();
    let to = to.as_ref();
    let common_prefix_size = path
        .components()
        .zip(to.components())
        .take_while(|(first, second)| first == second)
        .count();
    let path_suffix = path
        .components()
        .skip(common_prefix_size)
        .map(|x| x.as_os_str());
    let mut components: Vec<_> = to
        .components()
        .skip(common_prefix_size)
        .map(|_| Component::ParentDir.as_os_str())
        .chain(path_suffix)
        .collect();
    if components.is_empty() {
        components.push(Component::CurDir.as_os_str());
    }
    components.iter().collect()
}

fn walk_shadow_files<F>(shadow_dir: &Path, mut f: F) -> anyhow::Result<()>
where
    F: FnMut(&DirEntry) -> anyhow::Result<()>,
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
    let shadow_base = home_dir.join(Path::new("nix/conf/env")).canonicalize()?;
    let shadow_dir = shadow_base.join(current_dir()?.strip_prefix(home_dir)?);

    match &cli.command {
        Commands::Add { files } => add(files, &shadow_dir)?,
        Commands::Ls {} => ls(&shadow_dir)?,
        Commands::Restore { force } => restore(&shadow_dir, *force)?,
        Commands::Rm { files } => rm(files, &shadow_dir)?,
        Commands::GitClean {
            no_shdw,
            git_options,
        } => git_clean(git_options, no_shdw, shadow_dir)?,
    }
    Ok(())
}

fn add(files: &Vec<String>, shadow_dir: &PathBuf) -> Result<(), anyhow::Error> {
    Ok(for f in files {
        let shadow = shadow_dir.join(f);
        let src = PathBuf::from(f);
        if src.is_symlink() {
            bail!("{} is already a symlink", f);
        }
        let parent = shadow.parent().unwrap();
        create_dir_all(parent)
            .with_context(|| format!("Creating directory {}", parent.display()))?;
        let srcc = src
            .canonicalize()
            .with_context(|| format!("Canonicalizing {}", src.display()))?;
        fs::rename(f, &shadow) // FIXME: Don't require same filesystem
            .with_context(|| format!("Renaming {} to {}", f, shadow.display()))?;
        symlink(make_path_relative_to(&shadow, srcc.parent().unwrap()), srcc)?;
    })
}

fn ls(shadow_dir: &PathBuf) -> Result<(), anyhow::Error> {
    walk_shadow_files(shadow_dir, |e| {
        println!("{}", e.path().strip_prefix(&shadow_dir)?.display());
        Ok(())
    })
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
            if link.is_symlink() && fs::read_link(link)?.canonicalize()? == e.path() {
                return Ok(());
            }
            if force {
                remove_file(link).with_context(|| format!("Removing `{}`", link.display()))?;
            } else {
                bail!("File exists: `{}`", link.display());
            }
        }
        symlink(e.path(), link).with_context(|| format!("Creating link `{}`", link.display()))?;
        Ok(())
    })
}

fn rm(files: &Vec<String>, shadow_dir: &PathBuf) -> Result<(), anyhow::Error> {
    Ok(for f in files {
        let shadow = shadow_dir.join(f);
        let f = Path::new(f);
        if !shadow.is_file() {
            bail!("`{}` is not a file", shadow.display());
        }
        if f.is_symlink() && f.is_file() {
            let dst = fs::read_link(f)?;
            if dst
                .canonicalize()
                .with_context(|| format!("Canonicalizing {}", dst.display()))?
                == shadow
                    .canonicalize()
                    .with_context(|| format!("Canonicalizing {}", shadow.display()))?
            {
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
    })
}

fn git_clean(
    git_options: &Vec<String>,
    no_shdw: &bool,
    shadow_dir: PathBuf,
) -> Result<(), anyhow::Error> {
    let status = std::process::Command::new("git")
        .arg("clean")
        .args(git_options)
        .status()
        .with_context(|| format!("Running git clean {}", git_options.join(" ")))?;
    if !status.success() {
        bail!(status);
    }
    Ok(if !no_shdw {
        let mut has_shaddow = false;
        walk_shadow_files(&shadow_dir, |_| {
            has_shaddow = true;
            Ok(())
        })?;
        if has_shaddow {
            eprintln!("Restoring shdw files. Use --no-shdw (as the first argument) to skip this.");
            restore(&shadow_dir, false)?;
        }
    })
}
