use zerocopy::{byteorder::U32, ByteOrder, Immutable, KnownLayout, TryFromBytes};

use crate::{header::str_from_null_terminated_utf8_safe, parse::TryParse};

#[derive(Clone, Debug, KnownLayout, Immutable, TryFromBytes)]
pub(crate) struct LexedLogHeader<E: ByteOrder> {
    // Size of disk block in bytes
    size: U32<E>,
    // Size of memory block in bytes
    memory_size: U32<E>,
    // Byte offset to the text
    text_offset: U32<E>,
    // Byte size of the binary area (immediately after logstc)
    binary_size: U32<E>,
    // Byte size of the disk area (immediately after logbins)
    disk_area: U32<E>,
    // Reserved, must be set to zero
    reserved: [u8; 44],
}

impl<E: ByteOrder> LexedLogHeader<E> {
    pub(super) fn binary_size(&self) -> usize {
        let binary_size: u32 = self.binary_size.into();
        binary_size as usize
    }
}

#[derive(Clone, Debug)]
pub(crate) struct LexedLogBlock<'data, E: ByteOrder> {
    pub(super) header: &'data LexedLogHeader<E>,
    pub(super) data: &'data [u8],
    pub(super) text: &'data [u8],
}

#[derive(Clone, Debug, KnownLayout, Immutable, TryFromBytes)]
pub(crate) struct LogHeader {
    // Size of disk block in bytes
    size: u32,
    // Size of memory block in bytes
    memory_size: u32,
    // Byte offset to the text
    text_offset: u32,
    // Byte size of the binary area (immediately after logstc)
    binary_size: u32,
    // Byte size of the disk area (immediately after logbins)
    disk_area: u32,
    // Reserved, must be set to zero
    reserved: [u8; 44],
}

#[derive(Clone, Debug, thiserror::Error, miette::Diagnostic)]
pub(crate) enum LogHeaderParseError {
    #[error("the reserved bytes were not set to zero")]
    NonZeroReservedBytes,
    #[error("the log block memory size was not a multiple of 4096: found {0}")]
    InvalidMemorySize(u32),
}

impl<E: ByteOrder> TryParse for LexedLogHeader<E> {
    type Parsed = LogHeader;
    type Error = LogHeaderParseError;
    fn try_parse(&self) -> Result<Self::Parsed, Self::Error> {
        if self.reserved.iter().any(|val| *val != 0) {
            return Err(LogHeaderParseError::NonZeroReservedBytes);
        }
        if self.memory_size % 4096 != 0 {
            return Err(LogHeaderParseError::InvalidMemorySize(
                self.memory_size.get(),
            ));
        }
        Ok(LogHeader {
            size: self.size.get(),
            memory_size: self.memory_size.get(),
            text_offset: self.text_offset.get(),
            binary_size: self.binary_size.get(),
            disk_area: self.disk_area.get(),
            reserved: self.reserved,
        })
    }
}

#[derive(Clone, Debug)]
pub(crate) struct LogBlock {
    pub(super) header: LogHeader,
    pub(super) data: Vec<u8>,
    pub(super) text: String,
}

impl<E: ByteOrder> TryParse for LexedLogBlock<'_, E> {
    type Error = LogHeaderParseError;
    type Parsed = LogBlock;
    fn try_parse(&self) -> Result<Self::Parsed, Self::Error> {
        Ok(LogBlock {
            header: self.header.try_parse()?,
            data: self.data.to_owned(),
            text: str_from_null_terminated_utf8_safe(self.text)
                .trim()
                .to_owned(),
        })
    }
}
