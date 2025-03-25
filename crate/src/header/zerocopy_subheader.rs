use zerocopy::{
    byteorder::{F32, U16, U32},
    ByteOrder, Immutable, KnownLayout, TryFromBytes,
};

use crate::parse::{Parse, TryParse};

#[derive(Clone, Debug, thiserror::Error, miette::Diagnostic)]
pub(crate) enum SubheaderParseError {
    #[error("The reserved fields were not set to zero")]
    ReservedFieldsNotZero,
    #[error("The subheader flags should only have bits 0, 3, and 7 set but found: {0}")]
    SubheaderFlags(u8),
}

#[repr(C)]
#[derive(Clone, Debug, KnownLayout, Immutable, TryFromBytes)]
struct GuardedLexedSubheader<E: ByteOrder>(LexedSubheader<E>);

impl<E: ByteOrder> GuardedLexedSubheader<E> {
    pub(crate) fn try_into_inner(&self) -> Result<&LexedSubheader<E>, SubheaderParseError> {
        if self.0.reserved != [0; 4] {
            return Err(SubheaderParseError::ReservedFieldsNotZero);
        }
        if self.0.parameters.0 & 0b1000_1001 != self.0.parameters.0 {
            return Err(SubheaderParseError::SubheaderFlags(self.0.parameters.0));
        }
        Ok(&self.0)
    }
}

#[repr(C)]
#[derive(Clone, Copy, Debug, KnownLayout, Immutable, TryFromBytes)]
struct SubFlagParameters(u8);

#[repr(C)]
#[derive(Clone, Debug, KnownLayout, Immutable, TryFromBytes)]
pub(crate) struct LexedSubheader<E: ByteOrder> {
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
    index_number: U16<E>,
    /// The z-axis coordinate for this trace
    z: F32<E>,
    /// The z-axis coordinate for the next trace
    next_z: F32<E>,
    /// The floating peak pick noise value, if the high byte is nonzero
    noise: F32<E>,
    /// The integer number of subfile points for TXYXYS types
    pub(crate) number_points: U32<E>,
    /// The integer number of co-added scans
    scan: U32<E>,
    /// The value of the floating w-axis (if fwplanes is non-zero)
    w_level: F32<E>,
    /// A reserved region which must be set to zero
    ///
    /// This is only stored here so we can implement [`TryFromBytes`]
    reserved: [u8; 4],
}

#[derive(Clone, Debug)]
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
    pub(crate) number_points: u32,
    /// The integer number of co-added scans
    scan: u32,
    /// The value of the floating w-axis (if fwplanes is non-zero)
    w_level: f32,
    /// A reserved region which must be set to zero
    ///
    /// This is only stored here so we can implement [`TryFromBytes`]
    reserved: [u8; 4],
}

impl<E: ByteOrder> TryParse for LexedSubheader<E> {
    type Parsed = Subheader;
    type Error = SubheaderParseError;

    fn try_parse(&self) -> Result<Self::Parsed, Self::Error> {
        if self.reserved.iter().any(|val| *val != 0) {
            return Err(SubheaderParseError::ReservedFieldsNotZero);
        }
        if (self.parameters.0 & 0b1000_1001) != self.parameters.0 {
            return Err(SubheaderParseError::SubheaderFlags(self.parameters.0));
        }
        Ok(Subheader {
            parameters: self.parameters,
            exponent_y: self.exponent_y,
            index_number: self.index_number.get(),
            z: self.z.get(),
            next_z: self.next_z.get(),
            noise: self.noise.get(),
            number_points: self.number_points.get(),
            scan: self.scan.get(),
            w_level: self.w_level.get(),
            reserved: self.reserved,
        })
    }
}

impl<E: ByteOrder> LexedSubheader<E> {
    pub(crate) fn number_of_points(&self) -> usize {
        let number_points: u32 = self.number_points.into();
        number_points as usize
    }

    pub(crate) fn float_data_expected(&self) -> bool {
        self.exponent_y == -128
    }
}

#[cfg(test)]
mod test {
    use zerocopy::{LittleEndian, TryFromBytes};

    use crate::header::zerocopy_subheader::GuardedLexedSubheader;

    #[test]
    fn water_refractive_index_subheader_parses_correctly() {
        let data = include_bytes!("../../test_data/subheader/WTERN95SUBHEADER.SPC");
        let result = GuardedLexedSubheader::try_ref_from_bytes(data);

        assert!(result.is_ok());
        let parsed: &GuardedLexedSubheader<LittleEndian> = result.unwrap();

        let result = parsed.try_into_inner();
        assert!(result.is_ok());
    }
    #[test]
    fn water_absorption_coefficient_subheader() {
        let data = include_bytes!("../../test_data/subheader/WTERK95SUBHEADER.SPC");

        let result = GuardedLexedSubheader::try_ref_from_bytes(data);

        assert!(result.is_ok());
        let parsed: &GuardedLexedSubheader<LittleEndian> = result.unwrap();

        let result = parsed.try_into_inner();
        assert!(result.is_ok());
    }
}
