mod datetime;
mod flags;
mod subheader;

use datetime::{Month, MonthParseError};
pub(crate) use flags::DataShape;
use miette::Diagnostic;
pub(crate) use subheader::{SubHeaderParseError, SubHeaderParser, Subheader};

use crate::{parse::Endian, xzwType, yType, InstrumentTechnique, SPCFile};
use flags::FlagParameters;

use chrono::{DateTime, LocalResult, TimeZone, Utc};

#[derive(thiserror::Error, Debug, Diagnostic)]
pub(crate) enum HeaderParseError {
    #[error(
        "Ambiguous datetime data:\n
                year = {year},\n
                month = {month},\n
                date = {date},\n
                hours = {hours},\n
                minutes = {minutes},"
    )]
    Datetime {
        year: u16,
        month: u8,
        date: u8,
        hours: u8,
        minutes: u8,
    },
    #[error("Invalid value in spare field")]
    SpareNonZero,
    #[error("Invalid value in reserved field")]
    ReservedNonZero,
    #[error("File ended before fixed length header could be parsed")]
    PrematureTermination,
    #[error("Error in Month Value: {0}")]
    MonthParse(#[from] MonthParseError),
}

/// The header of an SPC file has two formats, depending on the version of software which created
/// the file.
#[derive(Clone, Debug)]
pub(crate) enum Header {
    // Headers created by SPC software pre-1996 with file version 0x4b
    Old(OldFormatHeader),
    // Headers created by SPC software post with file versions 0x4c of 0x4d
    New(NewFormatHeader),
}

/// In the old SPC format, the header is 224 bytes long. The subsequent single sub-header is
/// 32 bytes in length, so the total length is 256 bytes. This is common between new and old style
///    and is parsed separately.
///
/// Data in in aold-format header is organised as follows
/// - Byte: File type flag parameters
/// - Byte: SPC file version. Always equal to 0x4b. This is checked but not stored.
/// - Short: Exponent for y-values
/// - Float: Floating point number of data points
/// - Float: First x-coordinate in single precision
/// - Float: Last x-coordinate in single precision
/// - Byte: x-unit type code
/// - Byte: y-unit type code
/// - Word: Year collected. 0 refers to no date or time data being available.
///     The most significant four bits are the z-type
/// - Byte: Month collected (1=Jan)
/// - Byte: Day of month collected (1=1st)
/// - Byte: Hour of day collected (13=1pm)
/// - Byte: Minute of hour collected
/// - Char[8]: Resolution description text
/// - Word: Peak point number for interferograms
/// - Word: Number of scans
/// - Float[7]: Spare
/// - Char[130]: Memo
/// - Char[30]: XYZ custom axis strings
///
/// Note that the spare field is checked to be empty during parsing, but not stored in the
/// intermediate representation.
///
/// Contrasing the [`NewFormatHeader`], in this format the number of points is stored as a float,
/// rather than a double and the x-limits are only stored in single precision.
#[derive(Clone, Debug)]
pub(crate) struct OldFormatHeader {
    /// The [`FlagParameters`] for the .SPC
    pub(super) flags: FlagParameters,
    /// The exponent for the Y values.
    ///
    /// If the exponent is equal to 80h, then the values are to be interpreted directly as floating
    /// point data. If not the float values are reconstructed as
    /// - FloatY = (2^ExponentY) * IntY / (2^32)
    /// - FloatY = (2^ExponentY) * IntY / (2^16)
    /// Depending on the whether the data is 16 or 32 bit according to the [`FlagParameters`]
    pub(super) exponent_y: i16,
    /// The number of data points in the file if it is not XYXY format
    ///
    /// If x-data is not explicitly provided then it is assumed to be evenly spaced, and is
    /// constructed as a linear space from the starting_x to the ending_x containing number points.
    pub(super) number_points: f32,
    /// The first x-coordinate in the dataset
    ///
    /// If x-data is not explicitly provided then it is assumed to be evenly spaced, and is
    /// constructed as a linear space from the starting_x to the ending_x containing number points.
    pub(super) starting_x: f32,
    /// The last x-coordinate in the dataset
    ///
    /// If x-data is not explicitly provided then it is assumed to be evenly spaced, and is
    /// constructed as a linear space from the starting_x to the ending_x containing number points.
    pub(super) ending_x: f32,
    pub(super) x_unit_type: xzwType,
    pub(super) y_unit_type: yType,
    pub(super) z_unit_type: xzwType,
    pub(super) datetime: Option<DateTime<Utc>>,
    pub(super) resolution_description: String,
    pub(super) peak_point_number: u16,
    pub(super) scans: u16,
    pub(super) memo: String,
    pub(super) xyz_labels: String,
}

