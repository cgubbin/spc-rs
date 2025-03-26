
/// The [`InstrumentTechnique`] represents all the possible values taken by the third byte in a new
/// style header
///
/// This refers to the instrument technique code. Note that in older software packages the TCGRAM
/// flag in [`FlagParameters`] must be set when fexpr is non-zero. When TCGRAM is set, a general
/// chromatagraph is specified by a zero field
#[derive(Copy, Clone, Debug)]
pub(crate) enum InstrumentTechnique {
    /// A general SPC file, which could be anything at all
    GeneralSPC = 0x00,
    /// A gas chromatogram
    GasChromatogram = 0x01,
    /// A general chromatogram. This is the equivalent to 0x00 with TCGRAM set in
    /// [`FlagParameters`]
    GeneralChromatogram = 0x02,
    /// A high performance liquid chromatogram
    HPLCChromatogram = 0x03,
    /// Fourier Transform Infrared, Fourier Transform Near Infrared or Fourier Transform Raman
    /// spectrum or igram.
    FTIRFTNIRFTRaman = 0x04,
    /// A near-infrared spectrum
    NIRSpectrum = 0x05,
    /// A UV-Visible spectrum
    UVVISSpectrum = 0x06,
    /// An X-ray diffraction spectrum
    XRayDiffractionSpectrum = 0x08,
    /// A mass-spectrum, which can be single, GC-MS, continuum, centroid or time-of-flight
    MassSpectrum = 0x09,
    /// A nuclear magnetic resonance spectrum or free induction decay
    NMRSpectrum = 0x0A,
    /// A Raman spectrum, note that 0x04 is used for Fourier-transform Raman
    RamanSpectrum = 0x0B,
    /// A fluorescence spectrum
    FluorescenceSpectrum = 0x0C,
    /// An atomic spectrum
    AtomicSpectrum = 0x0D,
    /// A chromatography diode array spectra
    ChromatographyDiodeArraySpectra = 0x0E,
}

#[derive(Debug, thiserror::Error)]
#[error("Invalid InstrumentTechnique value: {0}")]
pub(crate) struct InstrumentTechniqueCreationError(u8);

impl InstrumentTechnique {
    pub(crate) fn new(val: u8) -> Result<Self, InstrumentTechniqueCreationError> {
        match val {
            1 => Ok(Self::GasChromatogram),
            2 => Ok(Self::GeneralChromatogram),
            3 => Ok(Self::HPLCChromatogram),
            4 => Ok(Self::FTIRFTNIRFTRaman),
            5 => Ok(Self::NIRSpectrum),
            7 => Ok(Self::UVVISSpectrum),
            8 => Ok(Self::XRayDiffractionSpectrum),
            9 => Ok(Self::MassSpectrum),
            10 => Ok(Self::NMRSpectrum),
            11 => Ok(Self::RamanSpectrum),
            12 => Ok(Self::FluorescenceSpectrum),
            13 => Ok(Self::AtomicSpectrum),
            14 => Ok(Self::ChromatographyDiodeArraySpectra),
            0 => Ok(Self::GeneralSPC),
            v => Err(InstrumentTechniqueCreationError(v)),
        }
    }
}

/// The [`xzwType`] represents all the possible settings for the fxtype, fztype and fwtype
#[derive(Copy, Clone, Debug, Eq, PartialEq)]
pub(crate) enum xzwType {
    // Arbitrary
    Arbitrary = 0,
    /// Wavenumber (cm-1)
    Wavenumber = 1,
    /// Micrometers (um)
    Micrometers = 2,
    /// Nanometers (nm)
    Nanometers = 3,
    /// Seconds
    Seconds = 4,
    /// Minutes
    Minutes = 5,
    /// Hertz (Hz)
    Hertz = 6,
    /// Kilohertz (KHz)
    Kilohertz = 7,
    /// Megahertz (MHz)
    MegaHertz = 8,
    /// Mass (M/z)
    Mass = 9,
    /// Parts per Million (PPM)
    PartsPerMillion = 10,
    /// Days
    Days = 11,
    /// Years
    Years = 12,
    // Raman shift (cm-1)
    RamanShift = 13,
    /// ElectronVolt (eV)
    ElectronVolt = 14,
    /// XYZ text labels are to be found in fcatxt (only in old style-headers)
    Unknown = 15,
    /// Diode Number
    DiodeNumber = 16,
    /// Channel
    Channel = 17,
    /// Degrees
    Degrees = 18,
    /// Temperature (F)
    TemperatureF = 19,
    /// Temperature (C)
    TemperatureC = 20,
    /// Temperature (K)
    TemperatureK = 21,
    /// Datapoints
    DataPoints = 22,
    /// Milliseconds (mS)
    Milliseconds = 23,
    /// Microseconds (uS)
    Microseconds = 24,
    /// Nanoseconds (nS)
    Nanoseconds = 25,
    /// GiagHertz (GHz)
    GigaHertz = 26,
    /// Centimetres (cm)
    Centimeters = 27,
    /// Metres (m)
    Meters = 28,
    /// Millimetres (mm)
    Millimeters = 29,
    /// Hours
    Hours = 30,
    /// Double interferogram, no display labels
    DoubleInterferogram = 255,
}

