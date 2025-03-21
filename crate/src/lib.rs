use std::io::Read;

use block::{Block, BlockParser};
use camino::Utf8Path;
use miette::IntoDiagnostic;

mod block;
mod header;
mod log;
mod parse;
pub(crate) mod units;
mod write;

use header::{Header, NewFormatHeader, OldFormatHeader};
use log::{LogBlock, LogBlockParser};
use parse::SPCFile;
use units::{xzwType, yType, InstrumentTechnique};
use write::{CsvWriter, WriteSPC};

#[derive(Clone, Debug)]
pub enum ParsedSPC {
    Old {
        header: OldFormatHeader,
        block: Block,
    },
    New {
        header: NewFormatHeader,
        block: Block,
        log: Option<LogBlock>,
    },
}

pub fn write_spc(input_path: &Utf8Path, parsed: ParsedSPC) -> miette::Result<()> {
    let output_path = input_path.with_extension("csv");
    let writer = CsvWriter;

    let mut file_handle = fs_err::OpenOptions::new()
        .write(true)
        .create(true)
        .open(output_path)
        .into_diagnostic()?;

    writer
        .write_spc(&mut file_handle, &parsed)
        .into_diagnostic()
}

pub fn parse<R: Read>(input: R) -> miette::Result<ParsedSPC> {
    let input = input
        .bytes()
        .collect::<Result<Vec<u8>, _>>()
        .into_diagnostic()?;

    let mut spc = SPCFile::new(&input);

    let header = spc.parse_header()?;
    let block = spc.parse_block(&header)?;

    let parsed = match header {
        Header::Old(header) => ParsedSPC::Old { header, block },
        Header::New(header) => {
            let log = if header.log_offset != 0 {
                Some(spc.parse_log(header.log_offset as usize)?)
            } else {
                None
            };

            ParsedSPC::New { header, block, log }
        }
    };

    Ok(parsed)
}
