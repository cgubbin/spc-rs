use scroll::{ctx, Endian};
use zerocopy::TryFromBytes;

/// A subfile-header preceeds an individual trace in a multi-type file. For evenly spaced files
/// the subtime (z) and subnext (next_z) are ignored for all except the first subfile. In this case
/// the spacing defined by the first subfile determines the z-spacing for all files.
///
/// In ordered and random multi-files subnext normally matches subtime but for all types the
/// subindx must be correct. The [`Subheader`] is always required even if there is only one
/// subfile, but if TMULTI is not set in the main header the exponent set in the subheader is
/// ignored in favour of that in the header.
use crate::SPCFile;

#[derive(Debug, thiserror::Error, miette::Diagnostic)]
pub(crate) enum SubHeaderParseError {
    #[error("Premature termination of binary input")]
    PrematureTermination,
    #[error("The reserved fields were not set to zero")]
    ReservedFieldsNotZero,
    #[error("The subheader flags should only have bits 0, 3, and 7 set but found: {0}")]
    SubheaderFlags(u8),
    #[error("Error in underlying scroll")]
    Scroll(#[from] scroll::Error),
}

/// [`SubFlagParameters`] are stored in the first byte of the [`Subheader`]
///
/// Only 3 of the 8-bits are in use. From least to most significant these are
/// - Bit 0: Indicates if the subfile changed
/// - Bit 3: Indicates if the peak table file should not be used
/// - Bit 7: Indicates if the subfile was modified by arithmetic
#[repr(C)]
#[derive(Clone, Debug, TryFromBytes)]
struct SubFlagParameters(u8);

impl<'a> ctx::TryFromCtx<'a, Endian> for SubFlagParameters {
    type Error = SubHeaderParseError;
    fn try_from_ctx(bytes: &'a [u8], _: Endian) -> Result<(Self, usize), Self::Error> {
        if (bytes[0] & 0b1000_1001) != bytes[0] {
            return Err(SubHeaderParseError::SubheaderFlags(bytes[0]));
        }
        Ok((Self(bytes[0]), 1))
    }
}

#[cfg(test)]
mod testsub {
    use super::SubFlagParameters;
    use scroll::{Endian, Pread};
    #[test]
    fn subflag_parameters() {
        let data = vec![0b1000_1001];
        let src = &data;
        let offset = &mut 0;
        let subheader_params = src.gread::<SubFlagParameters>(offset).unwrap();
        dbg!(&subheader_params);
    }
}

// impl TryFromBytes for SubFlagParameters {
//     type Error = zerocopy::FromBytesError;
//     fn try_from_bytes(bytes: &[u8]) -> Result<Self, Self::Error> {
//         Ok(Self(bytes[0]))
//     }
// }

#[repr(C)]
#[derive(Clone, Debug, TryFromBytes)]
pub(crate) struct Subheader {
    parameters: SubFlagParameters,
    /// The exponent of the Y axis for the sub-file
    ///
    /// If the exponent is equal to 80h, then the values are to be interpreted directly as floating
    /// point data. If not the float values are reconstructed as
    /// - FloatY = (2^ExponentY) * IntY / (2^32)
    /// - FloatY = (2^ExponentY) * IntY / (2^16)
    /// Depending on the whether the data is 16 or 32 bit according to the flag parameters.
    pub(crate) exponent_y: i8,
    /// The integer index number of the trace subfile, where 0 refers to the first
    index_number: u16,
    /// The z-axis coordinate for this trace
    z: f32,
    /// The z-axis coordinate for the next trace
    next_z: f32,
    /// The floating peak pick noise value, if the high byte is nonzero
    noise: f32,
    /// The integer number of subfile points for TXYXYS types
    number_points: u32,
    /// The integer number of co-added scans
    scan: u32,
    /// The value of the floating w-axis (if fwplanes is non-zero)
    w_level: f32,
    /// A reserved region which must be set to zero
    ///
    /// This is only stored here so we can implement [`TryFromBytes`]
    reserved: [u8; 4],
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
        let start = self.0.byte;

        // let parameters = SubFlagParameters(self.read_byte()?);
        let parameters = SubFlagParameters(self.read_byte()?);
        let exponent_y = self.read_i8()?;
        let index_number = self.read_u16()?;
        let z = self.read_f32()?;
        let next_z = self.read_f32()?;
        let noise = self.read_f32()?;
        let number_points = self.read_u32()?;
        let scan = self.read_u32()?;
        let w_level = self.read_f32()?;

        for _ in 0..4 {
            let each = self.read_byte()?;
            if each != 0 {
                return Err(SubHeaderParseError::ReservedFieldsNotZero);
            }
        }

        assert_eq!(self.0.byte - start, 32);

        Ok(Subheader {
            parameters,
            exponent_y,
            index_number,
            z,
            next_z,
            noise,
            number_points,
            scan,
            w_level,
            reserved: [0; 4],
        })
    }
}

#[cfg(test)]
mod test {
    use crate::parse::SPCFile;

    use super::SubHeaderParser;

    #[test]
    fn water_refractive_index_subheader_parses_correctly() {
        let data = include_bytes!("../../test_data/subheader/WTERN95SUBHEADER.SPC");
        let mut parser = SPCFile::new(data);

        let mut subheader_parser = SubHeaderParser(&mut parser);
        let result = subheader_parser.parse();

        assert!(result.is_ok());
        let parsed = result.unwrap();

        dbg!(&parsed);
    }

    #[test]
    fn water_absorption_coefficient_subheader() {
        let data = include_bytes!("../../test_data/subheader/WTERK95SUBHEADER.SPC");
        let mut parser = SPCFile::new(data);

        let mut subheader_parser = SubHeaderParser(&mut parser);
        let result = subheader_parser.parse();

        assert!(result.is_ok());
        let parsed = result.unwrap();

        dbg!(&parsed);
    }
}
