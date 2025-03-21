#[derive(Debug, thiserror::Error, miette::Diagnostic)]
pub(super) enum MonthParseError {
    #[error("Invalid month: {0}")]
    InvalidMonth(u8),
}

#[derive(Copy, Clone, Debug)]
pub(super) enum Month {
    January = 1,
    February = 2,
    March = 3,
    April = 4,
    May = 5,
    June = 6,
    July = 7,
    August = 8,
    September = 9,
    October = 10,
    November = 11,
    December = 12,
}

impl TryFrom<u8> for Month {
    type Error = MonthParseError;
    fn try_from(value: u8) -> Result<Self, Self::Error> {
        match value {
            // Although the Spec says 1=January, data often contains datetime information, but with
            // a 0 in the month field. This is undefined as per the spec, but we just assume it is
            // January...
            0 => Ok(Month::January),
            1 => Ok(Month::January),
            2 => Ok(Month::February),
            3 => Ok(Month::March),
            4 => Ok(Month::April),
            5 => Ok(Month::May),
            6 => Ok(Month::June),
            7 => Ok(Month::July),
            8 => Ok(Month::August),
            9 => Ok(Month::September),
            10 => Ok(Month::October),
            11 => Ok(Month::November),
            12 => Ok(Month::December),
            n => Err(MonthParseError::InvalidMonth(n)),
        }
    }
}