/// A New format header is always 512 bytes long.
///
/// The data contained in the Header is organised as follows:
/// - Byte: File type flag parameters
/// - Byte: SPC file version
/// - Byte: Experiment type code
/// - Char: Exponent for Y values
/// - Long: Number of data points contained in the file if format is not XYXY
/// - Double: First x-coordinate
/// - Double: Last x-coordinate
/// - Long: Number of subfiles contained in the file
/// - Byte: x-unit type code
/// - Byte: y-unit type code
/// - Byte: z-unit type code
/// - Byte: Posting disposition
/// - Long: Compressed date format.
///     6 bits = minutes
///     5 bits = hour
///     5 bits = day
///     4 bits = month
///     12 bits = year
/// - Char[9]: Resolution description text
/// - Char[9]: source_instrument_description text
/// - Word: Peak point number for interferograms
/// - Float[8]: Spare
/// - Char[130]: Memo
/// - Char[30]: XYZ custom axis strings
/// - Long: The offset to the log block location in bytes
/// - Long: File modification flags
/// - Byte : Processing code
/// - Byte : Calibration level + 1
/// - Word : Sub-method sample injection number
/// - Float: Floating Data Concentration Factor
/// - Char[48]: Method file name
/// - Float: Z subfile increment for even Z multifiles
/// - Long: Number of W planes
/// - Float: W plane increment
/// - Byte: W axis units
/// - Char[187]: Reserved
#[derive(Clone, Debug)]
pub(crate) struct NewFormatHeader {
    /// File version for a New Format SPC File.
    ///
    /// This must either be 0x4b or 0x4c. The difference refers to the ordering of data in the
    /// binary file:
    /// - 0x4b Refers to LSB (Least Significant Bit) ordering. Or Little Endian.
    /// - 0x4c Refers to MSB (Most Significant Bit) ordering. Or Big Endian.
    pub(super) file_version: u8,
    /// Flag parameters are packend into a single byte
    pub(super) flags: FlagParameters,
    pub(super) instrument_technique: InstrumentTechnique,
    /// The exponent for the Y values.
    ///
    /// If the exponent is equal to 80h, then the values are to be interpreted directly as floating
    /// point data. If not the float values are reconstructed as
    /// - FloatY = (2^ExponentY) * IntY / (2^32)
    /// - FloatY = (2^ExponentY) * IntY / (2^16)
    /// Depending on the whether the data is 16 or 32 bit according to the flag parameters.
    pub(super) exponent_y: i8,
    /// If the file is not in XYXY format then this refers to the number of points contained in the
    /// file
    pub(super) number_points: u32,
    pub(super) starting_x: f64,
    pub(super) ending_x: f64,
    pub(super) spectra: u32,
    pub(super) x_unit_type: xzwType,
    pub(super) y_unit_type: yType,
    pub(super) z_unit_type: xzwType,
    pub(super) posting_disposition: u8,
    pub(super) datetime: DateTime<Utc>,
    pub(super) resolution_description: String,
    pub(super) source_instrument_description: String,
    pub(super) peak_point_number: u16,
    pub(super) memo: String,
    pub(super) xyz_labels: String,
    pub(super) log_offset: u32,
    pub(super) modified_flag: u32,
    pub(super) processing_code: u8,
    pub(super) calibration_level: u8,
    pub(super) sub_method_sample_injection_number: u16,
    pub(super) concentration_factor: f32,
    pub(super) method_file: String,
    pub(super) z_sub_increment: f32,
    pub(super) w_planes: u32,
    pub(super) w_plane_increment: f32,
    pub(super) w_axis_units: xzwType,
}

pub(crate) struct HeaderParser<'a, 'de> {
    spc: &'a mut SPCFile<'de>,
    flags: FlagParameters,
    version: u8,
}

impl<'a, 'de> HeaderParser<'a, 'de> {
    pub(crate) fn new(spc: &'a mut SPCFile<'de>) -> Result<Self, HeaderParseError> {
        let flags = FlagParameters(
            spc.read_byte()
                .ok_or(HeaderParseError::PrematureTermination)?,
        );
        let version = spc
            .read_byte()
            .ok_or(HeaderParseError::PrematureTermination)?;

        if !matches!(version, 0x4b..=0x4d) {
            panic!("Unsupported SPC version: {}", version);
        }

        Ok(Self {
            spc,
            flags,
            version,
        })
    }

    pub(crate) fn parse(&mut self) -> Result<Header, HeaderParseError> {
        match self.version {
            0x4d => {
                let header = self.parse_old_format()?;

                Ok(Header::Old(header))
            }
            0x4b => {
                self.spc.set_endian(Endian::Little);
                let header = self.parse_new_format()?;
                Ok(Header::New(header))
            }
            0x4c => {
                self.spc.set_endian(Endian::Big);
                let header = self.parse_new_format()?;
                Ok(Header::New(header))
            }
            _ => unreachable!("can only create a header constructor with a valid file version"),
        }
    }

    fn read_byte(&mut self) -> Result<u8, HeaderParseError> {
        self.spc
            .read_byte()
            .ok_or(HeaderParseError::PrematureTermination)
    }

    fn read_i8(&mut self) -> Result<i8, HeaderParseError> {
        self.spc
            .read_i8()
            .ok_or(HeaderParseError::PrematureTermination)
    }

    fn read_i16(&mut self) -> Result<i16, HeaderParseError> {
        self.spc
            .read_i16()
            .ok_or(HeaderParseError::PrematureTermination)
    }

    fn read_u16(&mut self) -> Result<u16, HeaderParseError> {
        self.spc
            .read_u16()
            .ok_or(HeaderParseError::PrematureTermination)
    }

    fn read_u32(&mut self) -> Result<u32, HeaderParseError> {
        self.spc
            .read_u32()
            .ok_or(HeaderParseError::PrematureTermination)
    }

    fn read_f32(&mut self) -> Result<f32, HeaderParseError> {
        self.spc
            .read_f32()
            .ok_or(HeaderParseError::PrematureTermination)
    }

    fn read_f64(&mut self) -> Result<f64, HeaderParseError> {
        self.spc
            .read_f64()
            .ok_or(HeaderParseError::PrematureTermination)
    }

