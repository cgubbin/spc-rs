#[allow(dead_code)]
use camino::Utf8Path;
use lex::SPCReader;
use miette::IntoDiagnostic;
use parse::ParsedSPC;

mod block;
mod header;
mod lex;
mod log;
mod parse;
pub(crate) mod units;
mod write;

use lex::LexedSPC;
use parse::TryParse;
use units::{
    xzwType, xzwTypeCreationError, yType, yTypeCreationError, InstrumentTechnique,
    InstrumentTechniqueCreationError,
};
use write::{CsvWriter, WriteSPC};
use zerocopy::{BigEndian, LittleEndian};

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

pub fn parse(source: &'_ [u8]) -> miette::Result<ParsedSPC> {
    Ok(match source.get(1).copied() {
        Some(0x4c) => lex_big_endian_spc(source)?.try_parse(),
        Some(0x4b) | Some(0x4d) => lex_little_endian_spc(source)?.try_parse(),
        Some(b) => panic!("impossible file type descriptor {b}"),
        None => panic!("file contained less than two bytes"),
    }?)
}

pub fn lex_big_endian_spc(source: &'_ [u8]) -> miette::Result<LexedSPC<'_, BigEndian>> {
    assert!(matches!(source.get(1).copied().unwrap(), 0x4c));
    SPCReader::big_endian(source).lex()
}

pub fn lex_little_endian_spc(source: &'_ [u8]) -> miette::Result<LexedSPC<'_, LittleEndian>> {
    assert!(matches!(source.get(1).copied().unwrap(), 0x4b | 0x4d));
    SPCReader::little_endian(source).lex()
}
