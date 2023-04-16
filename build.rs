use clap::{CommandFactory, ValueEnum};
use clap_complete::{generate_to, Shell};
use clap_mangen::Man;
use std::env;
use std::fs::File;
use std::io::Error;
use std::path::Path;

include!("src/cli.rs");

fn main() -> Result<(), Error> {
    let outdir = match env::var_os("OUT_DIR") {
        None => return Ok(()),
        Some(outdir) => outdir,
    };

    let mut cmd = Cli::command();
    render_man(&outdir, &cmd, None);
    for subcommand in cmd.get_subcommands() {
        render_man(&outdir, &cmd, Some(subcommand));
    }

    for shell in Shell::value_variants() {
        let _path = generate_to(*shell, &mut cmd, "shdw", &outdir)?;
        // println!(
        //     "cargo:warning={} completion file is generated: {:?}",
        //     shell, path
        // );
    }

    Ok(())
}

fn render_man(outdir: &std::ffi::OsString, cmd: &clap::Command, subcmd: Option<&clap::Command>) {
    let title = match subcmd {
        Some(subcmd) => format!("{} {}", cmd.get_name(), subcmd.get_name()),
        None => format!("{}", cmd.get_name()),
    };
    let fname = match subcmd {
        Some(subcmd) => format!("{}-{}.1", cmd.get_name(), subcmd.get_name()),
        None => format!("{}.1", cmd.get_name()),
    };
    let man = Man::new(subcmd.unwrap_or(cmd).clone()).title(title);
    let path = Path::new(outdir).join(fname);
    man.render(&mut File::create(&path).unwrap()).unwrap();
    // println!("cargo:warning=man page generated: {:?}", path);
}
