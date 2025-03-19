use crate::SPCFile;

#[derive(Debug, thiserror::Error, miette::Diagnostic)]
pub(crate) enum LogBlockParseError {
    #[error("Premature termination of binary input")]
    PrematureTermination,
}

pub(crate) struct LogBlockParser<'a, 'de>(pub(crate) &'a mut SPCFile<'de>);

#[derive(Clone, Debug)]
pub(crate) struct LogBlock {
    header: LogHeader,
    data: String,
    text: String,
}

#[derive(Clone, Debug)]
struct LogHeader {
    size: u32,
    memory_size: u32,
    text_offset: u32,
    binary_size: u32,
    disk_area: u32,
    reserved: String,
}

impl<'a, 'de> LogBlockParser<'a, 'de> {
    fn read_u32(&mut self) -> Result<u32, LogBlockParseError> {
        self.0
            .read_u32()
            .ok_or(LogBlockParseError::PrematureTermination)
    }

    fn read_unescaped_utf8(&mut self, size: usize) -> Result<&'de str, LogBlockParseError> {
        self.0
            .read_unescaped_utf8(size)
            .ok_or(LogBlockParseError::PrematureTermination)
    }

    pub(crate) fn parse(&mut self, log_offset: usize) -> Result<LogBlock, LogBlockParseError> {
        let header = LogHeader {
            size: self.read_u32()?,
            memory_size: self.read_u32()?,
            text_offset: self.read_u32()?,
            binary_size: self.read_u32()?,
            disk_area: self.read_u32()?,
            reserved: self.read_unescaped_utf8(44)?.trim().to_string(),
        };

        let log_data = self
            .read_unescaped_utf8(header.binary_size as usize)?
            .to_string();
        self.0.goto(log_offset + header.text_offset as usize);
        let log_ascii = self
            .read_unescaped_utf8(header.size as usize - header.text_offset as usize)?
            .trim()
            .to_string();

        Ok(LogBlock {
            header,
            data: log_data,
            text: log_ascii,
        })
    }
}
