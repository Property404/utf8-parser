use core::fmt;

/// Error type used for the `utf8-parser` crate
#[derive(Copy, Clone, PartialEq, Eq, Debug)]
pub enum Utf8ParserError {
    /// Encountered an invalid byte. This is a byte that's invalid no matter the context.
    InvalidByte(u8),
    /// Found a character that's not valid UTF-8.
    InvalidChar(u32),
    /// Found a start byte in an unexpected place
    UnexpectedStartByte(u8),
    /// Found a continuation byte in an unexpected place
    UnexpectedContinuationByte(u8),
}

impl fmt::Display for Utf8ParserError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::InvalidByte(byte) => {
                write!(f, "Found invalid byte: 0x{byte:02x}")
            }
            Self::InvalidChar(word) => {
                write!(f, "Parsed invalid UTF-8 code point: 0x{word:04x}")
            }
            Self::UnexpectedStartByte(byte) => {
                write!(
                    f,
                    "Found start byte when a continuation byte was expected: 0x{byte:02x}"
                )
            }
            Self::UnexpectedContinuationByte(byte) => {
                write!(
                    f,
                    "Found continuation byte when a start byte was expected: 0x{byte:02x}"
                )
            }
        }
    }
}

#[cfg(feature = "std")]
impl std::error::Error for Utf8ParserError {}
