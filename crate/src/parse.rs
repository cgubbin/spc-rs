use crate::{
    block::Block,
    header::{Header, HeaderParseError, SubheaderParseError},
    logblock::{LogBlock, LogHeaderParseError},
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

#[derive(Clone, Debug)]
pub struct ParsedSPC {
    pub(crate) header: Header,
    pub(crate) block: Block,
    pub(crate) log: Option<LogBlock>,
}
