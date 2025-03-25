use std::marker::PhantomData;

use zerocopy::{
    byteorder::{F32, I16, I32, U32},
    BigEndian, ByteOrder, Immutable, KnownLayout, LittleEndian, TryFromBytes,
};

use crate::{
    header::{LexedSubheader, Precision, Subheader, SubheaderParseError},
    parse::{Parse, TryParse},
};

#[derive(Clone, Debug, KnownLayout, Immutable, TryFromBytes)]
pub(crate) struct LexedDirectory<E: ByteOrder> {
    ssfposn: U32<E>,
    ssfsize: U32<E>,
    ssftime: F32<E>,
}

#[derive(Clone, Debug)]
pub(crate) struct Directory {
    ssfposn: u32,
    ssfsize: u32,
    ssftime: f32,
}

impl<E: ByteOrder> Parse for LexedDirectory<E> {
    type Parsed = Directory;
    fn parse(&self) -> Self::Parsed {
        Directory {
            ssfposn: self.ssfposn.get(),
            ssfsize: self.ssfsize.get(),
            ssftime: self.ssftime.get(),
        }
    }
}

#[derive(Clone, Debug)]
pub(crate) struct LexedXData<'data, E: ByteOrder> {
    data: &'data [u8],
    byte_order: PhantomData<E>,
}

impl<'data, E: ByteOrder> LexedXData<'data, E> {
    pub(super) fn new(data: &'data [u8]) -> miette::Result<Self> {
        if data.len() % 4 != 0 {
            miette::bail!("x-data is a list of 32-bit floats, so the underlying data must contain a multiple of 4 bytes");
        }
        Ok(Self {
            data,
            byte_order: PhantomData,
        })
    }
}

#[derive(Clone, Debug)]
pub(crate) struct XData(Vec<f32>);

impl<E: ByteOrder> Parse for LexedXData<'_, E> {
    type Parsed = XData;
    fn parse(&self) -> Self::Parsed {
        let data = self
            .data
            .chunks_exact(4)
            .map(|each| F32::<E>::from_bytes(each.try_into().unwrap()))
            .map(|each| each.get())
            .collect();

        XData(data)
    }
}

#[derive(Clone, Debug)]
pub(crate) enum YMode {
    SixteenBitInt,
    ThirtyTwoBitInt,
    IEEEFloat,
}

impl YMode {
    pub(crate) fn bytes_per_point(&self) -> usize {
        match self {
            Self::SixteenBitInt => 2,
            Self::ThirtyTwoBitInt => 4,
            Self::IEEEFloat => 4,
        }
    }
}

#[derive(Clone, Debug)]
pub(crate) struct LexedSubfile<'data, E: ByteOrder> {
    pub(super) subheader: &'data LexedSubheader<E>,
    pub(super) data: &'data [u8],
    pub(super) mode: YMode,
}

impl<'data, E: ByteOrder> LexedSubfile<'data, E> {
    pub(super) fn new(
        subheader: &'data LexedSubheader<E>,
        data: &'data [u8],
        mode: YMode,
    ) -> miette::Result<Self> {
        let bytes_per_point = mode.bytes_per_point();

        // TODO: We can't check the number of points here if it's provided in the header rather
        // than the subheader, it's opaque atm how often this happens.
        if (subheader.number_of_points() != 0)
            & (data.len() / bytes_per_point != subheader.number_of_points())
        {
            miette::bail!("y-data is a list of 32-bit floats, or 16-bit integers so the underlying data must contain a multiple of 2 or 4 bytes");
        }

        Ok(Self {
            subheader,
            data,
            mode,
        })
    }
}

#[derive(Clone, Debug)]
pub(crate) enum YData {
    SixteenBitInteger(Vec<i16>),
    ThirtyTwoBitInteger(Vec<i32>),
    Float(Vec<f64>),
}

impl YData {
    fn len(&self) -> usize {
        match self {
            Self::SixteenBitInteger(vals) => vals.len(),
            Self::ThirtyTwoBitInteger(vals) => vals.len(),
            Self::Float(vals) => vals.len(),
        }
    }
}

#[derive(Clone, Debug)]
pub(crate) struct Subfile {
    pub(super) subheader: Subheader,
    pub(super) data: YData,
}

