// case 1:
//       return 'Gas Chromatogram';
//     case 2:
//       return 'General Chromatogram (same as SPCGEN with TCGRAM)';
//     case 3:
//       return 'HPLC Chromatogram';
//     case 4:
//       return 'FT-IR, FT-NIR, FT-Raman Spectrum or Igram (Can also be used for scanning IR.)';
//     case 5:
//       return 'NIR Spectrum (Usually multi-spectral data sets for calibration.)';
//     case 7:
//       return 'UV-VIS Spectrum (Can be used for single scanning UV-VIS-NIR.)';
//     case 8:
//       return 'X-ray Diffraction Spectrum';
//     case 9:
//       return 'Mass Spectrum  (Can be single, GC-MS, Continuum, Centroid or TOF.)';
//     case 10:
//       return 'NMR Spectrum or FID';
//     case 11:
//       return 'Raman Spectrum (Usually Diode Array, CCD, etc. use SPCFTIR for FT-Raman.)';
//     case 12:
//       return 'Fluorescence Spectrum';
//     case 13:
//       return 'Atomic Spectrum';
//     case 14:
//       return 'Chromatography Diode Array Spectra';
//     default:
//       return 'General SPC (could be anything)';
//
#[derive(Copy, Clone, Debug)]
pub(crate) enum ExperimentSettings {
    GeneralSPC = 0x00,
    GasChromatogram = 0x01,
    GeneralChromatogram = 0x02,
    HPLCChromatogram = 0x03,
    FTIRFTNIRFTRaman = 0x04,
    NIRSpectrum = 0x05,
    UVVISSpectrum = 0x06,
    XRayDiffractionSpectrum = 0x08,
    MassSpectrum = 0x09,
    NMRSpectrum = 0x0A,
    RamanSpectrum = 0x0B,
    FluorescenceSpectrum = 0x0C,
    AtomicSpectrum = 0x0D,
    ChromatographyDiodeArraySpectra = 0x0E,
}

impl ExperimentSettings {
    pub(crate) fn new(val: u8) -> Option<Self> {
        match val {
            1 => Some(Self::GasChromatogram),
            2 => Some(Self::GeneralChromatogram),
            3 => Some(Self::HPLCChromatogram),
            4 => Some(Self::FTIRFTNIRFTRaman),
            5 => Some(Self::NIRSpectrum),
            7 => Some(Self::UVVISSpectrum),
            8 => Some(Self::XRayDiffractionSpectrum),
            9 => Some(Self::MassSpectrum),
            10 => Some(Self::NMRSpectrum),
            11 => Some(Self::RamanSpectrum),
            12 => Some(Self::FluorescenceSpectrum),
            13 => Some(Self::AtomicSpectrum),
            14 => Some(Self::ChromatographyDiodeArraySpectra),
            0 => Some(Self::GeneralSPC),
            _ => None,
        }
    }
}

// export function xzwTypes(xzwType: number): string | number {
//   switch (xzwType) {
//     case 1:
//       return 'Wavenumber (cm-1)';
//     case 2:
//       return 'Micrometers (um)';
//     case 3:
//       return 'Nanometers (nm)';
//     case 4:
//       return 'Seconds';
//     case 5:
//       return 'Minutes';
//     case 6:
//       return 'Hertz (Hz)';
//     case 7:
//       return 'Kilohertz (KHz)';
//     case 8:
//       return 'Megahertz (MHz)';
//     case 9:
//       return 'Mass (M/z)';
//     case 10:
//       return 'Parts per million (PPM)';
//     case 11:
//       return 'Days';
//     case 12:
//       return 'Years';
//     case 13:
//       return 'Raman Shift (cm-1)';
//     case 14:
//       return 'eV';
//     case 15:
//       return 0;
//     case 16:
//       return 'Diode Number';
//     case 17:
//       return 'Channel ';
//     case 18:
//       return 'Degrees';
//     case 19:
//       return 'Temperature (F)';
//     case 20:
//       return 'Temperature (C)';
//     case 21:
//       return 'Temperature (K)';
//     case 22:
//       return 'Data Points';
//     case 23:
//       return 'Milliseconds (mSec)';
//     case 24:
//       return 'Microseconds (uSec)';
//     case 25:
//       return 'Nanoseconds (nSec)';
//     case 26:
//       return 'Gigahertz (GHz)';
//     case 27:
//       return 'Centimeters (cm)';
//     case 28:
//       return 'Meters (m)';
//     case 29:
//       return 'Millimeters (mm)';
//     case 30:
//       return 'Hours';
//     case 255:
//       return 'Double interferogram';
//     default:
//       return 'Arbitrary';
//   }
// }

#[derive(Copy, Clone, Debug)]
pub(crate) enum xzwType {
    Arbitrary = 0,
    Wavenumber = 1,
    Micrometers = 2,
    Nanometers = 3,
    Seconds = 4,
    Minutes = 5,
    Hertz = 6,
    Kilohertz = 7,
    MegaHertz = 8,
    Mass = 9,
    PartsPerMillion = 10,
    Days = 11,
    Years = 12,
    RamanShift = 13,
    ElectronVolt = 14,
    // TODO: Check the spec
    Unknown = 15,
    DiodeNumber = 16,
    Channel = 17,
    Degrees = 18,
    TemperatureF = 19,
    TemperatureC = 20,
    TemperatureK = 21,
    DataPoints = 22,
    Milliseconds = 23,
    Microseconds = 24,
    Nanoseconds = 25,
    GigaHertz = 26,
    Centimeters = 27,
    Meters = 28,
    Millimeters = 29,
    Hours = 30,
    DoubleInterferogram = 255,
}

