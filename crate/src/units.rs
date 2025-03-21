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

impl InstrumentTechnique {
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
