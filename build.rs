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

    let man = Path::new(&outdir).join("shdw.1");
    Man::new(cmd.clone())
        .render(&mut File::create(&man).unwrap())
        .unwrap();
    println!("cargo:warning=man page generated: {:?}", man);

    for shell in Shell::value_variants() {
        let path = generate_to(*shell, &mut cmd, "shdw", &outdir)?;
        println!(
            "cargo:warning={} completion file is generated: {:?}",
            shell, path
        );
    }

    Ok(())
}
