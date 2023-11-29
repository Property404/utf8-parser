//! A byte-by-byte UTF-8 parser
#![cfg_attr(not(any(test, feature = "std")), no_std)]
#![forbid(unsafe_code)]
use core::fmt;

/// Error type used for [Utf8Parser]
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

#[derive(Copy, Clone, PartialEq, Eq, Debug)]
enum Utf8ByteType {
    Continuation,
    Single,
    Double,
    Triple,
    Quadruple,
}

impl Utf8ByteType {
    const fn id(self) -> u8 {
        match self {
            Self::Single => 0b0,
            Self::Continuation => 0b10,
            Self::Double => 0b110,
            Self::Triple => 0b1110,
            Self::Quadruple => 0b11110,
        }
    }

    const fn id_length(self) -> u32 {
        self.id().count_ones() + 1
    }

    const fn value_mask(self) -> u8 {
        0xFF >> self.id_length()
    }

    const fn value_mask_length(self) -> u32 {
        self.value_mask().count_ones()
    }

    const fn matches(self, byte: u8) -> Option<u8> {
        if (byte >> self.value_mask_length()) == self.id() {
            Some(byte & self.value_mask())
        } else {
            None
        }
    }
}

/// A single byte from a UTF-8 stream
#[derive(Copy, Clone, Debug, PartialEq, Eq)]
pub enum ParsedByte {
    /// A one-byte UTF-8 character, i.e. an ASCII value
    Single(u8),
    /// A start byte that must be followed by one continuation character
    StartDouble(u8),
    /// A start byte that must be followed by two continuation characters
    StartTriple(u8),
    /// A start byte that must be followed by three continuation characters
    StartQuadruple(u8),
    /// A continuation character
    ContinuationByte(u8),
}

impl ParsedByte {
    /// Construct from a byte
    pub const fn from_byte(byte: u8) -> Result<Self, Utf8ParserError> {
        if let Some(value) = Utf8ByteType::Single.matches(byte) {
            Ok(Self::Single(value))
        } else if let Some(value) = Utf8ByteType::Double.matches(byte) {
            Ok(Self::StartDouble(value))
        } else if let Some(value) = Utf8ByteType::Triple.matches(byte) {
            Ok(Self::StartTriple(value))
        } else if let Some(value) = Utf8ByteType::Quadruple.matches(byte) {
            Ok(Self::StartQuadruple(value))
        } else if let Some(value) = Utf8ByteType::Continuation.matches(byte) {
            Ok(Self::ContinuationByte(value))
        } else {
            Err(Utf8ParserError::InvalidByte(byte))
        }
    }

    /// Returns true if this is a continuation byte
    pub const fn is_continuation(self) -> bool {
        matches!(self, ParsedByte::ContinuationByte(_))
    }
}

impl TryFrom<u8> for ParsedByte {
    type Error = Utf8ParserError;
    fn try_from(byte: u8) -> Result<Self, Self::Error> {
        Self::from_byte(byte)
    }
}

#[derive(Copy, Clone, Debug)]
enum State {
    Fresh,
    OneLeft(u32),
    TwoLeft(u32),
    ThreeLeft(u32),
}

const fn push_byte(current: u32, byte: u8) -> u32 {
    debug_assert!(current <= 0x00FFFFFF);
    debug_assert!(byte <= 0b0011_1111);
    (current << Utf8ByteType::Continuation.value_mask_length()) | (byte as u32)
}

/// A byte-by-byte UTF-8 parser.
#[derive(Clone, Debug)]
pub struct Utf8Parser {
    state: State,
}

impl Utf8Parser {
    /// Construct a new Utf8Parser
    pub const fn new() -> Self {
        Self {
            state: State::Fresh,
        }
    }

    /// Push a byte into the parser
    pub fn push(&mut self, byte: u8) -> Result<Option<char>, Utf8ParserError> {
        match self.push_inner_impl(byte) {
            Ok(val) => Ok(val),
            // Reset on error
            Err(val) => {
                self.reset();
                Err(val)
            }
        }
    }

