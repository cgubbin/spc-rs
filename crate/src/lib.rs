use std::io::Read;

use block::{Block, BlockParser};
use miette::IntoDiagnostic;

mod block;
mod header;
mod log;
pub(crate) mod units;

use header::{Header, HeaderParser, NewFormatHeader, OldFormatHeader};
use log::{LogBlock, LogBlockParser};
use units::{xzwType, yType, ExperimentSettings};

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

    dbg!(&parsed);

    Ok(parsed)
}

pub(crate) struct SPCFile<'de> {
    whole: &'de [u8],
    rest: &'de [u8],
    format: Endian,
    byte: usize,
}

#[derive(Debug, Copy, Clone)]
enum Endian {
    Little,
    Big,
}

fn str_from_null_terminated_utf8_safe(s: &[u8]) -> &str {
    if s.iter().any(|&x| x == 0) {
        unsafe { str_from_null_terminated_utf8(s) }
    } else {
        ::std::str::from_utf8(s).unwrap()
    }
}

// unsafe: s must contain a null byte
unsafe fn str_from_null_terminated_utf8(s: &[u8]) -> &str {
    ::std::ffi::CStr::from_ptr(s.as_ptr() as *const _)
        .to_str()
        .unwrap()
}

// unsafe: s must contain a null byte, and be valid utf-8
unsafe fn str_from_null_terminated_utf8_unchecked(s: &[u8]) -> &str {
    ::std::str::from_utf8_unchecked(::std::ffi::CStr::from_ptr(s.as_ptr() as *const _).to_bytes())
}

impl<'de> SPCFile<'de> {
    pub fn new(input: &'de [u8]) -> Self {
        Self {
            whole: input,
            rest: input,
            byte: 0,
            format: Endian::Little,
        }
    }

    fn is_exhausted(&self) -> bool {
        self.rest.is_empty()
    }

    pub(crate) fn current_byte(&self) -> usize {
        self.byte
    }

    pub fn goto(&mut self, byte: usize) {
        self.byte = byte;
        self.rest = &self.whole[byte..];
    }

    fn read_byte(&mut self) -> u8 {
        let byte = self.rest[0];
        self.byte += 1;
        self.rest = &self.rest[1..];
        byte
    }

    fn read_i8(&mut self) -> i8 {
        self.read_byte() as i8
    }

    fn read_u16(&mut self) -> u16 {
        let bytes = [self.read_byte(), self.read_byte()];
        match self.format {
            Endian::Little => u16::from_le_bytes(bytes),
            Endian::Big => u16::from_be_bytes(bytes),
        }
    }

    fn read_i16(&mut self) -> i16 {
        let bytes = [self.read_byte(), self.read_byte()];
        match self.format {
            Endian::Little => i16::from_le_bytes(bytes),
            Endian::Big => i16::from_be_bytes(bytes),
        }
    }

    fn read_u32(&mut self) -> u32 {
        let bytes = [
            self.read_byte(),
            self.read_byte(),
            self.read_byte(),
            self.read_byte(),
        ];
        match self.format {
            Endian::Little => u32::from_le_bytes(bytes),
            Endian::Big => u32::from_be_bytes(bytes),
        }
    }

    fn read_i32(&mut self) -> i32 {
        let bytes = [
            self.read_byte(),
            self.read_byte(),
            self.read_byte(),
            self.read_byte(),
        ];
        match self.format {
            Endian::Little => i32::from_le_bytes(bytes),
            Endian::Big => i32::from_be_bytes(bytes),
        }
    }

    fn read_f32(&mut self) -> f32 {
        let bytes = [
            self.read_byte(),
            self.read_byte(),
            self.read_byte(),
            self.read_byte(),
        ];
        match self.format {
            Endian::Little => f32::from_le_bytes(bytes),
            Endian::Big => f32::from_be_bytes(bytes),
        }
    }

    fn read_f64(&mut self) -> f64 {
        let bytes = [
            self.read_byte(),
            self.read_byte(),
            self.read_byte(),
            self.read_byte(),
            self.read_byte(),
            self.read_byte(),
            self.read_byte(),
            self.read_byte(),
        ];
        match self.format {
            Endian::Little => f64::from_le_bytes(bytes),
            Endian::Big => f64::from_be_bytes(bytes),
        }
    }

    fn read_utf8(&mut self, len: usize) -> &'de str {
        let bytes = &self.rest[..len];
        self.goto(self.byte + len);
        std::str::from_utf8(bytes).unwrap()
    }

    fn read_unescaped_utf8(&mut self, len: usize) -> &'de str {
        let bytes = &self.rest[..len];
        self.goto(self.byte + len);

        str_from_null_terminated_utf8_safe(bytes)
    }

    pub fn parse_header(&mut self) -> miette::Result<Header> {
        let mut parser = HeaderParser::new(self);
        parser.parse()
    }

    pub fn parse_block(&mut self, header: &Header) -> miette::Result<Block> {
        let mut parser = BlockParser(self);
        match header {
            Header::Old(header) => {
                let block = parser.parse_old_block(header)?;
                Ok(block)
            }
            Header::New(header) => {
                let block = parser.parse_new_block(header)?;
                Ok(block)
            }
        }
    }

    pub fn parse_log(&mut self, log_offset: usize) -> miette::Result<LogBlock> {
        let mut parser = LogBlockParser(self);
        parser.parse(log_offset)
    }
}