    fn read_unescaped_utf8(&mut self, len: usize) -> Result<&'de str, HeaderParseError> {
        self.spc
            .read_unescaped_utf8(len)
            .ok_or(HeaderParseError::PrematureTermination)
    }

    fn parse_new_format(&mut self) -> Result<NewFormatHeader, HeaderParseError> {
        let instrument_technique = InstrumentTechnique::new(self.read_byte()?).unwrap();
        let exponent_y = self.read_i8()?;
        let number_points = self.read_u32()?;
        let starting_x = self.read_f64()?;
        let ending_x = self.read_f64()?;
        let spectra = self.read_u32()?;
        let x_unit_type = xzwType::new(self.read_byte()?).unwrap();
        let y_unit_type = yType::new(self.read_byte()?).unwrap();
        let z_unit_type = xzwType::new(self.read_byte()?).unwrap();

        let posting_disposition = self.read_byte()?;

        let datetime = self.parse_new_format_datetime()?;

        let resolution_description = self.read_unescaped_utf8(9)?.trim().to_owned();
        let source_instrument_description = self.read_unescaped_utf8(9)?.trim().to_owned();

        let peak_point_number = self.read_u16()?;

        // Read the spare values, and check that they are null
        for _ in 0..8 {
            let value = self
                .spc
                .read_f32()
                .ok_or(HeaderParseError::PrematureTermination)?;
            if value != 0.0 {
                return Err(HeaderParseError::SpareNonZero);
            }
        }

        let memo = self.read_unescaped_utf8(130)?.trim().to_owned();
        let xyz_labels = self.read_unescaped_utf8(30)?.trim().to_owned();

        let log_offset = self.read_u32()?;
        let modified_flag = self.read_u32()?;
        let processing_code = self.read_byte()?;
        let calibration_level = self.read_byte()?;
        let sub_method_sample_injection_number = self.read_u16()?;

        let concentration_factor = self.read_f32()?;
        let method_file = self.read_unescaped_utf8(48)?.trim().to_owned();

        let z_sub_increment = self.read_f32()?;
        let w_planes = self.read_u32()?;
        let w_plane_increment = self.read_f32()?;
        let w_axis_units = xzwType::new(self.read_byte()?).unwrap();

        for _ in 0..187 {
            if self.read_byte()? != 0 {
                return Err(HeaderParseError::ReservedNonZero);
            }
        }

        assert_eq!(self.spc.byte, 512);

        Ok(NewFormatHeader {
            file_version: self.version,
            flags: self.flags,
            instrument_technique,
            exponent_y,
            number_points,
            starting_x,
            ending_x,
            spectra,
            x_unit_type,
            y_unit_type,
            z_unit_type,
            posting_disposition,
            datetime,
            resolution_description,
            source_instrument_description,
            peak_point_number,
            memo,
            xyz_labels,
            log_offset,
            modified_flag,
            processing_code,
            calibration_level,
            sub_method_sample_injection_number,
            concentration_factor,
            method_file,
            z_sub_increment,
            w_planes,
            w_plane_increment,
            w_axis_units,
        })
    }

    fn parse_old_format(&mut self) -> Result<OldFormatHeader, HeaderParseError> {
        let exponent_y = self.read_i16()?;
        let number_points = self.read_f32()?;
        let starting_x = self.read_f32()?;
        let ending_x = self.read_f32()?;

        let x_unit_type = xzwType::new(self.read_byte()?).unwrap();
        let y_unit_type = yType::new(self.read_byte()?).unwrap();

        // The z-data type is stored in the most significant 4 bits of the year
        let z_type_year = self.read_u16()?;
        let z_unit_type = xzwType::new((z_type_year >> 12) as u8).unwrap();

        let year = z_type_year & 0x0fff;

        // The datetime is only available if the year is non-zero
        let datetime = if year == 0 {
            let _ = self.read_f32()?; // If there is no data, then we still need to read the bytes
                                      // to advance
            None
        } else {
            Some(self.parse_old_format_datetime(year)?)
        };

        let resolution_description = self.read_unescaped_utf8(8)?.trim().to_owned();
        let peak_point_number = self.read_u16()?;
        let scans = self.read_u16()?;

        for _ in 0..7 {
            let each = self.read_f32()?;
            if each != 0.0 {
                panic!("Spare value is not 0.0");
            }
        }

        let memo = self.read_unescaped_utf8(130)?.trim().to_owned();
        let xyz_labels = self.read_unescaped_utf8(30)?.trim().to_owned();

        assert_eq!(self.spc.byte, 224);

        Ok(OldFormatHeader {
            flags: self.flags,
            exponent_y,
            number_points,
            starting_x,
            ending_x,
            x_unit_type,
            y_unit_type,
            z_unit_type,
            datetime,
            resolution_description,
            peak_point_number,
            scans,
            memo,
            xyz_labels,
        })
    }

    // Old format datetimes are stored in 4 consecutive bits following the year
    fn parse_old_format_datetime(&mut self, year: u16) -> Result<DateTime<Utc>, HeaderParseError> {
        let month = Month::try_from(self.read_byte()?)?;
        let date = self.read_byte()?;
        let hours = self.read_byte()?;
        let minutes = self.read_byte()?;

        match Utc.with_ymd_and_hms(
            year as i32,
            month as u32,
            date as u32,
            hours as u32,
            minutes as u32,
            0,
        ) {
            LocalResult::Single(datetime) => Ok(datetime),
            LocalResult::None => Err(HeaderParseError::Datetime {
                year,
                month: month as u8,
                date,
                hours,
                minutes,
            }),
            LocalResult::Ambiguous(a, b) => {
                dbg!(a, b);

                Err(HeaderParseError::Datetime {
                    year,
                    month: month as u8,
                    date,
                    hours,
                    minutes,
                })
            }
        }
    }

    fn parse_new_format_datetime(&mut self) -> Result<DateTime<Utc>, HeaderParseError> {
        let data = self.read_u32()?;

        // The least significant six bits are the minutes
        let minutes = (data & 0b111111) as u8;
        // The next five bits are the hour
        let hours = ((data >> 6) & 0b11111) as u8;
        // The next five bits are the day
        let date = ((data >> 11) & 0b11111) as u8;
        // The next four bits are the month
        let month = ((data >> 16) & 0b1111) as u8;
        // And the rest is the year
        let year = (data >> 20) as u16;
        //
        println!(
            "Year: {}, Month: {}, Date: {}, Hours: {}, Minutes: {}",
            year, month, date, hours, minutes
        );

        match Utc.with_ymd_and_hms(
            year as i32,
            month as u32,
            date as u32,
            hours as u32,
            minutes as u32,
            0,
        ) {
            LocalResult::Single(datetime) => Ok(datetime),
            LocalResult::None => Err(HeaderParseError::Datetime {
                year,
                month,
                date,
                hours,
                minutes,
            }),
            LocalResult::Ambiguous(_, _) => Err(HeaderParseError::Datetime {
                year,
                month,
                date,
                hours,
                minutes,
            }),
        }
    }
}

#[cfg(test)]
mod test {
    use std::io::{BufRead, BufReader};

    use crate::parse::SPCFile;

    use super::HeaderParser;

    use chrono::{Datelike, Timelike};
    use regex::Regex;

    #[test]
    fn m_xyxy_header_parses_correctly() {
        let data = include_bytes!("../../test_data/header/m_xyxy.spc");
        let mut parser = SPCFile::new(data);

        let mut header_parser = HeaderParser::new(&mut parser).unwrap();
        let result = header_parser.parse();

        assert!(result.is_ok());
        let parsed = result.unwrap();

        let header = match parsed {
            super::Header::New(header) => header,
            _ => panic!("Expected an new format header"),
        };

        assert!(header.flags.multifile());
        assert_eq!(header.x_unit_type, crate::xzwType::Mass);
        assert_eq!(header.z_unit_type, crate::xzwType::Minutes);

        let memo_regex = Regex::new(r"^Multiple .*X & Y arrays").unwrap();
        assert!(memo_regex.is_match(&header.memo));

        let datetime = header.datetime;
        assert_eq!(datetime.minute(), 47);
        assert_eq!(datetime.hour(), 8);
        assert_eq!(datetime.day(), 9);
        assert_eq!(datetime.month(), 1);
        assert_eq!(datetime.year(), 1986);
        //
    }

