mod flags;
mod subheader;

pub(crate) use flags::DataShape;
pub(crate) use subheader::{SubHeaderParser, Subheader};

use crate::{xzwType, yType, ExperimentSettings, SPCFile};
use flags::FlagParameters;

use chrono::{DateTime, LocalResult, TimeZone, Utc};

#[derive(Clone, Debug)]
pub(crate) enum Header {
    Old(OldFormatHeader),
    New(NewFormatHeader),
}

#[derive(Clone, Debug)]
pub(crate) struct OldFormatHeader {
    pub(super) file_version: u8,
    pub(super) flags: FlagParameters,
    pub(super) exponent_y: i16,
    pub(super) number_points: f32,
    pub(super) starting_x: f32,
    pub(super) ending_x: f32,
    pub(super) x_unit_type: xzwType,
    pub(super) y_unit_type: yType,
    pub(super) datetime: DateTime<Utc>,
    pub(super) resolution_description: String,
    pub(super) peak_point_number: u16,
    pub(super) scans: u16,
    pub(super) spare: [f32; 7],
    pub(super) memo: String,
    pub(super) xyz_labels: String,
}

#[derive(Clone, Debug)]
pub(crate) struct NewFormatHeader {
    pub(super) file_version: u8,
    pub(super) flags: FlagParameters,
    pub(super) experiment_settings: ExperimentSettings,
    pub(super) exponent_y: i8,
    pub(super) number_points: u32,
    pub(super) starting_x: f64,
    pub(super) ending_x: f64,
    pub(super) spectra: u32,
    pub(super) x_unit_type: xzwType,
    pub(super) y_unit_type: yType,
    pub(super) z_unit_type: xzwType,
    pub(super) posting_disposition: u8,
    pub(super) resolution_description: String,
    pub(super) source_instrument_description: String,
    pub(super) peak_point_number: u16,
    pub(super) spare: [f32; 8],
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
    pub(super) reserved: String,
}

pub(crate) struct HeaderParser<'a, 'de> {
    spc: &'a mut SPCFile<'de>,
    flags: FlagParameters,
    version: u8,
}

impl<'a, 'de> HeaderParser<'a, 'de> {
    pub(crate) fn new(spc: &'a mut SPCFile<'de>) -> Self {
        let flags = FlagParameters(spc.read_byte());

        let version = spc.read_byte();

        if !matches!(version, 0x4b..=0x4d) {
            panic!("Unsupported SPC version: {}", version);
        }

        Self {
            spc,
            flags,
            version,
        }
    }

    pub(crate) fn parse(&mut self) -> miette::Result<Header> {
        match self.version {
            0x4d => {
                let header = self.parse_old_format()?;

                Ok(Header::Old(header))
            }
            0x4b | 0x4c => {
                let header = self.parse_new_format()?;
                Ok(Header::New(header))
            }
            _ => unreachable!("can only create a header constructor with a valid file version"),
        }
    }

    fn parse_new_format(&mut self) -> miette::Result<NewFormatHeader> {
        let experiment_settings = ExperimentSettings::new(self.spc.read_byte()).unwrap();
        let exponent_y = self.spc.read_i8();
        let number_points = self.spc.read_u32();
        let starting_x = self.spc.read_f64();
        let ending_x = self.spc.read_f64();
        let spectra = self.spc.read_u32();
        let x_unit_type = xzwType::new(self.spc.read_byte()).unwrap();
        let y_unit_type = yType::new(self.spc.read_byte()).unwrap();
        let z_unit_type = xzwType::new(self.spc.read_byte()).unwrap();

        let posting_disposition = self.spc.read_byte();

        let date = self.spc.read_u32();

        let resolution_description = self.spc.read_unescaped_utf8(9).trim().to_owned();
        let source_instrument_description = self.spc.read_unescaped_utf8(9).trim().to_owned();

        let peak_point_number = self.spc.read_u16();

        let mut spare = [0.0; 8];
        for ii in 0..8 {
            match self.version {
                0x4b => spare[ii] = self.spc.read_f32(),
                0x4c => spare[8 - 1 - ii] = self.spc.read_f32(),
                _ => unreachable!(),
            }
        }

        let memo = self.spc.read_unescaped_utf8(130).trim().to_owned();
        let xyz_labels = self.spc.read_unescaped_utf8(30).trim().to_owned();

        let log_offset = self.spc.read_u32();
        let modified_flag = self.spc.read_u32();
        let processing_code = self.spc.read_byte();
        let calibration_level = self.spc.read_byte();
        let sub_method_sample_injection_number = self.spc.read_u16();

        let concentration_factor = self.spc.read_f32();
        let method_file = self.spc.read_unescaped_utf8(48).trim().to_owned();

        let z_sub_increment = self.spc.read_f32();
        let w_planes = self.spc.read_u32();
        let w_plane_increment = self.spc.read_f32();
        let w_axis_units = xzwType::new(self.spc.read_byte()).unwrap();
        let reserved = self.spc.read_unescaped_utf8(187).trim().to_owned();

        Ok(NewFormatHeader {
            file_version: self.version,
            flags: self.flags,
            experiment_settings,
            exponent_y,
            number_points,
            starting_x,
            ending_x,
            spectra,
            x_unit_type,
            y_unit_type,
            z_unit_type,
            posting_disposition,
            resolution_description,
            source_instrument_description,
            peak_point_number,
            spare,
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
            reserved,
        })
    }

    fn parse_old_format(&mut self) -> miette::Result<OldFormatHeader> {
        let exponent_y = self.spc.read_i16();
        let number_points = self.spc.read_f32();
        let starting_x = self.spc.read_f32();
        let ending_x = self.spc.read_f32();

        let x_unit_type = xzwType::new(self.spc.read_byte()).unwrap();
        let y_unit_type = yType::new(self.spc.read_byte()).unwrap();

        let zTypeYear = self.spc.read_u16();
        let month = (self.spc.read_byte() - 1).max(0);
        let date = self.spc.read_byte();
        let hours = self.spc.read_byte();
        let minutes = self.spc.read_byte();

        let datetime = match Utc.with_ymd_and_hms(
            zTypeYear as i32,
            month as u32,
            date as u32,
            hours as u32,
            minutes as u32,
            0,
        ) {
            LocalResult::Single(datetime) => datetime,
            _ => panic!(),
        };

        let resolution_description = self.spc.read_unescaped_utf8(8).trim().to_owned();
        let peak_point_number = self.spc.read_u16();
        let scans = self.spc.read_u16();

        let mut spare = [0.0; 7];
        for ii in 0..7 {
            spare[ii] = self.spc.read_f32();
        }

        let memo = self.spc.read_unescaped_utf8(130).trim().to_owned();
        let xyz_labels = self.spc.read_unescaped_utf8(30).trim().to_owned();

        Ok(OldFormatHeader {
            file_version: self.version,
            flags: self.flags,
            exponent_y,
            number_points,
            starting_x,
            ending_x,
            x_unit_type,
            y_unit_type,
            datetime,
            resolution_description,
            peak_point_number,
            scans,
            spare,
            memo,
            xyz_labels,
        })
    }
}
