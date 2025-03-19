// Parser for SPC Files.
//
// This module contains utility methods for parsing items out of the byte stream.

use crate::{
    block::{Block, BlockParseError, BlockParser},
    header::{Header, HeaderParseError, HeaderParser},
    log::{LogBlock, LogBlockParseError, LogBlockParser},
};

pub(crate) struct SPCFile<'de> {
    whole: &'de [u8],
    rest: &'de [u8],
    format: Endian,
    pub(crate) byte: usize,
}

#[derive(Debug, Copy, Clone)]
pub(crate) enum Endian {
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
    pub(crate) fn new(input: &'de [u8]) -> Self {
        Self {
            whole: input,
            rest: input,
            byte: 0,
            format: Endian::Little,
        }
    }

    pub(crate) fn set_endian(&mut self, format: Endian) {
        self.format = format;
    }

    pub(crate) fn is_exhausted(&self) -> bool {
        self.rest.is_empty()
    }

    pub(crate) fn current_byte(&self) -> usize {
        self.byte
    }

    pub fn goto(&mut self, byte: usize) {
        self.byte = byte;
        self.rest = &self.whole[byte..];
    }

    pub(crate) fn read_byte(&mut self) -> Option<u8> {
        if self.rest.is_empty() {
            return None;
        }
        let byte = self.rest[0];
        self.byte += 1;
        self.rest = &self.rest[1..];
        Some(byte)
    }

    pub(crate) fn read_i8(&mut self) -> Option<i8> {
        self.read_byte().map(|v| v as i8)
    }

    pub(crate) fn read_u16(&mut self) -> Option<u16> {
        let bytes = [self.read_byte()?, self.read_byte()?];
        Some(match self.format {
            Endian::Little => u16::from_le_bytes(bytes),
            Endian::Big => u16::from_be_bytes(bytes),
        })
    }

    pub(crate) fn read_i16(&mut self) -> Option<i16> {
        let bytes = [self.read_byte()?, self.read_byte()?];
        Some(match self.format {
            Endian::Little => i16::from_le_bytes(bytes),
            Endian::Big => i16::from_be_bytes(bytes),
        })
    }

    pub(crate) fn read_u32(&mut self) -> Option<u32> {
        let bytes = [
            self.read_byte()?,
            self.read_byte()?,
            self.read_byte()?,
            self.read_byte()?,
        ];
        Some(match self.format {
            Endian::Little => u32::from_le_bytes(bytes),
            Endian::Big => u32::from_be_bytes(bytes),
        })
    }

    pub(crate) fn read_i32(&mut self) -> Option<i32> {
        let bytes = [
            self.read_byte()?,
            self.read_byte()?,
            self.read_byte()?,
            self.read_byte()?,
        ];
        Some(match self.format {
            Endian::Little => i32::from_le_bytes(bytes),
            Endian::Big => i32::from_be_bytes(bytes),
        })
    }

    pub(crate) fn read_f32(&mut self) -> Option<f32> {
        let bytes = [
            self.read_byte()?,
            self.read_byte()?,
            self.read_byte()?,
            self.read_byte()?,
        ];
        Some(match self.format {
            Endian::Little => f32::from_le_bytes(bytes),
            Endian::Big => f32::from_be_bytes(bytes),
        })
    }

    pub(crate) fn read_f64(&mut self) -> Option<f64> {
        let bytes = [
            self.read_byte()?,
            self.read_byte()?,
            self.read_byte()?,
            self.read_byte()?,
            self.read_byte()?,
            self.read_byte()?,
            self.read_byte()?,
            self.read_byte()?,
        ];
        Some(match self.format {
            Endian::Little => f64::from_le_bytes(bytes),
            Endian::Big => f64::from_be_bytes(bytes),
        })
    }

    pub(crate) fn read_utf8(&mut self, len: usize) -> Option<&'de str> {
        if self.rest.len() < len {
            return None;
        }
        let bytes = &self.rest[..len];
        self.goto(self.byte + len);
        Some(std::str::from_utf8(bytes).unwrap())
    }

    pub(crate) fn read_unescaped_utf8(&mut self, len: usize) -> Option<&'de str> {
        if self.rest.len() < len {
            return None;
        }
        let bytes = &self.rest[..len];
        self.goto(self.byte + len);

        Some(str_from_null_terminated_utf8_safe(bytes))
    }

    pub(crate) fn parse_header(&mut self) -> Result<Header, HeaderParseError> {
        let mut parser = HeaderParser::new(self)?;
        parser.parse()
    }

    pub(crate) fn parse_block(&mut self, header: &Header) -> Result<Block, BlockParseError> {
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

    pub(crate) fn parse_log(&mut self, log_offset: usize) -> Result<LogBlock, LogBlockParseError> {
        let mut parser = LogBlockParser(self);
        parser.parse(log_offset)
    }
}
