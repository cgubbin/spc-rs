use camino::Utf8PathBuf;
use clap::Parser;
use fs_err::File;
use miette::{Context, IntoDiagnostic};

use spc_rs::parse;

#[derive(Debug, Parser)]
struct Args {
    file_path: Utf8PathBuf,
}

fn main() -> miette::Result<()> {
    match Args::try_parse() {
        Ok(args) => {
            let file = File::open(&args.file_path)
                .into_diagnostic()
                .wrap_err_with(|| format!("opening '{}' failed", args.file_path))?;
            let parsed = parse(file)?;
            // write_spc(&args.file_path, parsed)?;
        }
        Err(err) => {
            eprintln!("Error: {}", err);
        }
    }
    Ok(())
}
