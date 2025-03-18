use crate::{
    header::{DataShape, NewFormatHeader, OldFormatHeader, SubHeaderParser},
    SPCFile,
};

mod variables;

use variables::{FromTo, MeasurementXYVariables};

#[derive(Clone, Debug)]
pub(crate) struct Block(Vec<MeasurementXYVariables>);

pub(crate) struct BlockParser<'a, 'de>(pub(crate) &'a mut SPCFile<'de>);

impl<'a, 'de> BlockParser<'a, 'de> {
    pub(crate) fn parse_old_block(&mut self, header: &OldFormatHeader) -> miette::Result<Block> {
        let x_points = FromTo {
            from: header.starting_x as f64,
            to: header.ending_x as f64,
            length: header.number_points as usize,
        };

        let mut spectra = Vec::new();

        while !self.0.is_exhausted() {
            let mut subheader = SubHeaderParser(self.0).parse()?;

            if subheader.exponent_y == 0 {
                subheader.exponent_y = header.exponent_y as i8;
            }

            let y = self.get_old_y(
                x_points.length,
                subheader.exponent_y,
                header.flags.y_precision_is_16_bit(),
            );

            let data = MeasurementXYVariables::new(x_points.values(), y, header);
            spectra.push(data);
        }

        Ok(Block(spectra))
    }

    fn get_old_y(&mut self, length: usize, exponent_y: i8, y_16_bit_precision: bool) -> Vec<f64> {
        let factor = 2f64.powi(exponent_y as i32 - if y_16_bit_precision { 16 } else { 32 });

        (0..length)
            .map(|_| {
                if y_16_bit_precision {
                    self.0.read_i16() as f64 * factor
                } else {
                    (((self.0.read_byte() as u64) << 16)
                        + ((self.0.read_byte() as u64) << 24)
                        + ((self.0.read_byte() as u64) << 0)
                        + ((self.0.read_byte() as u64) << 8)) as f64
                        * factor
                }
            })
            .collect()
    }

    pub(crate) fn parse_new_block(&mut self, header: &NewFormatHeader) -> miette::Result<Block> {
        let datashape = header.flags.data_shape();

        let x_points: Vec<f64> = match datashape {
            // For these shapes, x-data comes before the subheader
            DataShape::XY | DataShape::XYY => (0..header.number_points)
                .map(|_| self.0.read_f32())
                .map(|each| each.into())
                .collect(),
            // No x-axis, so we create it
            DataShape::Y | DataShape::YY => {
                let x = FromTo {
                    from: header.starting_x as f64,
                    to: header.ending_x as f64,
                    length: header.number_points as usize,
                };
                x.values()
            }
            // In XYXY x-data is provided after each subheader
            DataShape::XYXY => {
                return self.parse_xyxy(header);
            }
        };

        let mut spectra = Vec::new();

        for i in 0..header.spectra {
            let mut subheader = SubHeaderParser(self.0).parse()?;
            if subheader.exponent_y == 0 {
                subheader.exponent_y = header.exponent_y as i8;
            }

            let y = self.get_new_y(
                x_points.len(),
                subheader.exponent_y,
                header.flags.y_precision_is_16_bit(),
            );

            // TODO: Interface and one function
            let data = MeasurementXYVariables::new_new(x_points.clone(), y, header);

            spectra.push(data);
        }

        Ok(Block(spectra))
    }

    pub(crate) fn parse_xyxy(&mut self, header: &NewFormatHeader) -> miette::Result<Block> {
        let mut spectra = Vec::new();

        for _ in 0..header.spectra {
            let mut subheader = SubHeaderParser(self.0).parse()?;
            if subheader.exponent_y == 0 {
                subheader.exponent_y = header.exponent_y as i8;
            }

            let x_points = (0..header.number_points)
                .map(|_| self.0.read_f32())
                .map(|each| each.into())
                .collect::<Vec<f64>>();

            let y = self.get_new_y(
                x_points.len(),
                subheader.exponent_y,
                header.flags.y_precision_is_16_bit(),
            );

            // TODO: Interface and one function
            let data = MeasurementXYVariables::new_new(x_points, y, header);

            spectra.push(data);
        }

        Ok(Block(spectra))
    }

    fn get_new_y(&mut self, length: usize, exponent_y: i8, y_16_bit_precision: bool) -> Vec<f64> {
        let factor = 2f64.powi(exponent_y as i32 - if y_16_bit_precision { 16 } else { 32 });

        (0..length)
            .map(|_| {
                if y_16_bit_precision {
                    self.0.read_i16() as f64 * factor
                } else {
                    if exponent_y != -128 {
                        self.0.read_i32() as f64 * factor
                    } else {
                        self.0.read_f32() as f64
                    }
                }
            })
            .collect()
    }
}