    #[test]
    fn raman_header_parses_correctly() {
        let data = include_bytes!("../../test_data/header/raman.spc");
        let mut parser = SPCFile::new(data);

        let mut header_parser = HeaderParser::new(&mut parser).unwrap();
        let result = header_parser.parse();

        assert!(result.is_ok());
        let parsed = result.unwrap();

        let header = match parsed {
            super::Header::New(header) => header,
            _ => panic!("Expected an new format header"),
        };

        assert_eq!(header.x_unit_type, crate::xzwType::RamanShift);

        let datetime = header.datetime;
        assert_eq!(datetime.minute(), 45);
        assert_eq!(datetime.hour(), 16);
        assert_eq!(datetime.day(), 26);
        assert_eq!(datetime.month(), 8);
        assert_eq!(datetime.year(), 1994);
        //
    }

    #[test]
    fn m_ordz_header_parses_correctly() {
        let data = include_bytes!("../../test_data/header/m_ordz.spc");
        let mut parser = SPCFile::new(data);

        let mut header_parser = HeaderParser::new(&mut parser).unwrap();
        let result = header_parser.parse();

        assert!(result.is_ok());
        let parsed = result.unwrap();

        let header = match parsed {
            super::Header::Old(header) => header,
            _ => panic!("Expected an new format header"),
        };

        let memo_regex = Regex::new(r"^Multiple .*ordered Z spacing").unwrap();
        assert!(memo_regex.is_match(&header.memo));
    }

    #[test]
    fn water_refractive_index_header_parses_correctly() {
        let data = include_bytes!("../../test_data/header/WTERN95HEADER.SPC");
        let mut parser = SPCFile::new(data);

        let mut header_parser = HeaderParser::new(&mut parser).unwrap();
        let result = header_parser.parse();

        assert!(result.is_ok());
        let parsed = result.unwrap();

        let header = match parsed {
            super::Header::Old(header) => header,
            _ => panic!("Expected an old format header"),
        };

        assert_eq!(header.x_unit_type, crate::xzwType::Wavenumber);
        assert_eq!(
            header.memo,
            "n spectrum of liquid H2O at 25 C; Appl. Spec. 50, 1047 (1996)"
        );

        let datetime = header.datetime.unwrap();
        assert_eq!(datetime.year(), 1995);
    }

    #[test]
    fn water_absorption_coefficient_header_parses_correctly() {
        let data = include_bytes!("../../test_data/header/WTERK95HEADER.SPC");
        let mut parser = SPCFile::new(data);

        let mut header_parser = HeaderParser::new(&mut parser).unwrap();
        let result = header_parser.parse();

        assert!(result.is_ok());
        let parsed = result.unwrap();

        let header = match parsed {
            super::Header::Old(header) => header,
            _ => panic!("Expected an old format header"),
        };

        assert_eq!(header.x_unit_type, crate::xzwType::Wavenumber);
        assert_eq!(
            header.memo,
            "k spectrum of liquid H2O at 25 C;  Appl. Spec. 50, 1047 (1996)"
        );

        let datetime = header.datetime.unwrap();
        assert_eq!(datetime.year(), 1995);
    }

    #[test]
    fn water_real_dielectric_constant_header_parses_correctly() {
        let data = include_bytes!("../../test_data/header/WTERDC95HEADER.SPC");
        let mut parser = SPCFile::new(data);

        let mut header_parser = HeaderParser::new(&mut parser).unwrap();
        let result = header_parser.parse();

        assert!(result.is_ok());
        let parsed = result.unwrap();

        let header = match parsed {
            super::Header::Old(header) => header,
            _ => panic!("Expected an old format header"),
        };

        assert_eq!(header.x_unit_type, crate::xzwType::Wavenumber);
        assert_eq!(
            header.memo,
            "Real dielectric constant of H2O at 25 C; Appl. Spec. 50, 1047 (1996)"
        );

        let datetime = header.datetime.unwrap();
        assert_eq!(datetime.year(), 1995);
    }

    #[test]
    fn water_dielectric_loss_header_parses_correctly() {
        let data = include_bytes!("../../test_data/header/WTERDL95HEADER.SPC");
        let mut parser = SPCFile::new(data);

        let mut header_parser = HeaderParser::new(&mut parser).unwrap();
        let result = header_parser.parse();

        assert!(result.is_ok());
        let parsed = result.unwrap();

        let header = match parsed {
            super::Header::Old(header) => header,
            _ => panic!("Expected an old format header"),
        };

        assert_eq!(header.x_unit_type, crate::xzwType::Wavenumber);
        assert_eq!(
            header.memo,
            "dielectric loss spectrum of H2O at 25 C;  Appl. Spec. 50, 1047 (1996)"
        );

        let datetime = header.datetime.unwrap();
        assert_eq!(datetime.year(), 1995);
    }

    #[test]
    fn water_molar_absorptivity_header_parses_correctly() {
        let data = include_bytes!("../../test_data/header/WTEREM95HEADER.SPC");
        let mut parser = SPCFile::new(data);

        let mut header_parser = HeaderParser::new(&mut parser).unwrap();
        let result = header_parser.parse();

        assert!(result.is_ok());

        let parsed = result.unwrap();

        let header = match parsed {
            super::Header::Old(header) => header,
            _ => panic!("Expected an old format header"),
        };

        assert_eq!(header.x_unit_type, crate::xzwType::Wavenumber);
        assert_eq!(
            header.memo,
            "molar absorptivity spectrum of H2O at 25 C in L/(mole-cm)"
        );

        let datetime = header.datetime.unwrap();
        assert_eq!(datetime.year(), 1995);
    }

    #[test]
    fn c6d6_dielectric_constant_header_parses_correctly() {
        let data = include_bytes!("../../test_data/header/C6D6DC98.SPC");
        let mut parser = SPCFile::new(data);

        let mut header_parser = HeaderParser::new(&mut parser).unwrap();
        let result = header_parser.parse();

        assert!(result.is_ok());

        let parsed = result.unwrap();

        let header = match parsed {
            super::Header::Old(header) => header,
            _ => panic!("Expected an old format header"),
        };

        assert_eq!(header.x_unit_type, crate::xzwType::Wavenumber);
        assert_eq!(header.memo, "Dielectric Constant of liquid C6D6 at 25 C");

        let datetime = header.datetime.unwrap();
        assert_eq!(datetime.year(), 1990);

        let txt_path = include_str!("../../test_data/txt/C6D6ASC/C6D6DC98.txt");
        let num_lines = BufReader::new(txt_path.as_bytes()).lines().count();
        assert_eq!(num_lines, header.number_points as usize);

        let mut reader = BufReader::new(txt_path.as_bytes()).lines();
        let first_x_in_txt: f32 = reader
            .next()
            .unwrap()
            .unwrap()
            .split_whitespace()
            .next()
            .unwrap()
            .parse()
            .unwrap();

        let last_x_in_txt: f32 = reader
            .last()
            .unwrap()
            .unwrap()
            .split_whitespace()
            .next()
            .unwrap()
            .parse()
            .unwrap();

        approx::assert_relative_eq!(first_x_in_txt, header.starting_x, epsilon = 1e-2);
        approx::assert_relative_eq!(last_x_in_txt, header.ending_x, epsilon = 1e-2);
    }

