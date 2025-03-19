use crate::SPCFile;

#[derive(Debug, thiserror::Error, miette::Diagnostic)]
pub(crate) enum SubHeaderParseError {
    #[error("Premature termination of binary input")]
    PrematureTermination,
}

#[derive(Clone, Debug)]
struct SubFlagParameters(u8);

#[derive(Clone, Debug)]
pub(crate) struct Subheader {
    parameters: SubFlagParameters,
    pub(crate) exponent_y: i8,
    index_number: u16,
    starting_z: f32,
    ending_z: f32,
    noise_value: f32,
    number_points: u32,
    number_co_added_scans: u32,
    w_axis_value: f32,
    reserved: String,
}

pub(crate) struct SubHeaderParser<'a, 'de>(pub(crate) &'a mut SPCFile<'de>);

impl<'a, 'de> SubHeaderParser<'a, 'de> {
    fn read_byte(&mut self) -> Result<u8, SubHeaderParseError> {
        self.0
            .read_byte()
            .ok_or(SubHeaderParseError::PrematureTermination)
    }

    fn read_i8(&mut self) -> Result<i8, SubHeaderParseError> {
        self.0
            .read_i8()
            .ok_or(SubHeaderParseError::PrematureTermination)
    }

    fn read_u16(&mut self) -> Result<u16, SubHeaderParseError> {
        self.0
            .read_u16()
            .ok_or(SubHeaderParseError::PrematureTermination)
    }

    fn read_u32(&mut self) -> Result<u32, SubHeaderParseError> {
        self.0
            .read_u32()
            .ok_or(SubHeaderParseError::PrematureTermination)
    }

    fn read_f32(&mut self) -> Result<f32, SubHeaderParseError> {
        self.0
            .read_f32()
            .ok_or(SubHeaderParseError::PrematureTermination)
    }

    fn read_unescaped_utf8(&mut self, size: usize) -> Result<&'de str, SubHeaderParseError> {
        self.0
            .read_unescaped_utf8(size)
            .ok_or(SubHeaderParseError::PrematureTermination)
    }

    pub(crate) fn parse(&mut self) -> Result<Subheader, SubHeaderParseError> {
        let parameters = SubFlagParameters(self.read_byte()?);
        let exponent_y = self.read_i8()?;
        let index_number = self.read_u16()?;
        let starting_z = self.read_f32()?;
        let ending_z = self.read_f32()?;
        let noise_value = self.read_f32()?;
        let number_points = self.read_u32()?;
        let number_co_added_scans = self.read_u32()?;
        let w_axis_value = self.read_f32()?;
        let reserved = self.read_unescaped_utf8(4)?.trim().to_owned();
        Ok(Subheader {
            parameters,
            exponent_y,
            index_number,
            starting_z,
            ending_z,
            noise_value,
            number_points,
            number_co_added_scans,
            w_axis_value,
            reserved,
        })
    }
}
