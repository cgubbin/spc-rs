use std::io::{BufReader, Read};

use camino::Utf8PathBuf;
use clap::Parser;
use fs_err::File;
use miette::{Context, IntoDiagnostic};

use spc_rs::{parse, write_spc};

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

            let source = BufReader::new(file)
                .bytes()
                .into_iter()
                .collect::<Result<Vec<_>, _>>()
                .unwrap();

            let parsed = parse(&source[..])?;
            dbg!(&parsed);

            write_spc(&args.file_path, parsed)?;
        }
        Err(err) => {
            eprintln!("Error: {}", err);
        }
    }
    Ok(())
}