    #[test]
    fn c6d6_dielectric_loss_header_parses_correctly() {
        let data = include_bytes!("../../test_data/header/C6D6DL98.SPC");
        let mut parser = SPCFile::new(data);

        let mut header_parser = HeaderParser::new(&mut parser).unwrap();
        let result = header_parser.parse();

        assert!(result.is_ok());

        let parsed = result.unwrap();

        let header = match parsed {
            super::Header::Old(header) => header,
            _ => panic!("Expected an old format header"),
        };

        assert_eq!(header.x_unit_type, crate::xzwType::Wavenumber);
        assert_eq!(header.memo, "Dielectric Loss of liquid C6D6 at 25 C");

        let datetime = header.datetime.unwrap();
        assert_eq!(datetime.year(), 1990);

        let txt_path = include_str!("../../test_data/txt/C6D6ASC/C6D6DL98.txt");
        let num_lines = BufReader::new(txt_path.as_bytes()).lines().count();
        assert_eq!(num_lines, header.number_points as usize);

        let mut reader = BufReader::new(txt_path.as_bytes()).lines();
        let first_x_in_txt: f32 = reader
            .next()
            .unwrap()
            .unwrap()
            .split_whitespace()
            .next()
            .unwrap()
            .parse()
            .unwrap();

        let last_x_in_txt: f32 = reader
            .last()
            .unwrap()
            .unwrap()
            .split_whitespace()
            .next()
            .unwrap()
            .parse()
            .unwrap();

        approx::assert_relative_eq!(first_x_in_txt, header.starting_x, epsilon = 1e-2);
        approx::assert_relative_eq!(last_x_in_txt, header.ending_x, epsilon = 1e-2);
    }

    #[test]
    fn c6d6_molar_absorptivity_header_parses_correctly() {
        let data = include_bytes!("../../test_data/header/C6D6EM98.SPC");
        let mut parser = SPCFile::new(data);

        let mut header_parser = HeaderParser::new(&mut parser).unwrap();
        let result = header_parser.parse();

        assert!(result.is_ok());

        let parsed = result.unwrap();

        let header = match parsed {
            super::Header::Old(header) => header,
            _ => panic!("Expected an old format header"),
        };

        assert_eq!(header.x_unit_type, crate::xzwType::Wavenumber);
        assert_eq!(
            header.memo,
            "Molar Absorptivity of C6D6(l) at 25C; units L/(mole-cm)"
        );

        let datetime = header.datetime.unwrap();
        assert_eq!(datetime.year(), 1990);

        let txt_path = include_str!("../../test_data/txt/C6D6ASC/C6D6EM98.txt");
        let num_lines = BufReader::new(txt_path.as_bytes()).lines().count();
        assert_eq!(num_lines, header.number_points as usize);

        let mut reader = BufReader::new(txt_path.as_bytes()).lines();
        let first_x_in_txt: f32 = reader
            .next()
            .unwrap()
            .unwrap()
            .split_whitespace()
            .next()
            .unwrap()
            .parse()
            .unwrap();

        let last_x_in_txt: f32 = reader
            .last()
            .unwrap()
            .unwrap()
            .split_whitespace()
            .next()
            .unwrap()
            .parse()
            .unwrap();

        approx::assert_relative_eq!(first_x_in_txt, header.starting_x, epsilon = 1e-2);
        approx::assert_relative_eq!(last_x_in_txt, header.ending_x, epsilon = 1e-2);
    }

    #[test]
    fn c6d6_imaginary_molar_polarisability_header_parses_correctly() {
        let data = include_bytes!("../../test_data/header/C6D6IP98.SPC");
        let mut parser = SPCFile::new(data);

        let mut header_parser = HeaderParser::new(&mut parser).unwrap();
        let result = header_parser.parse();

        assert!(result.is_ok());

        let parsed = result.unwrap();

        let header = match parsed {
            super::Header::Old(header) => header,
            _ => panic!("Expected an old format header"),
        };

        assert_eq!(header.x_unit_type, crate::xzwType::Wavenumber);
        assert_eq!(
            header.memo,
            "imaginary molar polarizability of C6D6 at 25 C.Units:  cm^3/mole"
        );

        let datetime = header.datetime.unwrap();
        assert_eq!(datetime.year(), 1990);

        let txt_path = include_str!("../../test_data/txt/C6D6ASC/C6D6IP98.txt");
        let num_lines = BufReader::new(txt_path.as_bytes()).lines().count();
        assert_eq!(num_lines, header.number_points as usize);

        let mut reader = BufReader::new(txt_path.as_bytes()).lines();
        let first_x_in_txt: f32 = reader
            .next()
            .unwrap()
            .unwrap()
            .split_whitespace()
            .next()
            .unwrap()
            .parse()
            .unwrap();

        let last_x_in_txt: f32 = reader
            .last()
            .unwrap()
            .unwrap()
            .split_whitespace()
            .next()
            .unwrap()
            .parse()
            .unwrap();

        approx::assert_relative_eq!(first_x_in_txt, header.starting_x, epsilon = 1e-2);
        approx::assert_relative_eq!(last_x_in_txt, header.ending_x, epsilon = 1e-2);
    }