    // Inner functionality of `push`
    fn push_inner_impl(&mut self, byte: u8) -> Result<Option<char>, Utf8ParserError> {
        let byte = ParsedByte::try_from(byte)?;

        match (self.state, byte) {
            (State::OneLeft(current), ParsedByte::ContinuationByte(value)) => {
                self.state = State::Fresh;
                let c = push_byte(current, value);
                Ok(Some(
                    char::try_from(c).map_err(|_| Utf8ParserError::InvalidChar(c))?,
                ))
            }
            (State::TwoLeft(current), ParsedByte::ContinuationByte(value)) => {
                self.state = State::OneLeft(push_byte(current, value));
                Ok(None)
            }
            (State::ThreeLeft(current), ParsedByte::ContinuationByte(value)) => {
                self.state = State::TwoLeft(push_byte(current, value));
                Ok(None)
            }
            (State::Fresh, ParsedByte::Single(value)) => Ok(Some(value as char)),
            (State::Fresh, ParsedByte::StartDouble(value)) => {
                self.state = State::OneLeft(value as u32);
                Ok(None)
            }
            (State::Fresh, ParsedByte::StartTriple(value)) => {
                self.state = State::TwoLeft(value as u32);
                Ok(None)
            }
            (State::Fresh, ParsedByte::StartQuadruple(value)) => {
                self.state = State::ThreeLeft(value as u32);
                Ok(None)
            }
            (
                State::OneLeft(_) | State::TwoLeft(_) | State::ThreeLeft(_),
                ParsedByte::Single(value)
                | ParsedByte::StartDouble(value)
                | ParsedByte::StartTriple(value)
                | ParsedByte::StartQuadruple(value),
            ) => Err(Utf8ParserError::UnexpectedStartByte(value)),
            (State::Fresh, ParsedByte::ContinuationByte(value)) => {
                Err(Utf8ParserError::UnexpectedContinuationByte(value))
            }
        }
    }

    // Reset the state
    fn reset(&mut self) {
        self.state = State::Fresh;
    }
}

impl Default for Utf8Parser {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use rand::Rng;

    #[test]
    fn conversion() -> Result<(), Utf8ParserError> {
        let test_vectors = &[
            (0x00, ParsedByte::Single(0x00)),
            (0x01, ParsedByte::Single(0x01)),
            (0x65, ParsedByte::Single(0x65)),
            (0x7f, ParsedByte::Single(0x7f)),
            (0b110_00000, ParsedByte::StartDouble(0)),
            (0b110_00001, ParsedByte::StartDouble(0b1)),
            (0b110_11001, ParsedByte::StartDouble(0b11001)),
            (0b110_11111, ParsedByte::StartDouble(0b11111)),
            (0b1110_0000, ParsedByte::StartTriple(0)),
            (0b1110_0001, ParsedByte::StartTriple(0b1)),
            (0b1110_1001, ParsedByte::StartTriple(0b1001)),
            (0b1110_1111, ParsedByte::StartTriple(0b1111)),
            (0b1111_0000, ParsedByte::StartQuadruple(0)),
            (0b1111_0001, ParsedByte::StartQuadruple(0b1)),
            (0b1111_0111, ParsedByte::StartQuadruple(0b111)),
            (0x80, ParsedByte::ContinuationByte(0x00)),
            (0x81, ParsedByte::ContinuationByte(0x01)),
            (0b10_111111, ParsedByte::ContinuationByte(0b111111)),
        ];

        for tv in test_vectors.iter() {
            assert_eq!(ParsedByte::try_from(tv.0)?, tv.1);
        }

        Ok(())
    }

    #[test]
    fn basic() -> Result<(), Utf8ParserError> {
        let mut parser = Utf8Parser::default();
        assert_eq!(parser.push(b'h')?, Some('h'));
        assert_eq!(parser.push(b'e')?, Some('e'));
        assert_eq!(parser.push(b'l')?, Some('l'));
        assert_eq!(parser.push(b'l')?, Some('l'));
        assert_eq!(parser.push(b'o')?, Some('o'));
        assert_eq!(parser.push(0b1100_0000)?, None);
        Ok(())
    }

    fn parse_str_by_bytes(original: &str) -> Result<(), Utf8ParserError> {
        let original = original.as_bytes();
        let mut rebuilt = String::new();

        let mut parser = Utf8Parser::default();
        for byte in original {
            if let Some(c) = parser.push(*byte)? {
                rebuilt.push(c);
            }
        }

        assert_eq!(String::from_utf8(original.into()).unwrap(), rebuilt);

        Ok(())
    }

    #[test]
    fn parse_ascii_stream() -> Result<(), Utf8ParserError> {
        parse_str_by_bytes("The quick brown fox jamped over the lazy dog")
    }

    #[test]
    fn parse_emoji_stream() -> Result<(), Utf8ParserError> {
        parse_str_by_bytes("Thé quick brown 🦊 jamped over the lazy 🐕")
    }

    #[test]
    fn reset_state_after_error() {
        let mut parser = Utf8Parser::default();
        // Push a valid start byte
        assert!(parser.push(0b1110_0000).is_ok());
        // Push an invalid byte
        assert!(parser.push(0b1111_1110).is_err());
        assert_eq!(parser.push(b'a'), Ok(Some('a')));
    }

    #[test]
    fn random_input_dont_panic() {
        let mut parser = Utf8Parser::default();
        let mut rng = rand::thread_rng();
        for _ in 0..1_000_000 {
            let _ = parser.push(rng.gen());
        }
    }

    #[test]
    fn random_ascii_dont_error() {
        let mut parser = Utf8Parser::default();
        let mut rng = rand::thread_rng();
        for _ in 0..1_000_000 {
            let val: u8 = rng.gen();
            parser.push(val % 0x80).unwrap();
        }
    }
}
