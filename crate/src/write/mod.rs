use std::io::Write;

use csv::WriterBuilder;
use serde::Serialize;

use crate::{
    block::{Block, YData},
    header::{NewFormatHeader, OldFormatHeader},
    log::LogBlock,
    ParsedSPC,
};

pub trait WriteSPC {
    type Error;
    fn write_spc<W: Write>(&self, writer: &mut W, spc: &ParsedSPC) -> Result<(), Self::Error>;
}

pub(crate) struct CsvWriter;

impl WriteSPC for CsvWriter {
    type Error = csv::Error;
    fn write_spc<W: Write>(&self, writer: &mut W, spc: &ParsedSPC) -> Result<(), Self::Error> {
        let mut writer = WriterBuilder::new()
            .has_headers(true)
            .comment(Some(b'#'))
            .from_writer(writer);

        #[derive(Debug, Serialize)]
        struct Record {
            x: f64,
            y: f64,
        }

        let exponent = spc.header.exponent_y();

        if exponent == -128 {
            panic!("no impl for float data storage");
        }

        // For Y-data the exponent in the subheader is ignored, and that in the header is used
        // instead
        if let Block::Y(block) = &spc.block {
            let step = (spc.header.ending_x() - spc.header.starting_x())
                / ((spc.header.number_points() - 1) as f64);

            let x = (0..spc.header.number_points())
                .map(|i| spc.header.starting_x() + i as f64 * step)
                .collect::<Vec<_>>();
            if let YData::ThirtyTwoBitInteger(data) = &block.data {
                let factor = 2f64.powi(exponent - 32);
                for (x, each) in x.iter().zip(data) {
                    let val = factor * *each as f64;

                    dbg!(&val);

                    let record = Record { x: *x, y: val };
                    writer.serialize(record)?;
                }
            }
        }

        writer.flush()?;
        Ok(())
    }
}