    #[test]
    fn c6d6_absorption_coefficient_header_parses_correctly() {
        let data = include_bytes!("../../test_data/header/C6D6K98.SPC");
        let mut parser = SPCFile::new(data);

        let mut header_parser = HeaderParser::new(&mut parser).unwrap();
        let result = header_parser.parse();

        assert!(result.is_ok());

        let parsed = result.unwrap();

        let header = match parsed {
            super::Header::Old(header) => header,
            _ => panic!("Expected an old format header"),
        };

        assert_eq!(header.x_unit_type, crate::xzwType::Wavenumber);
        assert_eq!(
            header.memo,
            "Imaginary refractive index of liquid C6D6 at 25 C"
        );

        let datetime = header.datetime.unwrap();
        assert_eq!(datetime.year(), 1990);

        let txt_path = include_str!("../../test_data/txt/C6D6ASC/C6D6K98.txt");
        let num_lines = BufReader::new(txt_path.as_bytes()).lines().count();
        assert_eq!(num_lines, header.number_points as usize);

        let mut reader = BufReader::new(txt_path.as_bytes()).lines();
        let first_x_in_txt: f32 = reader
            .next()
            .unwrap()
            .unwrap()
            .split_whitespace()
            .next()
            .unwrap()
            .parse()
            .unwrap();

        let last_x_in_txt: f32 = reader
            .last()
            .unwrap()
            .unwrap()
            .split_whitespace()
            .next()
            .unwrap()
            .parse()
            .unwrap();

        approx::assert_relative_eq!(first_x_in_txt, header.starting_x, epsilon = 1e-2);
        approx::assert_relative_eq!(last_x_in_txt, header.ending_x, epsilon = 1e-2);
    }

    #[test]
    fn c6d6_refractive_index_header_parses_correctly() {
        let data = include_bytes!("../../test_data/header/C6D6N98.SPC");
        let mut parser = SPCFile::new(data);

        let mut header_parser = HeaderParser::new(&mut parser).unwrap();
        let result = header_parser.parse();

        assert!(result.is_ok());

        let parsed = result.unwrap();

        let header = match parsed {
            super::Header::Old(header) => header,
            _ => panic!("Expected an old format header"),
        };

        assert_eq!(header.x_unit_type, crate::xzwType::Wavenumber);
        assert_eq!(header.memo, "real refractive index of liquid C6D6 at 25 C");

        let datetime = header.datetime.unwrap();
        assert_eq!(datetime.year(), 1994);

        let txt_path = include_str!("../../test_data/txt/C6D6ASC/C6D6N98.txt");
        let num_lines = BufReader::new(txt_path.as_bytes()).lines().count();
        assert_eq!(num_lines, header.number_points as usize);

        let mut reader = BufReader::new(txt_path.as_bytes()).lines();
        let first_x_in_txt: f32 = reader
            .next()
            .unwrap()
            .unwrap()
            .split_whitespace()
            .next()
            .unwrap()
            .parse()
            .unwrap();

        let last_x_in_txt: f32 = reader
            .last()
            .unwrap()
            .unwrap()
            .split_whitespace()
            .next()
            .unwrap()
            .parse()
            .unwrap();

        approx::assert_relative_eq!(first_x_in_txt, header.starting_x, epsilon = 1e-2);
        approx::assert_relative_eq!(last_x_in_txt, header.ending_x, epsilon = 1e-2);
    }

    #[test]
    fn c6d6_na_header_parses_correctly() {
        let data = include_bytes!("../../test_data/header/C6D6NA98.SPC");
        let mut parser = SPCFile::new(data);

        let mut header_parser = HeaderParser::new(&mut parser).unwrap();
        let result = header_parser.parse();

        assert!(result.is_ok());

        let parsed = result.unwrap();

        let header = match parsed {
            super::Header::Old(header) => header,
            _ => panic!("Expected an old format header"),
        };

        assert_eq!(header.x_unit_type, crate::xzwType::Wavenumber);
        assert_eq!(
            header.memo,
            "nu*IP of liquid C6D6 at 25 C.  Units 10^5 cm^2/mole"
        );

        let datetime = header.datetime.unwrap();
        assert_eq!(datetime.year(), 1990);

        let txt_path = include_str!("../../test_data/txt/C6D6ASC/C6D6NA98.txt");
        let num_lines = BufReader::new(txt_path.as_bytes()).lines().count();
        assert_eq!(num_lines, header.number_points as usize);

        let mut reader = BufReader::new(txt_path.as_bytes()).lines();
        let first_x_in_txt: f32 = reader
            .next()
            .unwrap()
            .unwrap()
            .split_whitespace()
            .next()
            .unwrap()
            .parse()
            .unwrap();

        let last_x_in_txt: f32 = reader
            .last()
            .unwrap()
            .unwrap()
            .split_whitespace()
            .next()
            .unwrap()
            .parse()
            .unwrap();

        approx::assert_relative_eq!(first_x_in_txt, header.starting_x, epsilon = 1e-2);
        approx::assert_relative_eq!(last_x_in_txt, header.ending_x, epsilon = 1e-2);
    }

    #[test]
    fn c6d6_rp_header_parses_correctly() {
        let data = include_bytes!("../../test_data/header/C6D6RP98.SPC");
        let mut parser = SPCFile::new(data);

        let mut header_parser = HeaderParser::new(&mut parser).unwrap();
        let result = header_parser.parse();

        assert!(result.is_ok());

        let parsed = result.unwrap();

        let header = match parsed {
            super::Header::Old(header) => header,
            _ => panic!("Expected an old format header"),
        };

        assert_eq!(header.x_unit_type, crate::xzwType::Wavenumber);
        assert_eq!(
            header.memo,
            "real molar polarizability of liquid C6D6 at 25 C.  Unit: cm^3/mole"
        );

        let datetime = header.datetime.unwrap();
        assert_eq!(datetime.year(), 1990);

        let txt_path = include_str!("../../test_data/txt/C6D6ASC/C6D6RP98.txt");
        let num_lines = BufReader::new(txt_path.as_bytes()).lines().count();
        assert_eq!(num_lines, header.number_points as usize);

        let mut reader = BufReader::new(txt_path.as_bytes()).lines();
        let first_x_in_txt: f32 = reader
            .next()
            .unwrap()
            .unwrap()
            .split_whitespace()
            .next()
            .unwrap()
            .parse()
            .unwrap();

        let last_x_in_txt: f32 = reader
            .last()
            .unwrap()
            .unwrap()
            .split_whitespace()
            .next()
            .unwrap()
            .parse()
            .unwrap();

        approx::assert_relative_eq!(first_x_in_txt, header.starting_x, epsilon = 1e-2);
        approx::assert_relative_eq!(last_x_in_txt, header.ending_x, epsilon = 1e-2);
    }