impl xzwType {
    pub(crate) fn new(val: u8) -> Option<Self> {
        match val {
            // TODO: Should we return arbitrary for all non-enumerated values?
            0 => Some(Self::Arbitrary),
            1 => Some(Self::Wavenumber),
            2 => Some(Self::Micrometers),
            3 => Some(Self::Nanometers),
            4 => Some(Self::Seconds),
            5 => Some(Self::Minutes),
            6 => Some(Self::Hertz),
            7 => Some(Self::Kilohertz),
            8 => Some(Self::MegaHertz),
            9 => Some(Self::Mass),
            10 => Some(Self::PartsPerMillion),
            11 => Some(Self::Days),
            12 => Some(Self::Years),
            13 => Some(Self::RamanShift),
            14 => Some(Self::ElectronVolt),
            15 => Some(Self::Unknown),
            16 => Some(Self::DiodeNumber),
            17 => Some(Self::Channel),
            18 => Some(Self::Degrees),
            19 => Some(Self::TemperatureF),
            20 => Some(Self::TemperatureC),
            21 => Some(Self::TemperatureK),
            22 => Some(Self::DataPoints),
            23 => Some(Self::Milliseconds),
            24 => Some(Self::Microseconds),
            25 => Some(Self::Nanoseconds),
            26 => Some(Self::GigaHertz),
            27 => Some(Self::Centimeters),
            28 => Some(Self::Meters),
            29 => Some(Self::Millimeters),
            30 => Some(Self::Hours),
            255 => Some(Self::DoubleInterferogram),
            _ => None,
        }
    }
}

// /**
//  * Gives meaning to y type codes
//  * @param  yType y type code
//  * @return  String corresponding to the code
//  */
// export function yTypes(yType: number): string {
//   switch (yType) {
//     case 0:
//       return 'Arbitrary Intensity';
//     case 1:
//       return 'Interferogram';
//     case 2:
//       return 'Absorbance';
//     case 3:
//       return 'Kubelka-Monk';
//     case 4:
//       return 'Counts';
//     case 5:
//       return 'Volts';
//     case 6:
//       return 'Degrees';
//     case 7:
//       return 'Milliamps';
//     case 8:
//       return 'Millimeters';
//     case 9:
//       return 'Millivolts';
//     case 10:
//       return 'Log(1/R)';
//     case 11:
//       return 'Percent';
//     case 12:
//       return 'Intensity';
//     case 13:
//       return 'Relative Intensity';
//     case 14:
//       return 'Energy';
//     case 16:
//       return 'Decibel';
//     case 19:
//       return 'Temperature (F)';
//     case 20:
//       return 'Temperature (C)';
//     case 21:
//       return 'Temperature (K)';
//     case 22:
//       return 'Index of Refraction [N]';
//     case 23:
//       return 'Extinction Coeff. [K]';
//     case 24:
//       return 'Real';
//     case 25:
//       return 'Imaginary';
//     case 26:
//       return 'Complex';
//     case 128:
//       return 'Transmission';
//     case 129:
//       return 'Reflectance';
//     case 130:
//       return 'Arbitrary or Single Beam with Valley Peaks';
//     case 131:
//       return 'Emission';
//     default:
//       return 'Reference Arbitrary Energy';
//   }
// }
//
#[derive(Copy, Clone, Debug)]
pub(crate) enum yType {
    ArbitraryIntensity = 0,
    Interferogram = 1,
    Absorbance = 2,
    KubelkaMonk = 3,
    Counts = 4,
    Volts = 5,
    Degrees = 6,
    Milliamps = 7,
    Millimeters = 8,
    Millivolts = 9,
    LogInvR = 10,
    Percent = 11,
    Intensity = 12,
    RelativeIntensity = 13,
    Energy = 14,
    Decibel = 15,
    TemperatureF = 19,
    TemperatureC = 20,
    TemperatureK = 21,
    IndexOfRefraction = 22,
    ExtinctionCoeff = 23,
    Real = 24,
    Imaginary = 25,
    Complex = 26,
    Transmission = 128,
    Reflectance = 129,
    ArbitraryOrSingleBeamWithValleyPeaks = 130,
    Emission = 131,
}

impl yType {
    pub(crate) fn new(val: u8) -> Option<Self> {
        match val {
            0 => Some(Self::ArbitraryIntensity),
            1 => Some(Self::Interferogram),
            2 => Some(Self::Absorbance),
            3 => Some(Self::KubelkaMonk),
            4 => Some(Self::Counts),
            5 => Some(Self::Volts),
            6 => Some(Self::Degrees),
            7 => Some(Self::Milliamps),
            8 => Some(Self::Millimeters),
            9 => Some(Self::Millivolts),
            10 => Some(Self::LogInvR),
            11 => Some(Self::Percent),
            12 => Some(Self::Intensity),
            13 => Some(Self::RelativeIntensity),
            14 => Some(Self::Energy),
            15 => Some(Self::Decibel),
            19 => Some(Self::TemperatureF),
            20 => Some(Self::TemperatureC),
            21 => Some(Self::TemperatureK),
            22 => Some(Self::IndexOfRefraction),
            23 => Some(Self::ExtinctionCoeff),
            24 => Some(Self::Real),
            25 => Some(Self::Imaginary),
            26 => Some(Self::Complex),
            128 => Some(Self::Transmission),
            129 => Some(Self::Reflectance),
            130 => Some(Self::ArbitraryOrSingleBeamWithValleyPeaks),
            131 => Some(Self::Emission),
            _ => None,
        }
    }
}
