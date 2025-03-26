use zerocopy::{BigEndian, ByteOrder, Immutable, KnownLayout, LittleEndian, TryFromBytes};

use crate::{
    block::{LexedBlock, LexedDirectory, LexedSubfile, LexedXData, YMode},
    header::{
        DataShape, FlagParameters, LexedHeader, LexedNewFormatHeader, LexedOldFormatHeader,
        LexedSubheader, Precision,
    },
    logblock::{LexedLogBlock, LexedLogHeader},
    parse::{ParseError, ParsedSPC, TryParse},
};

#[derive(Clone, Debug)]
pub struct LexedSPC<'data, E: ByteOrder> {
    header: LexedHeader<'data, E>,
    block: LexedBlock<'data, E>,
    log: Option<LexedLogBlock<'data, E>>,
}

impl<E: ByteOrder> TryParse for LexedSPC<'_, E> {
    type Parsed = ParsedSPC;
    type Error = ParseError;
    fn try_parse(&self) -> Result<Self::Parsed, Self::Error> {
        Ok(ParsedSPC {
            header: self.header.try_parse()?,
            block: self.block.try_parse()?,
            log: self.log.as_ref().map(|log| log.try_parse()).transpose()?,
        })
    }
}

#[derive(Copy, Clone, Debug)]
pub(crate) enum Version {
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
        self.rest.is_empty()
    }

    fn remaining_bytes(&self) -> usize {
        self.rest.len()
    }

    fn read_byte_slice(&mut self, len: usize) -> miette::Result<&'data [u8]> {
        if len > self.rest.len() {
            return Err(miette::miette!(
                "Not enough bytes left in the buffer. requested {}, remaining {}",
                len,
                self.rest.len()
            ));
        }
        let slice = &self.rest[..len];
        self.rest = &self.rest[len..];
        self.byte += len;
        Ok(slice)
    }

    fn lex_header(&mut self) -> miette::Result<LexedHeader<'data, E>> {
        let header_len = match self.version {
            Version::Old => 224,
            Version::New => 512,
        };
        let header = &self.read_byte_slice(header_len)?;

        let header = match self.version {
            Version::Old => {
                log::info!("lexing old format header");
                let header = LexedOldFormatHeader::try_ref_from_bytes(header).unwrap();
                LexedHeader::Old(header)
            }
            Version::New => {
                log::info!("lexing new format header");
                let header = LexedNewFormatHeader::try_ref_from_bytes(header).unwrap();
                LexedHeader::New(header)
            }
        };
        Ok(header)
    }

    fn lex_subheader(&mut self) -> miette::Result<&'data LexedSubheader<E>> {
        let source = self.read_byte_slice(32)?;
        Ok(LexedSubheader::try_ref_from_bytes(source).unwrap())
    }

    // Lex X-data from the input
    //
    // X-data is always stored as a contiguous list of 32-bit floating point values.
    fn lex_x(&mut self, num_points: usize) -> miette::Result<LexedXData<'data, E>> {
        let data = self.read_byte_slice(num_points * Precision::ThirtyTwoBit.bytes_per_point())?;
        LexedXData::new(data)
    }

    fn lex_subfile(
        &mut self,
        y_mode: YMode,
        num_points: usize,
    ) -> miette::Result<LexedSubfile<'data, E>> {
        log::info!("lexing subfile containing {} points", num_points);
        let subheader = self.lex_subheader()?;

        // Check to see if the subfile overrides the header data type
        let float_expected_from_subheader = subheader.float_data_expected();
        let mode = match (y_mode, float_expected_from_subheader) {
            (YMode::SixteenBitInt, false) => YMode::SixteenBitInt,
            (YMode::ThirtyTwoBitInt(m), false) => YMode::ThirtyTwoBitInt(m),
            (_, true) => YMode::IEEEFloat,
            (_, false) => panic!("Inconsistent data types expected"),
        };
        let data = self.read_byte_slice(num_points * mode.bytes_per_point())?;
        LexedSubfile::new(subheader, data, mode)
    }

    fn lex_subfiles(
        &mut self,
        header: &LexedHeader<'data, E>,
    ) -> miette::Result<Vec<LexedSubfile<'data, E>>> {
        // A new-style header stores the number of subfiles in the `fnsub` field, if this
        // is provided we just use it.
        let num_subfiles = if let Some(num_subfiles) = header.number_of_subfiles() {
            num_subfiles
        // If not we have to try and work out the number of subfiles present in the data.
        } else {
            todo!()
        };

        let mut subfiles = Vec::new();
        for _ in 0..num_subfiles {
            let subfile = self.lex_subfile(header.y_mode(), header.number_points())?;
            subfiles.push(subfile);
        }
        Ok(subfiles)
    }

    fn lex_xyxy_blocks(
        &mut self,
        header: &LexedHeader<'data, E>,
    ) -> miette::Result<Vec<(LexedXData<'data, E>, LexedSubfile<'data, E>)>> {
        // Only new style headers can be XYXY format, and the number_of_subfiles method always
        // returns Some for a new style header. This means we can unwrap safely.
        let num_subfiles = header.number_of_subfiles().unwrap();

        let mut subfiles = Vec::new();
        for _ in 0..num_subfiles {
            let subheader = self.lex_subheader()?;
            let x_data = self.lex_x(subheader.number_of_points())?;

            let header_mode = header.y_mode();
            let float_expected_from_subheader = subheader.float_data_expected();

            let mode = match (header_mode, float_expected_from_subheader) {
                (YMode::SixteenBitInt, false) => YMode::SixteenBitInt,
                (YMode::ThirtyTwoBitInt(m), false) => YMode::ThirtyTwoBitInt(m),
                (_, true) => YMode::IEEEFloat,
                _ => panic!("Invalid mode for XYXY data"),
            };

            let data =
                self.read_byte_slice(subheader.number_of_points() * mode.bytes_per_point())?;

            subfiles.push((x_data, LexedSubfile::new(subheader, data, mode)?));
        }
        Ok(subfiles)
    }

    fn lex_block(
        &mut self,
        header: &LexedHeader<'data, E>,
    ) -> miette::Result<LexedBlock<'data, E>> {
        let block = match header.data_shape() {
            // If the DataShape is Y, after the header the file consists of a single subfile
            // containing the y-data points
            DataShape::Y => {
                log::info!("lexing Y block");
                LexedBlock::Y(self.lex_subfile(header.y_mode(), header.number_points())?)
            }
            // If the DataShape is YY, after the header the file consists of multiple subfiles,
            // containing the y-data points for each measurement.
            DataShape::YY => {
                log::info!("lexing YY block");
                LexedBlock::YY(self.lex_subfiles(header)?)
            }
            // If the DataShape is XY, after the header the file consists of a the x-data points,
            // followed by a single subfile containing the y-data points
            DataShape::XY => {
                log::info!("lexing XY block");
                LexedBlock::XY {
                    x: self.lex_x(header.number_points())?,
                    y: self.lex_subfile(header.y_mode(), header.number_points())?,
                }
            }
            // If the datashape is XYY then the header is followed by the (shared) x-data points,
            // then multiple subfiles containing the different sets of y-data points
            DataShape::XYY => {
                log::info!("lexing XYY block");
                LexedBlock::XYY {
                    x: self.lex_x(header.number_points())?,
                    ys: self.lex_subfiles(header)?,
                }
            }
            // If the datashape is XYXY then the header is followed by a number of subfiles, each
            // consisting of a header, followed by the x-data points and the y-data points
            DataShape::XYXY => {
                log::info!("lexing XYXY block");
                let data = self.lex_xyxy_blocks(header)?;

                // XYXY data can be optionally followed by a directory structure, containing
                // information about the individual subfiles
                let directory = match header.log_offset() {
                    // If there is a log, and the reader is already at the log position there is no
                    // directory
                    Some(n) if n == self.byte => None,
                    // If there is no log, and the buffer is exhausted there is no directory data
                    None if self.is_exhausted() => None,
                    // If there is no log, and the buffer is not exhausted then it must contain the
                    // directory data
                    None => {
                        assert_eq!(self.remaining_bytes(), data.len() * 12);
                        Some(
                            (0..data.len())
                                .map(|_| self.read_byte_slice(12))
                                .collect::<Result<Vec<_>, _>>()?
                                .into_iter()
                                .map(|source| LexedDirectory::try_ref_from_bytes(source))
                                .collect::<Result<Vec<_>, _>>()
                                .unwrap(),
                        )
                    }
                    // If there is a log, and the buffer is not at the log position the gap must
                    // contain the directory data
                    Some(n) => {
                        let bytes_to_log = n - self.byte;
                        assert_eq!(bytes_to_log, data.len() * 12);

                        Some(
                            (0..data.len())
                                .map(|_| self.read_byte_slice(12))
                                .collect::<Result<Vec<_>, _>>()?
                                .into_iter()
                                .map(|source| LexedDirectory::try_ref_from_bytes(source))
                                .collect::<Result<Vec<_>, _>>()
                                .unwrap(),
                        )
                    }
                };
                LexedBlock::XYXY { data, directory }
            }
        };

        // Check we read enough
        match header.log_offset() {
            // If there is no log, then we should have read the whole file
            None => assert!(self.is_exhausted()),
            // And if there is a log it should be next in the buffer
            Some(log_offset) => assert_eq!(self.byte, log_offset),
        }

        Ok(block)
    }

    // This assumes the current byte is equal to the log-offset, and that the stream is not
    // exhausted. This should be checked by the caller
    fn lex_log(&mut self) -> miette::Result<LexedLogBlock<'data, E>> {
        // The log header is 64 bytes
        let source = self.read_byte_slice(64)?;
        let header = LexedLogHeader::try_ref_from_bytes(source).unwrap();

        // The log data is immediately after the header
        let data = &self.rest[..header.binary_size()];
        // And the text block is the remainder?
        let text = self.rest;

        // So at this point the rest is always empty

        self.rest = &[];

        Ok(LexedLogBlock { header, data, text })
    }

    pub(super) fn lex(&mut self) -> miette::Result<LexedSPC<'data, E>> {
        // Lex the header, but don't parse yet
        log::info!("lexing header");
        let header = self.lex_header()?;

        log::info!("lexing block");
        let block = self.lex_block(&header)?;

        log::info!("lexing log block");
        let log = if self.is_exhausted() {
            None
        } else {
            Some(self.lex_log()?)
        };

        Ok(LexedSPC { header, block, log })
    }
}