impl<E: ByteOrder> TryParse for LexedSubfile<'_, E> {
    type Parsed = Subfile;
    type Error = SubheaderParseError;
    fn try_parse(&self) -> Result<Self::Parsed, Self::Error> {
        let data = match self.mode {
            YMode::SixteenBitInt => YData::SixteenBitInteger(
                self.data
                    .chunks_exact(2)
                    .map(|each| I16::<E>::from_bytes(each.try_into().unwrap()))
                    .map(|each| each.get())
                    .collect(),
            ),
            YMode::ThirtyTwoBitInt => YData::ThirtyTwoBitInteger(
                self.data
                    .chunks_exact(4)
                    .map(|each| {
                        let first = I16::<E>::from_bytes([each[0], each[1]]);
                        let second = I16::<E>::from_bytes([each[2], each[3]]);
                        ((first.get() as i32) << 16) + second.get() as i32
                    })
                    .collect(),
            ),
            YMode::IEEEFloat => YData::Float(
                self.data
                    .chunks_exact(4)
                    .map(|each| F32::<E>::from_bytes(each.try_into().unwrap()))
                    .map(|each| each.get() as f64)
                    .collect(),
            ),
        };

        Ok(Subfile {
            subheader: self.subheader.try_parse()?,
            data,
        })
    }
}

#[derive(Clone)]
pub(crate) enum LexedBlock<'data, E: ByteOrder> {
    Y(LexedSubfile<'data, E>),
    YY(Vec<LexedSubfile<'data, E>>),
    XY {
        x: LexedXData<'data, E>,
        y: LexedSubfile<'data, E>,
    },
    XYY {
        x: LexedXData<'data, E>,
        ys: Vec<LexedSubfile<'data, E>>,
    },
    XYXY {
        data: Vec<(LexedXData<'data, E>, LexedSubfile<'data, E>)>,
        directory: Option<Vec<&'data LexedDirectory<E>>>,
    },
}

#[derive(Clone)]
pub(crate) enum Block {
    Y(Subfile),
    YY(Vec<Subfile>),
    XY {
        x: XData,
        y: Subfile,
    },
    XYY {
        x: XData,
        ys: Vec<Subfile>,
    },
    XYXY {
        data: Vec<(XData, Subfile)>,
        directory: Option<Vec<Directory>>,
    },
}

impl<E: ByteOrder> TryParse for LexedBlock<'_, E> {
    type Parsed = Block;
    type Error = SubheaderParseError;
    fn try_parse(&self) -> Result<Self::Parsed, Self::Error> {
        Ok(match self {
            Self::Y(subfile) => Block::Y(subfile.try_parse()?),
            Self::YY(subfiles) => Block::YY(
                subfiles
                    .iter()
                    .map(TryParse::try_parse)
                    .collect::<Result<Vec<_>, _>>()?,
            ),
            Self::XY { x, y } => Block::XY {
                x: x.parse(),
                y: y.try_parse()?,
            },
            Self::XYY { x, ys } => Block::XYY {
                x: x.parse(),
                ys: ys
                    .iter()
                    .map(TryParse::try_parse)
                    .collect::<Result<Vec<_>, _>>()?,
            },
            Self::XYXY { data, directory } => Block::XYXY {
                data: data
                    .iter()
                    .map(|(x, y)| y.try_parse().map(|y| (x.parse(), y)))
                    .collect::<Result<Vec<_>, _>>()?,
                directory: directory
                    .clone()
                    .map(|directorys| directorys.into_iter().map(Parse::parse).collect()),
            },
        })
    }
}

impl<'data, E: ByteOrder> ::std::fmt::Debug for LexedBlock<'data, E> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            LexedBlock::Y(subfile) => {
                writeln!(
                    f,
                    "y-data type, with {} data points ",
                    subfile.data.len() / subfile.mode.bytes_per_point(),
                )?;
            }
            LexedBlock::YY(subfiles) => {
                writeln!(f, "yy-data type, {} subfiles", subfiles.len())?;
            }
            LexedBlock::XY { x, .. } => {
                writeln!(f, "xy-data type, {} data points", x.data.len())?;
            }
            LexedBlock::XYY { x, ys } => {
                writeln!(
                    f,
                    "xyy-data type, {} data points and {} subfiles",
                    x.data.len(),
                    ys.len()
                )?;
            }
            LexedBlock::XYXY { data, .. } => {
                writeln!(f, "xyxy-data type, with {} subfiles", data.len(),)?;
            }
        }
        Ok(())
    }
}

impl ::std::fmt::Debug for Block {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Block::Y(subfile) => {
                writeln!(f, "y-data type, with {} data points ", subfile.data.len())?;
            }
            Block::YY(subfiles) => {
                writeln!(f, "yy-data type, {} subfiles", subfiles.len())?;
            }
            Block::XY { x, .. } => {
                writeln!(f, "xy-data type, {} data points", x.0.len())?;
            }
            Block::XYY { x, ys } => {
                writeln!(
                    f,
                    "xyy-data type, {} data points and {} subfiles",
                    x.0.len(),
                    ys.len()
                )?;
            }
            Block::XYXY { data, .. } => {
                writeln!(f, "xyxy-data type, with {} subfiles", data.len(),)?;
            }
        }
        Ok(())
    }
}
