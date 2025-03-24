use zerocopy::{
    byteorder::{F32, U16, U32},
    ByteOrder, Immutable, KnownLayout, TryFromBytes,
};

#[derive(Debug, thiserror::Error, miette::Diagnostic)]
pub(crate) enum SubheaderValidationError {
    #[error("The reserved fields were not set to zero")]
    ReservedFieldsNotZero,
    #[error("The subheader flags should only have bits 0, 3, and 7 set but found: {0}")]
    SubheaderFlags(u8),
}

#[repr(C)]
#[derive(Clone, Debug, KnownLayout, Immutable, TryFromBytes)]
struct GuardedSubheader<E: ByteOrder>(Subheader<E>);

impl<E: ByteOrder> GuardedSubheader<E> {
    pub(crate) fn try_into_inner(&self) -> Result<&Subheader<E>, SubheaderValidationError> {
        if self.0.reserved != [0; 4] {
            return Err(SubheaderValidationError::ReservedFieldsNotZero);
        }
        if self.0.parameters.0 & 0b1000_1001 != self.0.parameters.0 {
            return Err(SubheaderValidationError::SubheaderFlags(
                self.0.parameters.0,
            ));
        }
        Ok(&self.0)
    }
}

#[repr(C)]
#[derive(Clone, Debug, KnownLayout, Immutable, TryFromBytes)]
struct SubFlagParameters(u8);

#[repr(C)]
#[derive(Clone, Debug, KnownLayout, Immutable, TryFromBytes)]
pub(crate) struct Subheader<E: ByteOrder> {
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

#[cfg(test)]
mod test {
    use zerocopy::{LittleEndian, TryFromBytes};

    use crate::header::zerocopy_subheader::GuardedSubheader;

    #[test]
    fn water_refractive_index_subheader_parses_correctly() {
        let data = include_bytes!("../../test_data/subheader/WTERN95SUBHEADER.SPC");
        let result = GuardedSubheader::try_ref_from_bytes(data);

        assert!(result.is_ok());
        let parsed: &GuardedSubheader<LittleEndian> = result.unwrap();

        let result = parsed.try_into_inner();
        assert!(result.is_ok());
    }
    #[test]
    fn water_absorption_coefficient_subheader() {
        let data = include_bytes!("../../test_data/subheader/WTERK95SUBHEADER.SPC");

        let result = GuardedSubheader::try_ref_from_bytes(data);

        assert!(result.is_ok());
        let parsed: &GuardedSubheader<LittleEndian> = result.unwrap();

        let result = parsed.try_into_inner();
        assert!(result.is_ok());
    }
}
