use crate::{
    header::{HeaderParseError, SubheaderParseError},
    log::LogHeaderParseError,
};

#[derive(Debug, thiserror::Error, miette::Diagnostic)]
pub(crate) enum ParseError {
    #[error("failed to parse log block: {0:?}")]
    Log(#[from] LogHeaderParseError),
    #[error("failed to parse subheader: {0:?}")]
    Subheader(#[from] SubheaderParseError),
    #[error("failed to parse header: {0:?}")]
    Header(#[from] HeaderParseError),
}

pub(crate) trait Parse {
    type Parsed;
    fn parse(&self) -> Self::Parsed;
}

pub(crate) trait TryParse {
    type Parsed;
    type Error;
    fn try_parse(&self) -> Result<Self::Parsed, Self::Error>;
}
