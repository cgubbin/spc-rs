#[allow(dead_code)]
use camino::Utf8Path;
use lex::SPCReader;
use miette::IntoDiagnostic;
use parse::ParsedSPC;

mod block;
mod header;
mod lex;
mod logblock;
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
    log::info!("lexing big-endian SPC file");
    assert!(matches!(source.get(1).copied().unwrap(), 0x4c));
    SPCReader::big_endian(source).lex()
}

pub fn lex_little_endian_spc(source: &'_ [u8]) -> miette::Result<LexedSPC<'_, LittleEndian>> {
    log::info!("lexing little-endian SPC file");
    assert!(matches!(source.get(1).copied().unwrap(), 0x4b | 0x4d));
    SPCReader::little_endian(source).lex()
}

#[cfg(test)]
mod test {
    use std::io::{BufReader, Cursor, Read};

    use fs_err::File;
    use miette::{Context, IntoDiagnostic};

    use crate::{
        parse,
        write::{CsvWriter, WriteSPC},
    };

    #[test]
    fn ftir_data_parse_matches_expected() -> miette::Result<()> {
        let expected = include_str!("/tmp/spc/test_data/Ft-ir.csv");

        let spc = "/tmp/spc/test_data/Ft-ir.spc";

        let file = File::open(spc)
            .into_diagnostic()
            .wrap_err_with(|| format!("opening '{}' failed", spc))?;

        let source = BufReader::new(file)
            .bytes()
            .collect::<Result<Vec<_>, _>>()
            .unwrap();

        let parsed = parse(&source[..])?;

        let writer = CsvWriter;
        let mut sink: Cursor<Vec<u8>> = Cursor::new(Vec::new());

        writer.write_spc(&mut sink, &parsed).unwrap();

        let output_vec: Vec<u8> = sink.into_inner();
        let output_string: String = String::from_utf8(output_vec).unwrap();

        for (expected, actual) in expected
            .split_terminator('\n')
            .zip(output_string.split_terminator('\n'))
        {
            assert_eq!(expected, actual);
        }

        Ok(())
    }
}