    #[test]
    fn bzh5d_dielectric_constant_header_parses_correctly() {
        let data = include_bytes!("../../test_data/header/BZH5DDC.SPC");
        let mut parser = SPCFile::new(data);

        let mut header_parser = HeaderParser::new(&mut parser).unwrap();
        let result = header_parser.parse();

        assert!(result.is_ok());

        let parsed = result.unwrap();

        let header = match parsed {
            super::Header::Old(header) => header,
            _ => panic!("Expected an old format header"),
        };

        assert_eq!(header.x_unit_type, crate::xzwType::Wavenumber);
        assert_eq!(header.memo, "(Real) dielectric constant");

        assert!(header.datetime.is_none());

        let txt_path = include_str!("../../test_data/txt/C6H5DASC/BzH5DDC.txt");
        let num_lines = BufReader::new(txt_path.as_bytes()).lines().count();
        assert_eq!(num_lines, header.number_points as usize);

        let mut reader = BufReader::new(txt_path.as_bytes()).lines();
        let first_x_in_txt: f32 = reader
            .next()
            .unwrap()
            .unwrap()
            .split_whitespace()
            .next()
            .unwrap()
            .parse()
            .unwrap();

        let last_x_in_txt: f32 = reader
            .last()
            .unwrap()
            .unwrap()
            .split_whitespace()
            .next()
            .unwrap()
            .parse()
            .unwrap();

        approx::assert_relative_eq!(first_x_in_txt, header.starting_x, epsilon = 1e-2);
        approx::assert_relative_eq!(last_x_in_txt, header.ending_x, epsilon = 1e-2);
    }

    #[test]
    fn bzh5d_dielectric_loss_header_parses_correctly() {
        let data = include_bytes!("../../test_data/header/BZH5DDL.SPC");
        let mut parser = SPCFile::new(data);

        let mut header_parser = HeaderParser::new(&mut parser).unwrap();
        let result = header_parser.parse();

        assert!(result.is_ok());

        let parsed = result.unwrap();

        let header = match parsed {
            super::Header::Old(header) => header,
            _ => panic!("Expected an old format header"),
        };

        assert_eq!(header.x_unit_type, crate::xzwType::Wavenumber);
        assert_eq!(
            header.memo,
            "Dielectric loss, i.e., imaginary dielectric constant"
        );

        assert!(header.datetime.is_none());

        let txt_path = include_str!("../../test_data/txt/C6H5DASC/BzH5DDL.txt");
        let num_lines = BufReader::new(txt_path.as_bytes()).lines().count();
        assert_eq!(num_lines, header.number_points as usize);

        let mut reader = BufReader::new(txt_path.as_bytes()).lines();
        let first_x_in_txt: f32 = reader
            .next()
            .unwrap()
            .unwrap()
            .split_whitespace()
            .next()
            .unwrap()
            .parse()
            .unwrap();

        let last_x_in_txt: f32 = reader
            .last()
            .unwrap()
            .unwrap()
            .split_whitespace()
            .next()
            .unwrap()
            .parse()
            .unwrap();

        approx::assert_relative_eq!(first_x_in_txt, header.starting_x, epsilon = 1e-2);
        approx::assert_relative_eq!(last_x_in_txt, header.ending_x, epsilon = 1e-2);
    }

    #[test]
    fn bzh5d_molar_absorptivity_header_parses_correctly() {
        let data = include_bytes!("../../test_data/header/BZH5DEM.SPC");
        let mut parser = SPCFile::new(data);

        let mut header_parser = HeaderParser::new(&mut parser).unwrap();
        let result = header_parser.parse();

        assert!(result.is_ok());

        let parsed = result.unwrap();

        let header = match parsed {
            super::Header::Old(header) => header,
            _ => panic!("Expected an old format header"),
        };

        assert_eq!(header.x_unit_type, crate::xzwType::Wavenumber);
        assert_eq!(
            header.memo,
            "Decadic Molar Absorption Coefficient in L/(mole-cm)"
        );

        assert!(header.datetime.is_none());

        let txt_path = include_str!("../../test_data/txt/C6H5DASC/BzH5DEM.txt");
        let num_lines = BufReader::new(txt_path.as_bytes()).lines().count();
        assert_eq!(num_lines, header.number_points as usize);

        let mut reader = BufReader::new(txt_path.as_bytes()).lines();
        let first_x_in_txt: f32 = reader
            .next()
            .unwrap()
            .unwrap()
            .split_whitespace()
            .next()
            .unwrap()
            .parse()
            .unwrap();

        let last_x_in_txt: f32 = reader
            .last()
            .unwrap()
            .unwrap()
            .split_whitespace()
            .next()
            .unwrap()
            .parse()
            .unwrap();

        approx::assert_relative_eq!(first_x_in_txt, header.starting_x, epsilon = 1e-2);
        approx::assert_relative_eq!(last_x_in_txt, header.ending_x, epsilon = 1e-2);
    }

    #[test]
    fn bzh5d_imaginary_molar_polarisability_header_parses_correctly() {
        let data = include_bytes!("../../test_data/header/BZH5DIP.SPC");
        let mut parser = SPCFile::new(data);

        let mut header_parser = HeaderParser::new(&mut parser).unwrap();
        let result = header_parser.parse();

        assert!(result.is_ok());

        let parsed = result.unwrap();

        let header = match parsed {
            super::Header::Old(header) => header,
            _ => panic!("Expected an old format header"),
        };

        assert_eq!(header.x_unit_type, crate::xzwType::Wavenumber);
        assert_eq!(
            header.memo,
            "IP, Imaginary molar polarizability, in cm^3/mole"
        );

        assert!(header.datetime.is_none());

        let txt_path = include_str!("../../test_data/txt/C6H5DASC/BzH5DIP.txt");
        let num_lines = BufReader::new(txt_path.as_bytes()).lines().count();
        assert_eq!(num_lines, header.number_points as usize);

        let mut reader = BufReader::new(txt_path.as_bytes()).lines();
        let first_x_in_txt: f32 = reader
            .next()
            .unwrap()
            .unwrap()
            .split_whitespace()
            .next()
            .unwrap()
            .parse()
            .unwrap();

        let last_x_in_txt: f32 = reader
            .last()
            .unwrap()
            .unwrap()
            .split_whitespace()
            .next()
            .unwrap()
            .parse()
            .unwrap();

        approx::assert_relative_eq!(first_x_in_txt, header.starting_x, epsilon = 1e-2);
        approx::assert_relative_eq!(last_x_in_txt, header.ending_x, epsilon = 1e-2);
    }

