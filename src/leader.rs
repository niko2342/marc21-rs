use nom::character::complete::satisfy;
use nom::error::{FromExternalError, ParseError};
use nom::multi::fold_many_m_n;
use nom::{Finish, IResult};

/// The leader contains information for the processing of the record.
#[derive(Debug, PartialEq, Eq)]
pub struct Leader {
    pub(crate) record_len: u32,
}

/// An error that can occur when parsing the leader field.
#[derive(Debug, thiserror::Error)]
pub enum ParseLeaderError {
    #[error("invalid record length")]
    InvalidRecordLength,

    #[error("incomplete leader, missing: {0:?}")]
    Incomplete(nom::Needed),

    #[error("parse error: {0:?}")]
    Nom(nom::error::ErrorKind),
}

impl<'a> ParseError<&'a [u8]> for ParseLeaderError {
    fn from_error_kind(
        _: &'a [u8],
        kind: nom::error::ErrorKind,
    ) -> Self {
        Self::Nom(kind)
    }

    fn append(
        _: &'a [u8],
        kind: nom::error::ErrorKind,
        _: Self,
    ) -> Self {
        Self::Nom(kind)
    }
}

impl From<ParseLeaderError> for nom::Err<ParseLeaderError> {
    fn from(e: ParseLeaderError) -> Self {
        nom::Err::Error(e)
    }
}

impl<I, E> FromExternalError<I, E> for ParseLeaderError {
    fn from_external_error(
        _: I,
        kind: nom::error::ErrorKind,
        _: E,
    ) -> Self {
        Self::Nom(kind)
    }
}

/// Holds the result of a parsing function.
pub(crate) type ParseResult<'a, O, E = ParseLeaderError> =
    IResult<&'a [u8], O, E>;

impl Leader {
    /// Creates a leader from a byte slice.
    ///
    /// # Example
    ///
    /// ```rust
    /// use marc21::Leader;
    ///
    /// # fn main() { example().unwrap(); }
    /// fn example() -> anyhow::Result<()> {
    ///     let leader = Leader::from_bytes(b"00827")?;
    ///     assert_eq!(leader.record_length(), 827);
    ///
    ///     Ok(())
    /// }
    /// ```
    pub fn from_bytes(data: &[u8]) -> Result<Self, ParseLeaderError> {
        parse_leader(data).finish().map(|(_, leader)| leader)
    }

    /// Returns the length of the entire record, including the leader
    /// and the record terminator.
    ///
    /// # Note
    ///
    /// The maximum length of a record is 99999 bytes/octets.
    ///
    /// # Example
    ///
    /// ```rust
    /// use marc21::Leader;
    ///
    /// # fn main() { example().unwrap(); }
    /// fn example() -> anyhow::Result<()> {
    ///     let leader = Leader::from_bytes(b"00827")?;
    ///     assert_eq!(leader.record_length(), 827);
    ///
    ///     Ok(())
    /// }
    /// ```
    pub fn record_length(&self) -> u32 {
        self.record_len
    }
}

/// Parse the record length field.
///
/// The record length is encoded as five right justified ASCII digits.
/// An unused positions is set to zero. The record length is between 0
/// and 99999.
#[inline]
fn parse_record_len(i: &[u8]) -> ParseResult<u32> {
    fold_many_m_n(
        5,
        5,
        satisfy(|ch| ch.is_ascii_digit()),
        || 0,
        |acc, n| acc * 10 + (n as u8 - b'0') as u32,
    )(i)
}

pub(crate) fn parse_leader(i: &[u8]) -> ParseResult<Leader> {
    let (i, record_len) = parse_record_len(i)
        .map_err(|_| ParseLeaderError::InvalidRecordLength)?;

    Ok((i, Leader { record_len }))
}

#[cfg(test)]
mod tests {
    use nom_test_helpers::prelude::*;

    use super::*;

    #[test]
    fn test_leader_from_bytes() -> anyhow::Result<()> {
        let leader = Leader::from_bytes(b"00123")?;
        assert_eq!(leader.record_length(), 123);

        assert!(Leader::from_bytes(b"1234").is_err());
        Ok(())
    }

    #[test]
    fn test_parse_record_len() {
        assert_finished_and_eq!(parse_record_len(b"99999"), 99999);
        assert_finished_and_eq!(parse_record_len(b"12345"), 12345);
        assert_finished_and_eq!(parse_record_len(b"00000"), 0);
        assert_error!(parse_record_len(b"-1000"));
        assert_error!(parse_record_len(b"1234"));
    }
}
