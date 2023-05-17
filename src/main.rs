mod cli;

use anyhow::{bail, Context};
use clap::Parser;
use cli::{Cli, Commands};
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
    if !shadow_dir.exists() {
        // It's not an error when there is no shadow directory
        return Ok(());
    }
    for entry in WalkDir::new(shadow_dir) {
        let e = entry?;
        if e.file_type().is_file() {
            f(&e)?;
        }
    }
    Ok(())
}

fn main() -> anyhow::Result<()> {
    let cli = Cli::parse();

    let base_dir = ensure_absolute_path(cli.base_dir.into())?;
    let shadow_dir_abs = ensure_absolute_path(cli.shadow_dir.into())?;
    let current_dir_shadow = shadow_dir_abs.join(current_dir()?.strip_prefix(base_dir)?);

    match &cli.command {
        Commands::Add { files } => add(files, &current_dir_shadow)?,
        Commands::Ls {} => ls(&current_dir_shadow)?,
        Commands::Restore { force } => restore(&current_dir_shadow, *force)?,
        Commands::Rm { files } => rm(files, &current_dir_shadow)?,
        Commands::GitClean {
            no_shdw,
            git_options,
        } => git_clean(git_options, no_shdw, current_dir_shadow)?,
    }
    Ok(())
}

fn ensure_absolute_path(shadow_dir: PathBuf) -> Result<PathBuf, anyhow::Error> {
    Ok(if shadow_dir.is_absolute() {
        shadow_dir
    } else {
        current_dir()?.join(shadow_dir)
    })
}

fn add(files: &Vec<String>, current_dir_shadow: &Path) -> Result<(), anyhow::Error> {
    for f in files {
        let shadow = current_dir_shadow.join(f);
        let src = PathBuf::from(f);
        if !src.exists() {
            bail!("{} does not exist", src.display());
        }
        if src.is_symlink() {
            bail!("{} is already a symlink", src.display());
        }
        let parent = shadow.parent().unwrap();
        create_dir_all(parent)
            .with_context(|| format!("Creating directory {}", parent.display()))?;
        let srcc = src
            .canonicalize()
            .with_context(|| format!("Canonicalizing {}", src.display()))?;
        fs::rename(&src, &shadow) // FIXME: Don't require same filesystem
            .with_context(|| format!("Renaming {} to {}", src.display(), shadow.display()))?;
        symlink(make_path_relative_to(&shadow, srcc.parent().unwrap()), srcc)?;
    }
    Ok(())
}

fn ls(current_dir_shadow: &Path) -> Result<(), anyhow::Error> {
    walk_shadow_files(current_dir_shadow, |e| {
        println!("{}", e.path().strip_prefix(current_dir_shadow)?.display());
        Ok(())
    })
}

fn restore(current_dir_shadow: &std::path::PathBuf, force: bool) -> Result<(), anyhow::Error> {
    walk_shadow_files(current_dir_shadow, |e| {
        let link = e.path().strip_prefix(current_dir_shadow)?;
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
        symlink(
            make_path_relative_to(
                e.path(),
                ensure_absolute_path(link.into())?.parent().unwrap(),
            ),
            link,
        )
        .with_context(|| format!("Creating link `{}`", link.display()))?;
        Ok(())
    })
}

fn rm(files: &Vec<String>, current_dir_shadow: &Path) -> Result<(), anyhow::Error> {
    for f in files {
        let shadow = current_dir_shadow.join(f);
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
    }
    Ok(())
}

fn git_clean(
    git_options: &Vec<String>,
    no_shdw: &bool,
    current_dir_shadow: PathBuf,
) -> Result<(), anyhow::Error> {
    let status = std::process::Command::new("git")
        .arg("clean")
        .args(git_options)
        .status()
        .with_context(|| format!("Running git clean {}", git_options.join(" ")))?;
    if !status.success() {
        bail!(status);
    }
    if !no_shdw {
        let mut has_shaddow = false;
        walk_shadow_files(&current_dir_shadow, |_| {
            has_shaddow = true;
            Ok(())
        })?;
        if has_shaddow {
            eprintln!("Restoring shdw files. Use --no-shdw (as the first argument) to skip this.");
            restore(&current_dir_shadow, false)?;
        }
    }
    Ok(())
}
