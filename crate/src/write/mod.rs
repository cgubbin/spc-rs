use std::io::Write;

use csv::WriterBuilder;
use serde::Serialize;

use crate::{block::Block, ParsedSPC};

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

        match &spc.block {
            // For Y-data the exponent in the subheader is ignored, and that in the header is used
            // instead
            Block::Y(block) => {
                let x = spc.header.x_points();
                let y = block.data.decode(exponent);
                for (x, y) in x.into_iter().zip(y) {
                    let record = Record { x, y };
                    writer.serialize(record)?;
                }
            }
            // For XY-data the exponent in the subheader is ignored, and that in the header is used
            // instead
            Block::XY { x, y } => {
                let y = y.data.decode(exponent);
                for (x, y) in x.iter().zip(y) {
                    let record = Record { x: (*x).into(), y };
                    writer.serialize(record)?;
                }
            }
            // For YY-data is the exponent in the subheader ignored, and that in the header used
            // instead? It's not clear again...
            Block::YY(ys) => {
                let x = spc.header.x_points();
                let ys: Vec<_> = ys.iter().map(|each| each.data.decode(exponent)).collect();

                assert!(ys.iter().all(|each| each.len() == x.len()));

                for (ii, x) in x.iter().enumerate() {
                    let record: Vec<f64> = std::iter::once(*x)
                        .chain(ys.iter().map(|each| each[ii]))
                        .collect::<Vec<_>>();

                    writer.serialize(record)?;
                }
            }
            Block::XYY { x, ys } => {
                let ys: Vec<_> = ys.iter().map(|each| each.data.decode(exponent)).collect();

                assert!(ys.iter().all(|each| each.len() == x.len()));

                for (ii, x) in x.iter().enumerate() {
                    let record: Vec<f64> = std::iter::once(*x as f64)
                        .chain(ys.iter().map(|each| each[ii]))
                        .collect::<Vec<_>>();

                    writer.serialize(record)?;
                }
            }
            Block::XYXY { data, directory } => {
                for (x, y) in data {
                    let z = y.subheader.z;
                    writer.write_record(&[format!("# z = {z}")])?;
                    let y = y.data.decode(exponent);
                    for (x, y) in x.iter().zip(y) {
                        let record = Record { x: (*x).into(), y };
                        writer.serialize(record)?;
                    }
                }
            }
        }

        writer.flush()?;
        Ok(())
    }
}
