// SPDX-License-Identifier: MPL-2.0

use std::cmp::Ordering;
use std::fmt;

const MAX_DEPTH: usize = 64;

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Error {
    Truncated,
    NonDeterministic,
    DuplicateMapKey,
    InvalidUtf8,
    UnsupportedType,
    NestingTooDeep,
    TrailingData,
    LengthOverflow,
}

impl fmt::Display for Error {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let msg = match self {
            Self::Truncated => "truncated CBOR",
            Self::NonDeterministic => "non-deterministic CBOR",
            Self::DuplicateMapKey => "duplicate CBOR map key",
            Self::InvalidUtf8 => "invalid CBOR UTF-8 text",
            Self::UnsupportedType => "unsupported CBOR type in v0.2 core",
            Self::NestingTooDeep => "CBOR nesting too deep",
            Self::TrailingData => "trailing CBOR data",
            Self::LengthOverflow => "CBOR length overflow",
        };
        f.write_str(msg)
    }
}

impl std::error::Error for Error {}

pub fn validate_cde(input: &[u8]) -> Result<(), Error> {
    let end = scan_item(input, 0, 0)?;
    if end != input.len() {
        return Err(Error::TrailingData);
    }
    Ok(())
}

fn scan_item(input: &[u8], start: usize, depth: usize) -> Result<usize, Error> {
    if depth > MAX_DEPTH {
        return Err(Error::NestingTooDeep);
    }
    let first = *input.get(start).ok_or(Error::Truncated)?;
    let major = first >> 5;
    let ai = first & 0x1f;
    let mut pos = start + 1;

    match major {
        0 | 1 => {
            let (_, next) = read_argument(input, pos, ai)?;
            Ok(next)
        }
        2 | 3 => {
            let (len, next) = read_argument(input, pos, ai)?;
            pos = next;
            let len = usize::try_from(len).map_err(|_| Error::LengthOverflow)?;
            let end = pos.checked_add(len).ok_or(Error::LengthOverflow)?;
            let bytes = input.get(pos..end).ok_or(Error::Truncated)?;
            if major == 3 && std::str::from_utf8(bytes).is_err() {
                return Err(Error::InvalidUtf8);
            }
            Ok(end)
        }
        4 => {
            let (count, next) = read_argument(input, pos, ai)?;
            pos = next;
            for _ in 0..count {
                pos = scan_item(input, pos, depth + 1)?;
            }
            Ok(pos)
        }
        5 => {
            let (count, next) = read_argument(input, pos, ai)?;
            pos = next;
            let mut previous_key: Option<&[u8]> = None;
            for _ in 0..count {
                let key_start = pos;
                pos = scan_item(input, pos, depth + 1)?;
                let key = &input[key_start..pos];
                if let Some(prev) = previous_key {
                    match prev.cmp(key) {
                        Ordering::Equal => return Err(Error::DuplicateMapKey),
                        Ordering::Greater => return Err(Error::NonDeterministic),
                        Ordering::Less => {}
                    }
                }
                previous_key = Some(key);
                pos = scan_item(input, pos, depth + 1)?;
            }
            Ok(pos)
        }
        6 => {
            let (_, next) = read_argument(input, pos, ai)?;
            scan_item(input, next, depth + 1)
        }
        7 => scan_simple(input, pos, ai),
        _ => unreachable!(),
    }
}

fn scan_simple(input: &[u8], pos: usize, ai: u8) -> Result<usize, Error> {
    match ai {
        0..=23 => Ok(pos),
        24 => {
            let value = *input.get(pos).ok_or(Error::Truncated)?;
            if value < 32 {
                return Err(Error::NonDeterministic);
            }
            Ok(pos + 1)
        }
        25..=27 => Err(Error::UnsupportedType), // floats are outside v0.2 security objects
        28..=31 => Err(Error::NonDeterministic),
        _ => unreachable!(),
    }
}

fn read_argument(input: &[u8], pos: usize, ai: u8) -> Result<(u64, usize), Error> {
    match ai {
        0..=23 => Ok((u64::from(ai), pos)),
        24 => {
            let v = u64::from(*input.get(pos).ok_or(Error::Truncated)?);
            if v < 24 {
                return Err(Error::NonDeterministic);
            }
            Ok((v, pos + 1))
        }
        25 => {
            let bytes: [u8; 2] = input
                .get(pos..pos + 2)
                .ok_or(Error::Truncated)?
                .try_into()
                .expect("slice length checked");
            let v = u64::from(u16::from_be_bytes(bytes));
            if v <= u64::from(u8::MAX) {
                return Err(Error::NonDeterministic);
            }
            Ok((v, pos + 2))
        }
        26 => {
            let bytes: [u8; 4] = input
                .get(pos..pos + 4)
                .ok_or(Error::Truncated)?
                .try_into()
                .expect("slice length checked");
            let v = u64::from(u32::from_be_bytes(bytes));
            if v <= u64::from(u16::MAX) {
                return Err(Error::NonDeterministic);
            }
            Ok((v, pos + 4))
        }
        27 => {
            let bytes: [u8; 8] = input
                .get(pos..pos + 8)
                .ok_or(Error::Truncated)?
                .try_into()
                .expect("slice length checked");
            let v = u64::from_be_bytes(bytes);
            if v <= u64::from(u32::MAX) {
                return Err(Error::NonDeterministic);
            }
            Ok((v, pos + 8))
        }
        28..=31 => Err(Error::NonDeterministic),
        _ => unreachable!(),
    }
}
