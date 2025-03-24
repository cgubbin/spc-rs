use std::io::{Bytes, Read};

// use block::{Block, BlockParser};
use camino::Utf8Path;
use header::{DataShape, Header, NewFormatHeader, OldFormatHeader, Precision, Subheader};
use miette::IntoDiagnostic;

// mod block;
mod header;
// mod log;
// mod parse;
pub(crate) mod units;
// mod write;

// use header::{Header, NewFormatHeader, OldFormatHeader};
// use log::{LogBlock, LogBlockParser};
// use parse::SPCFile;
use units::{
    xzwType, xzwTypeCreationError, yType, yTypeCreationError, InstrumentTechnique,
    InstrumentTechniqueCreationError,
};
use zerocopy::{BigEndian, ByteOrder, LittleEndian, TryFromBytes};
// use write::{CsvWriter, WriteSPC};
//
// #[derive(Clone, Debug)]
// pub enum ParsedSPC {
//     Old {
//         header: OldFormatHeader,
//         block: Block,
//     },
//     New {
//         header: NewFormatHeader,
//         block: Block,
//         log: Option<LogBlock>,
//     },
// }
//
// pub fn write_spc(input_path: &Utf8Path, parsed: ParsedSPC) -> miette::Result<()> {
//     let output_path = input_path.with_extension("csv");
//     let writer = CsvWriter;
//
//     let mut file_handle = fs_err::OpenOptions::new()
//         .write(true)
//         .create(true)
//         .open(output_path)
//         .into_diagnostic()?;
//
//     writer
//         .write_spc(&mut file_handle, &parsed)
//         .into_diagnostic()
// }
//
// pub fn parse<R: Read>(input: R) -> miette::Result<ParsedSPC> {
//     let input = input
//         .bytes()
//         .collect::<Result<Vec<u8>, _>>()
//         .into_diagnostic()?;
//
//     let mut spc = SPCFile::new(&input);
//
//     let header = spc.parse_header()?;
//     let block = spc.parse_block(&header)?;
//
//     let parsed = match header {
//         Header::Old(header) => ParsedSPC::Old { header, block },
//         Header::New(header) => {
//             let log = if header.log_offset != 0 {
//                 Some(spc.parse_log(header.log_offset as usize)?)
//             } else {
//                 None
//             };
//
//             ParsedSPC::New { header, block, log }
//         }
//     };
//
//     Ok(parsed)
// }
//
//

#[derive(Debug)]
enum Version {
    Old,
    New,
}

#[derive(Debug)]
pub(crate) struct SPCReader<'data, E: ByteOrder> {
    // header: &'data [u8],
    whole: &'data [u8],
    rest: &'data [u8],
    pub(crate) byte: usize,
    version: Version,
    byte_order: std::marker::PhantomData<E>,
}

impl<'data> SPCReader<'data, BigEndian> {
    pub(crate) fn big_endian(input: &'data [u8]) -> Self {
        let version = match input.get(1).copied().unwrap() {
            0x4c => Version::New,
            0x4d => Version::Old,
            _ => panic!("Invalid SPC file for big endian ordering"),
        };
        Self {
            whole: input,
            rest: input,
            byte: 0,
            version,
            byte_order: std::marker::PhantomData,
        }
    }
}

impl<'data> SPCReader<'data, LittleEndian> {
    pub(crate) fn little_endian(input: &'data [u8]) -> Self {
        let version = match input.get(1).copied().unwrap() {
            0x4b => Version::New,
            0x4d => Version::Old,
            _ => panic!("Invalid SPC file for little endian ordering"),
        };
        Self {
            whole: input,
            rest: input,
            byte: 0,
            version,
            byte_order: std::marker::PhantomData,
        }
    }
}

impl<'data, E: ByteOrder> SPCReader<'data, E> {
    fn is_exhausted(&self) -> bool {
        dbg!(&self.whole.len(), &self.rest.len());
        dbg!(&self.whole.len() - &self.rest.len());
        self.rest.is_empty()
    }

    fn remaining_bytes(&self) -> usize {
        self.rest.len()
    }

    fn read_byte_slice(&mut self, len: usize) -> &'data [u8] {
        let slice = &self.rest[..len];
        self.rest = &self.rest[len..];
        self.byte += len;
        slice
    }

    fn lex_header(&mut self) -> miette::Result<Header<'data, E>> {
        let header_len = match self.version {
            Version::Old => 224,
            Version::New => 512,
        };
        let header = &self.read_byte_slice(header_len);

        let header = match self.version {
            Version::Old => {
                let header = OldFormatHeader::try_ref_from_bytes(header).unwrap();
                Header::Old(header)
            }
            Version::New => {
                let header = NewFormatHeader::try_ref_from_bytes(header).unwrap();
                Header::New(header)
            }
        };
        Ok(header)
    }

    fn lex_subheader(&mut self) -> miette::Result<&'data Subheader<E>> {
        let source = self.read_byte_slice(32);
        Ok(Subheader::try_ref_from_bytes(source).unwrap())
    }

    fn lex_x(&mut self, num_points: usize) -> miette::Result<XData<'data>> {
        let data = self.read_byte_slice(num_points * Precision::ThirtyTwoBit.bytes_per_point());
        Ok(XData { data })
    }

    fn lex_subfile(
        &mut self,
        y_precision: Precision,
        num_points: usize,
    ) -> miette::Result<Subfile<'data, E>> {
        let subheader = self.lex_subheader()?;
        let data = self.read_byte_slice(num_points * y_precision.bytes_per_point());
        Ok(Subfile { subheader, data })
    }
}

