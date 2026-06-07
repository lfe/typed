use std::fmt;
use std::path::PathBuf;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Position {
    pub offset: usize,
    pub line: usize,
    pub column: usize,
}

impl Position {
    pub fn new(offset: usize, line: usize, column: usize) -> Self {
        Self { offset, line, column }
    }
}

impl fmt::Display for Position {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}:{}", self.line, self.column)
    }
}

#[derive(Debug, thiserror::Error)]
pub enum LexError {
    #[error("unexpected character '{ch}' at {pos}")]
    UnexpectedChar { ch: char, pos: Position },

    #[error("unterminated string at {pos}")]
    UnterminatedString { pos: Position },

    #[error("invalid escape sequence '\\{ch}' at {pos}")]
    InvalidEscape { ch: char, pos: Position },

    #[error("unexpected end of input")]
    UnexpectedEof,
}

#[derive(Debug, thiserror::Error)]
pub enum ParseError {
    #[error("unexpected token {token:?} at {pos}")]
    UnexpectedToken { token: String, pos: Position },

    #[error("expected {expected}, found {found} at {pos}")]
    Expected { expected: String, found: String, pos: Position },

    #[error("unterminated list at {pos}")]
    UnterminatedList { pos: Position },

    #[error("unexpected closing parenthesis at {pos}")]
    UnexpectedCloseParen { pos: Position },

    #[error("empty input")]
    EmptyInput,

    #[error("failed to read file {path:?}: {source}")]
    FileReadError {
        path: PathBuf,
        #[source]
        source: std::io::Error,
    },

    #[error("{0}")]
    LexError(#[from] LexError),
}

#[derive(Debug, thiserror::Error)]
pub enum CheckError {
    #[error("{file}:{pos}: {message}")]
    Diagnostic {
        file: String,
        pos: Position,
        message: String,
    },
}

pub type Result<T> = std::result::Result<T, ParseError>;