    #[test]
    fn bzh5d_absorption_coefficient_header_parses_correctly() {
        let data = include_bytes!("../../test_data/header/BZH5DK.SPC");
        let mut parser = SPCFile::new(data);

        let mut header_parser = HeaderParser::new(&mut parser).unwrap();
        let result = header_parser.parse();

        assert!(result.is_ok());

        let parsed = result.unwrap();

        let header = match parsed {
            super::Header::Old(header) => header,
            _ => panic!("Expected an old format header"),
        };

        assert_eq!(header.x_unit_type, crate::xzwType::Wavenumber);
        assert_eq!(header.memo, "imaginary refractive index of C6H5D at 25 C");

        assert!(header.datetime.is_some());
        assert_eq!(header.datetime.unwrap().year(), 1990);

        let txt_path = include_str!("../../test_data/txt/C6H5DASC/BzH5DK.txt");
        let num_lines = BufReader::new(txt_path.as_bytes()).lines().count();
        assert_eq!(num_lines, header.number_points as usize);

        let mut reader = BufReader::new(txt_path.as_bytes()).lines();
        let first_x_in_txt: f32 = reader
            .next()
            .unwrap()
            .unwrap()
            .split_whitespace()
            .next()
            .unwrap()
            .parse()
            .unwrap();

        let last_x_in_txt: f32 = reader
            .last()
            .unwrap()
            .unwrap()
            .split_whitespace()
            .next()
            .unwrap()
            .parse()
            .unwrap();

        approx::assert_relative_eq!(first_x_in_txt, header.starting_x, epsilon = 1e-2);
        approx::assert_relative_eq!(last_x_in_txt, header.ending_x, epsilon = 1e-2);
    }

    #[test]
    fn bzh5d_refractive_index_header_parses_correctly() {
        let data = include_bytes!("../../test_data/header/BZH5DN.SPC");
        let mut parser = SPCFile::new(data);

        let mut header_parser = HeaderParser::new(&mut parser).unwrap();
        let result = header_parser.parse();

        assert!(result.is_ok());

        let parsed = result.unwrap();

        let header = match parsed {
            super::Header::Old(header) => header,
            _ => panic!("Expected an old format header"),
        };

        assert_eq!(header.x_unit_type, crate::xzwType::Wavenumber);
        assert_eq!(header.memo, "Real refractive index of C6H5D at 25 C");

        assert!(header.datetime.is_some());
        assert_eq!(header.datetime.unwrap().year(), 1994);

        let txt_path = include_str!("../../test_data/txt/C6H5DASC/BzH5DN.txt");
        let num_lines = BufReader::new(txt_path.as_bytes()).lines().count();
        assert_eq!(num_lines, header.number_points as usize);

        let mut reader = BufReader::new(txt_path.as_bytes()).lines();
        let first_x_in_txt: f32 = reader
            .next()
            .unwrap()
            .unwrap()
            .split_whitespace()
            .next()
            .unwrap()
            .parse()
            .unwrap();

        let last_x_in_txt: f32 = reader
            .last()
            .unwrap()
            .unwrap()
            .split_whitespace()
            .next()
            .unwrap()
            .parse()
            .unwrap();

        approx::assert_relative_eq!(first_x_in_txt, header.starting_x, epsilon = 1e-2);
        approx::assert_relative_eq!(last_x_in_txt, header.ending_x, epsilon = 1e-2);
    }

    #[test]
    fn bzh5d_na_header_parses_correctly() {
        let data = include_bytes!("../../test_data/header/BZH5DNA.SPC");
        let mut parser = SPCFile::new(data);

        let mut header_parser = HeaderParser::new(&mut parser).unwrap();
        let result = header_parser.parse();

        assert!(result.is_ok());

        let parsed = result.unwrap();

        let header = match parsed {
            super::Header::Old(header) => header,
            _ => panic!("Expected an old format header"),
        };

        assert_eq!(header.x_unit_type, crate::xzwType::Wavenumber);
        assert_eq!(header.memo, "nu*IP in cm^2/mole");

        assert!(header.datetime.is_none());

        let txt_path = include_str!("../../test_data/txt/C6H5DASC/BzH5DNA.txt");
        let num_lines = BufReader::new(txt_path.as_bytes()).lines().count();
        assert_eq!(num_lines, header.number_points as usize);

        let mut reader = BufReader::new(txt_path.as_bytes()).lines();
        let first_x_in_txt: f32 = reader
            .next()
            .unwrap()
            .unwrap()
            .split_whitespace()
            .next()
            .unwrap()
            .parse()
            .unwrap();

        let last_x_in_txt: f32 = reader
            .last()
            .unwrap()
            .unwrap()
            .split_whitespace()
            .next()
            .unwrap()
            .parse()
            .unwrap();

        approx::assert_relative_eq!(first_x_in_txt, header.starting_x, epsilon = 1e-2);
        approx::assert_relative_eq!(last_x_in_txt, header.ending_x, epsilon = 1e-2);
    }

    #[test]
    fn bzh5d_rp_header_parses_correctly() {
        let data = include_bytes!("../../test_data/header/BZH5DRP.SPC");
        let mut parser = SPCFile::new(data);

        let mut header_parser = HeaderParser::new(&mut parser).unwrap();
        let result = header_parser.parse();

        assert!(result.is_ok());

        let parsed = result.unwrap();

        let header = match parsed {
            super::Header::Old(header) => header,
            _ => panic!("Expected an old format header"),
        };

        assert_eq!(header.x_unit_type, crate::xzwType::Wavenumber);
        assert_eq!(header.memo, "RP, real molar polarizability, in cm^3/mole");

        assert!(header.datetime.is_none());

        let txt_path = include_str!("../../test_data/txt/C6H5DASC/BzH5DRP.txt");
        let num_lines = BufReader::new(txt_path.as_bytes()).lines().count();
        assert_eq!(num_lines, header.number_points as usize);

        let mut reader = BufReader::new(txt_path.as_bytes()).lines();
        let first_x_in_txt: f32 = reader
            .next()
            .unwrap()
            .unwrap()
            .split_whitespace()
            .next()
            .unwrap()
            .parse()
            .unwrap();

        let last_x_in_txt: f32 = reader
            .last()
            .unwrap()
            .unwrap()
            .split_whitespace()
            .next()
            .unwrap()
            .parse()
            .unwrap();

        approx::assert_relative_eq!(first_x_in_txt, header.starting_x, epsilon = 1e-2);
        approx::assert_relative_eq!(last_x_in_txt, header.ending_x, epsilon = 1e-2);
    }
}
