// SPDX-License-Identifier: MPL-2.0

use std::fmt;
use std::fs;
use std::path::Path;

use serde_json::Value;

#[derive(Debug, Clone)]
pub struct FixtureError(pub String);

impl fmt::Display for FixtureError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(&self.0)
    }
}
impl std::error::Error for FixtureError {}

pub fn read_json(path: &Path) -> Result<Value, FixtureError> {
    let bytes =
        fs::read(path).map_err(|e| FixtureError(format!("read {}: {e}", path.display())))?;
    serde_json::from_slice(&bytes)
        .map_err(|e| FixtureError(format!("parse {}: {e}", path.display())))
}

pub fn req<'a>(root: &'a Value, pointer: &str) -> Result<&'a Value, FixtureError> {
    root.pointer(pointer)
        .ok_or_else(|| FixtureError(format!("missing fixture field {pointer}")))
}

pub fn req_str<'a>(root: &'a Value, pointer: &str) -> Result<&'a str, FixtureError> {
    req(root, pointer)?
        .as_str()
        .ok_or_else(|| FixtureError(format!("fixture field {pointer} must be a string")))
}

pub fn req_u64(root: &Value, pointer: &str) -> Result<u64, FixtureError> {
    req(root, pointer)?.as_u64().ok_or_else(|| {
        FixtureError(format!(
            "fixture field {pointer} must be an unsigned integer"
        ))
    })
}

pub fn req_array<'a>(root: &'a Value, pointer: &str) -> Result<&'a Vec<Value>, FixtureError> {
    req(root, pointer)?
        .as_array()
        .ok_or_else(|| FixtureError(format!("fixture field {pointer} must be an array")))
}

pub fn hex_at(root: &Value, pointer: &str) -> Result<Vec<u8>, FixtureError> {
    let raw = req_str(root, pointer)?;
    hex::decode(raw)
        .map_err(|e| FixtureError(format!("fixture field {pointer} is invalid hex: {e}")))
}

pub fn version(root: &Value) -> Result<&str, FixtureError> {
    req_str(root, "/version")
}
