use std::io::Write;

use csv::WriterBuilder;
use serde::Serialize;

use crate::{
    block::Block,
    header::{NewFormatHeader, OldFormatHeader},
    log::LogBlock,
    ParsedSPC,
};

pub(crate) trait WriteSPC {
    type Error;
    fn write_spc<W: Write>(&self, writer: &mut W, spc: &ParsedSPC) -> Result<(), Self::Error> {
        match spc {
            ParsedSPC::Old { header, block } => self.write_old_format(writer, header, block),
            ParsedSPC::New { header, block, log } => {
                self.write_new_format(writer, header, block, log.as_ref())
            }
        }
    }
    fn write_old_format<W: Write>(
        &self,
        writer: &mut W,
        header: &OldFormatHeader,
        block: &Block,
    ) -> Result<(), Self::Error>;
    fn write_new_format<W: Write>(
        &self,
        writer: &mut W,
        header: &NewFormatHeader,
        block: &Block,
        log: Option<&LogBlock>,
    ) -> Result<(), Self::Error>;
}

pub(crate) struct CsvWriter;

impl WriteSPC for CsvWriter {
    type Error = csv::Error;
    fn write_old_format<W: Write>(
        &self,
        writer: &mut W,
        header: &OldFormatHeader,
        block: &Block,
    ) -> Result<(), Self::Error> {
        let mut writer = WriterBuilder::new()
            .has_headers(true)
            .comment(Some(b'#'))
            .from_writer(writer);

        #[derive(Debug, Serialize)]
        struct Record {
            x: f64,
            y: f64,
        }

        for each_variable_set in block.iter() {
            for (x, y) in each_variable_set.iter() {
                let record = Record { x: *x, y: *y };
                writer.serialize(record)?;
            }
        }

        writer.flush()?;
        Ok(())
    }
    fn write_new_format<W: Write>(
        &self,
        writer: &mut W,
        header: &NewFormatHeader,
        block: &Block,
        log: Option<&LogBlock>,
    ) -> Result<(), Self::Error> {
        todo!()
    }
}
