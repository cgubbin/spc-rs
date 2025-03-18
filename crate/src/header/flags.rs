/// The first byte of the SPC file contains flags, describing the data to come

/// Flag parameters for an SPC file
///
/// The 8 bits of the flag parameters correspond to the following, ordered from smallest to
/// largest:
/// - TSPREC: Y data blocks are 16-bit integers
/// - TCGRAM: Enables fexper in older software (unused)
/// - TMULTI: Dataformat is multifile
/// - TRANDM: If TMULTI and TRANDM then Z values in SUBHDR structures are randomly ordered (unused)
/// - TORDRD: If TMULTI and TORDRD then Z values are in ascending or descending order, but are not
///     evenly spaced. Z-values are read from individual SUBHDR structures
/// - TALABS: Axis label text is stored in fcatxt, separated by null values. Ignores fxtype. fytype
///     and fztype corresponding to non-null text in fcatxt
/// - TXYXYS: Each subfile has a unique x-array. This can only be used if TXVALS is also used.
/// - TXVALS: X-data is not evenly spaced, an x-value array preceeds the y-data blocks
#[derive(Copy, Clone, Debug)]
pub(crate) struct FlagParameters(pub(super) u8);

/**
 * The new file format records as:
 * - Y. X is implicit (calc from XStart, XEnd, Y.length)
 * - XY. Single Y uneven, explicit X.
 * - YY. Multiple spectra (Ys), implicit, unique, even X.
 * - XYY. Multiple Ys, one unique, uneven X.
 * - XYYX. Multiple Ys, Multiple Xs (even or not, I think, but will be explicit).
 * The old file format records only: Y or YY.
 *
 */
pub(crate) enum DataShape {
    Y,
    XY,
    YY,
    XYY,
    XYXY,
}

impl FlagParameters {
    pub(crate) fn y_precision_is_16_bit(&self) -> bool {
        (self.0 & 1) == 1
    }

    fn use_fexper_extension(&self) -> bool {
        (self.0 >> 1 & 1) == 1
    }

    fn multifile(&self) -> bool {
        (self.0 >> 2 & 1) == 1
    }

    fn z_values_are_random(&self) -> bool {
        (self.0 >> 3 & 1) == 1
    }

    fn z_values_are_uneven(&self) -> bool {
        (self.0 >> 4 & 1) == 1
    }

    fn custom_axis_labels(&self) -> bool {
        (self.0 >> 5 & 1) == 1
    }

    fn xyxy(&self) -> bool {
        (self.0 >> 6 & 1) == 1
    }

    fn xy(&self) -> bool {
        (self.0 >> 7 & 1) == 1
    }

    pub(crate) fn data_shape(&self) -> DataShape {
        // Single file data
        if !self.multifile() {
            // Data is Y or XY
            if !self.xy() {
                return DataShape::Y;
            } else {
                if self.xyxy() {
                    panic!("exception in datashape creation")
                } else {
                    return DataShape::XY;
                }
            }
        }

        // multifile data
        if !self.xy() {
            // Even X with equidistant Y
            DataShape::YY
        } else {
            // Uneven x
            if !self.xy() {
                DataShape::XYY
            } else {
                DataShape::XYXY
            }
        }
    }
}

impl ::std::fmt::Display for FlagParameters {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        writeln!(
            f,
            "y precision: {} bit",
            if self.y_precision_is_16_bit() { 16 } else { 32 }
        )?;
        writeln!(f, "fexper: {}", self.use_fexper_extension())?;
        writeln!(f, "multifile data: {}", self.multifile())?;
        writeln!(f, "random z: {}", self.z_values_are_random())?;
        writeln!(f, "uneven z: {}", self.z_values_are_uneven())?;
        writeln!(f, "custom axis labels: {}", self.custom_axis_labels())?;
        writeln!(f, "xyxy: {}", self.xyxy())?;
        writeln!(f, "xy: {}", self.xy())?;
        Ok(())
    }
}