#[derive(Debug, thiserror::Error)]
#[error("Invalid xzwType value: {0}")]
pub(crate) struct xzwTypeCreationError(u8);

impl xzwType {
    pub(crate) fn new(val: u8) -> Result<Self, xzwTypeCreationError> {
        match val {
            // TODO: Should we return arbitrary for all non-enumerated values?
            0 => Ok(Self::Arbitrary),
            1 => Ok(Self::Wavenumber),
            2 => Ok(Self::Micrometers),
            3 => Ok(Self::Nanometers),
            4 => Ok(Self::Seconds),
            5 => Ok(Self::Minutes),
            6 => Ok(Self::Hertz),
            7 => Ok(Self::Kilohertz),
            8 => Ok(Self::MegaHertz),
            9 => Ok(Self::Mass),
            10 => Ok(Self::PartsPerMillion),
            11 => Ok(Self::Days),
            12 => Ok(Self::Years),
            13 => Ok(Self::RamanShift),
            14 => Ok(Self::ElectronVolt),
            15 => Ok(Self::Unknown),
            16 => Ok(Self::DiodeNumber),
            17 => Ok(Self::Channel),
            18 => Ok(Self::Degrees),
            19 => Ok(Self::TemperatureF),
            20 => Ok(Self::TemperatureC),
            21 => Ok(Self::TemperatureK),
            22 => Ok(Self::DataPoints),
            23 => Ok(Self::Milliseconds),
            24 => Ok(Self::Microseconds),
            25 => Ok(Self::Nanoseconds),
            26 => Ok(Self::GigaHertz),
            27 => Ok(Self::Centimeters),
            28 => Ok(Self::Meters),
            29 => Ok(Self::Millimeters),
            30 => Ok(Self::Hours),
            255 => Ok(Self::DoubleInterferogram),
            v => Err(xzwTypeCreationError(v)),
        }
    }
}

/// The [`yType`] represents all the possible settings for the fytype. Note that all the first 127
/// values exhibit positive peaks, while values 129 or greater are expected to exhibit valleys
#[derive(Copy, Clone, Debug)]
pub(crate) enum yType {
    /// Arbitrary intensity
    ArbitraryIntensity = 0,
    /// Interferogram
    Interferogram = 1,
    /// Absorbance
    Absorbance = 2,
    /// Kubelka-Monk
    KubelkaMonk = 3,
    /// Counts
    Counts = 4,
    /// Volts
    Volts = 5,
    /// Degrees
    Degrees = 6,
    /// Milliamps
    Milliamps = 7,
    /// Millimeters
    Millimeters = 8,
    /// Millivolts
    Millivolts = 9,
    /// Log(1/R)
    LogInvR = 10,
    /// Percent
    Percent = 11,
    /// Intensity
    Intensity = 12,
    /// Relative intensity
    RelativeIntensity = 13,
    /// Energy
    Energy = 14,
    /// Decibel
    Decibel = 15,
    /// Temperature (F)
    TemperatureF = 19,
    /// Temperature (C)
    TemperatureC = 20,
    /// Temperature (K)
    TemperatureK = 21,
    /// Index of Refraction [N]
    IndexOfRefraction = 22,
    /// Index of Refraction [K]
    ExtinctionCoeff = 23,
    /// Real
    Real = 24,
    /// Imaginary
    Imaginary = 25,
    /// Complex
    Complex = 26,
    /// Transmission
    Transmission = 128,
    /// Reflectance
    Reflectance = 129,
    /// Arbitrary or Single Beam with Valley Peaks
    ArbitraryOrSingleBeamWithValleyPeaks = 130,
    /// Emission
    Emission = 131,
}

#[derive(Debug, thiserror::Error)]
#[error("Invalid yType value: {0}")]
pub(crate) struct yTypeCreationError(u8);

impl yType {
    pub(crate) fn new(val: u8) -> Result<Self, yTypeCreationError> {
        match val {
            0 => Ok(Self::ArbitraryIntensity),
            1 => Ok(Self::Interferogram),
            2 => Ok(Self::Absorbance),
            3 => Ok(Self::KubelkaMonk),
            4 => Ok(Self::Counts),
            5 => Ok(Self::Volts),
            6 => Ok(Self::Degrees),
            7 => Ok(Self::Milliamps),
            8 => Ok(Self::Millimeters),
            9 => Ok(Self::Millivolts),
            10 => Ok(Self::LogInvR),
            11 => Ok(Self::Percent),
            12 => Ok(Self::Intensity),
            13 => Ok(Self::RelativeIntensity),
            14 => Ok(Self::Energy),
            15 => Ok(Self::Decibel),
            19 => Ok(Self::TemperatureF),
            20 => Ok(Self::TemperatureC),
            21 => Ok(Self::TemperatureK),
            22 => Ok(Self::IndexOfRefraction),
            23 => Ok(Self::ExtinctionCoeff),
            24 => Ok(Self::Real),
            25 => Ok(Self::Imaginary),
            26 => Ok(Self::Complex),
            128 => Ok(Self::Transmission),
            129 => Ok(Self::Reflectance),
            130 => Ok(Self::ArbitraryOrSingleBeamWithValleyPeaks),
            131 => Ok(Self::Emission),
            v => Err(yTypeCreationError(v)),
        }
    }
}