struct XData<'data> {
    data: &'data [u8],
}

struct Subfile<'data, E: ByteOrder> {
    subheader: &'data Subheader<E>,
    data: &'data [u8],
}

pub fn parse<R: Read>(input: R) -> miette::Result<()> {
    let input = input
        .bytes()
        .collect::<Result<Vec<u8>, _>>()
        .into_diagnostic()?;

    match input.get(1).copied().unwrap() {
        0x4c => {
            let mut reader = SPCReader::big_endian(&input);

            // Lex the header, but don't parse yet
            let header = reader.lex_header()?;

            // Get the expected data shape from the flags byte in the header.
            match header.data_shape() {
                // For Y data there is one and only one subfile, so we can just lex it directly
                // from the input
                DataShape::Y => {
                    let subfile =
                        reader.lex_subfile(header.y_precision(), header.number_points())?;
                    // If there is a log offset, we need to lex the log data as well and the reader
                    // will not be exhausted.
                    assert!(reader.is_exhausted());
                }
                // For YY data there are an unknown number of subfiles. We can work out how many
                // there are using the log offset
                DataShape::YY => match header.log_offset() {
                    None => {
                        let subfile_length =
                            header.y_precision().bytes_per_point() * header.number_points();
                        let num_subfiles = reader.remaining_bytes() / subfile_length;

                        let mut subfiles = Vec::new();
                        for _ in 0..num_subfiles {
                            let subfile =
                                reader.lex_subfile(header.y_precision(), header.number_points())?;
                            subfiles.push(subfile);
                        }
                        // Now we are at the end of the file, as we lexed every subfile and there
                        // is no log block
                        assert!(reader.is_exhausted());
                    }
                    Some(log_offset) => {
                        let mut subfiles = Vec::new();
                        while reader.byte < log_offset {
                            let subfile =
                                reader.lex_subfile(header.y_precision(), header.number_points())?;
                            subfiles.push(subfile);
                        }
                        dbg!(log_offset, reader.byte);
                    }
                },

                // For Y data there is one and only one subfile, so we can just lex it directly
                // from the input
                DataShape::XY => {
                    let x_data = reader.lex_x(header.number_points())?;
                    let subfile =
                        reader.lex_subfile(header.y_precision(), header.number_points())?;
                    // If there is a log offset, we need to lex the log data as well and the reader
                    // will not be exhausted.
                    assert!(reader.is_exhausted());
                }
                // For Y data there is one and only one subfile, so we can just lex it directly
                // from the input
                DataShape::XYY => {
                    let x_data = reader.lex_x(header.number_points())?;
                    let y_data = match header.log_offset() {
                        None => {
                            let subfile_length =
                                header.y_precision().bytes_per_point() * header.number_points();
                            let num_subfiles = reader.remaining_bytes() / subfile_length;

                            let mut subfiles = Vec::new();
                            for _ in 0..num_subfiles {
                                let subfile = reader
                                    .lex_subfile(header.y_precision(), header.number_points())?;
                                subfiles.push(subfile);
                            }
                            // Now we are at the end of the file, as we lexed every subfile and there
                            // is no log block
                            assert!(reader.is_exhausted());
                        }
                        Some(log_offset) => {
                            let mut subfiles = Vec::new();
                            while reader.byte < log_offset {
                                let subfile = reader
                                    .lex_subfile(header.y_precision(), header.number_points())?;
                                subfiles.push(subfile);
                            }
                            dbg!(log_offset, reader.byte);
                        }
                    };
                }
                _ => unimplemented!(),
            }
        }
        0x4b | 0x4d => {
            let mut reader = SPCReader::little_endian(&input);
            let header = reader.lex_header()?;

            // Get the expected data shape from the flags byte in the header.
            match header.data_shape() {
                DataShape::Y => {
                    let subfile =
                        reader.lex_subfile(header.y_precision(), header.number_points())?;
                    assert!(reader.is_exhausted());
                }
                DataShape::YY => {}
                _ => unimplemented!(),
            }
        }
        _ => panic!("Invalid SPC file"),
    }

    Ok(())

    //
    //
    // let header = spc.parse_header()?;
    // let block = spc.parse_block(&header)?;
    //
    // let parsed = match header {
    //     Header::Old(header) => ParsedSPC::Old { header, block },
    //     Header::New(header) => {
    //         let log = if header.log_offset != 0 {
    //             Some(spc.parse_log(header.log_offset as usize)?)
    //         } else {
    //             None
    //         };
    //
    //         ParsedSPC::New { header, block, log }
    //     }
    // };
    //
    // Ok(parsed)
}
