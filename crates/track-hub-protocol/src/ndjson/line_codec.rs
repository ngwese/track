//! Read and write one NDJSON line (ADR 0004 §Wire format).

use std::io::{self, Write};

use serde::Serialize;
use serde::de::DeserializeOwned;
use thiserror::Error;

/// Errors parsing or encoding NDJSON lines.
#[derive(Debug, Error, PartialEq)]
pub enum LineCodecError {
    /// Input ended before a complete line was available.
    #[error("incomplete line")]
    IncompleteLine,
    /// Trailing bytes remain after one JSON value.
    #[error("trailing garbage after JSON value")]
    TrailingGarbage,
    /// Line was empty or whitespace only.
    #[error("empty line")]
    EmptyLine,
    /// JSON deserialization failed.
    #[error("invalid JSON: {0}")]
    InvalidJson(String),
}

fn trim_line(bytes: &[u8]) -> Result<&[u8], LineCodecError> {
    let line = bytes
        .split(|&b| b == b'\n')
        .next()
        .ok_or(LineCodecError::IncompleteLine)?;
    let mut end = line.len();
    while end > 0 && (line[end - 1] == b'\r' || line[end - 1] == b' ') {
        end -= 1;
    }
    let mut start = 0;
    while start < end && line[start] == b' ' {
        start += 1;
    }
    let trimmed = &line[start..end];
    if trimmed.is_empty() {
        return Err(LineCodecError::EmptyLine);
    }
    Ok(trimmed)
}

/// Reads one complete JSON value from `bytes`, rejecting trailing garbage.
pub fn read_line<T: DeserializeOwned>(bytes: &[u8]) -> Result<T, LineCodecError> {
    let trimmed = trim_line(bytes)?;
    let mut de = serde_json::Deserializer::from_slice(trimmed);
    let value =
        T::deserialize(&mut de).map_err(|err| LineCodecError::InvalidJson(err.to_string()))?;
    if de.end().is_err() {
        return Err(LineCodecError::TrailingGarbage);
    }
    Ok(value)
}

/// Parses one complete JSON object from a string line.
pub fn parse_line<T: DeserializeOwned>(line: &str) -> Result<T, LineCodecError> {
    read_line(line.as_bytes())
}

/// Writes `value` as one NDJSON line including trailing newline.
pub fn write_line<T: Serialize>(writer: impl Write, value: &T) -> io::Result<()> {
    let mut writer = writer;
    serde_json::to_writer(&mut writer, value)?;
    writer.write_all(b"\n")?;
    Ok(())
}

/// Serializes `value` to one NDJSON line string (no trailing newline).
pub fn write_line_string<T: Serialize>(value: &T) -> Result<String, LineCodecError> {
    serde_json::to_string(value).map_err(|err| LineCodecError::InvalidJson(err.to_string()))
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde::{Deserialize, Serialize};

    #[derive(Debug, PartialEq, Serialize, Deserialize)]
    struct Sample {
        n: u32,
    }

    #[test]
    fn rejects_trailing_garbage() {
        let err = read_line::<Sample>(b"{\"n\":1}{\"n\":2}\n").unwrap_err();
        assert_eq!(err, LineCodecError::TrailingGarbage);
    }

    #[test]
    fn rejects_partial_line() {
        let err = read_line::<Sample>(b"").unwrap_err();
        assert_eq!(err, LineCodecError::EmptyLine);
    }

    #[test]
    fn write_and_read_round_trip() {
        let sample = Sample { n: 7 };
        let mut buf = Vec::new();
        write_line(&mut buf, &sample).unwrap();
        let parsed: Sample = read_line(&buf).unwrap();
        assert_eq!(parsed, sample);
    }

    #[test]
    fn parse_line_rejects_garbage() {
        let err = parse_line::<Sample>("{\"n\":1} extra").unwrap_err();
        assert_eq!(err, LineCodecError::TrailingGarbage);
    }
}
