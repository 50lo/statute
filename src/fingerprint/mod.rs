//! Link fingerprints: bytes in, hashes out. Each mode fixes its hash input; golden tests pin it.

mod go;
mod python;
mod syntax;

use crate::model::Mode;
use sha2::{Digest, Sha256};
use std::collections::BTreeMap;
use std::fmt;

/// Why a target has no fingerprint. Each makes the link missing.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Missing {
    /// The file does not exist, or is not a regular file.
    NoFile,
    /// The file has no such symbol, or the mode has no symbols.
    NoSymbol,
    /// More than one symbol matches.
    Ambiguous,
    /// The file does not parse cleanly. A partial parse never counts as unchanged.
    ParseError,
}

impl fmt::Display for Missing {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(match self {
            Missing::NoFile => "the file does not exist",
            Missing::NoSymbol => "the symbol is not found",
            Missing::Ambiguous => "more than one symbol matches",
            Missing::ParseError => "the file does not parse cleanly",
        })
    }
}

/// The current fingerprint of each target, in order. `files` holds the bytes of every target file,
/// `None` for a file that does not exist. Each file is parsed once per mode, for all its symbols.
pub fn sign(
    files: &BTreeMap<String, Option<Vec<u8>>>,
    targets: &[(Mode, &str)],
) -> Vec<Result<String, Missing>> {
    let mut results = vec![Err(Missing::NoFile); targets.len()];
    let mut symbols: BTreeMap<(&str, Mode), Vec<(usize, &str)>> = BTreeMap::new();
    for (i, (mode, target)) in targets.iter().enumerate() {
        let (file, symbol) = match target.split_once('#') {
            Some((file, symbol)) => (file, Some(symbol)),
            None => (*target, None),
        };
        let Some(bytes) = files.get(file).and_then(Option::as_ref) else {
            continue;
        };
        results[i] = match (mode, symbol) {
            (Mode::BytesSha256V1, None) => Ok(sha256(bytes)),
            (_, None) => Err(Missing::NoSymbol),
            (_, Some(symbol)) => {
                symbols.entry((file, *mode)).or_default().push((i, symbol));
                continue;
            }
        };
    }
    for ((file, mode), wanted) in symbols {
        let bytes = files[file].as_deref().unwrap_or_default();
        let names: Vec<&str> = wanted.iter().map(|(_, s)| *s).collect();
        let hashes = match mode {
            Mode::BytesSha256V1 => vec![Err(Missing::NoSymbol); names.len()],
            Mode::GoAstSha256V1 => go::sign(bytes, &names),
            Mode::PythonAstSha256V1 => python::sign(bytes, &names),
        };
        for ((i, _), hash) in wanted.iter().zip(hashes) {
            results[*i] = hash;
        }
    }
    results
}

/// The mode `link` uses for a new target: by file extension when a symbol is given, otherwise bytes.
/// `None` when no mode reads symbols in this kind of file.
pub fn mode_for(file: &str, symbol: Option<&str>) -> Option<Mode> {
    if symbol.is_none() {
        return Some(Mode::BytesSha256V1);
    }
    match file.rsplit_once('.').map(|(_, ext)| ext) {
        Some("go") => Some(Mode::GoAstSha256V1),
        Some("py") => Some(Mode::PythonAstSha256V1),
        _ => None,
    }
}

fn sha256(bytes: &[u8]) -> String {
    Sha256::digest(bytes)
        .iter()
        .map(|b| format!("{b:02x}"))
        .collect()
}

#[cfg(test)]
mod tests {
    use super::*;

    fn one(file: Option<&[u8]>, target: &str) -> Result<String, Missing> {
        let files = BTreeMap::from([("a.go".to_string(), file.map(<[u8]>::to_vec))]);
        sign(&files, &[(Mode::BytesSha256V1, target)]).remove(0)
    }

    /// Golden: the hash input of `bytes-sha256-v1` is the file's bytes, unchanged.
    #[test]
    fn bytes_mode_hashes_the_raw_file() {
        assert_eq!(
            one(Some(b"package billing\n\nfunc Approve() {}\n"), "a.go").unwrap(),
            "89d22855a94f23b681b63f6d1486d642105dd9a7fd517327ec9621adcdb637da"
        );
        assert_eq!(
            one(Some(b""), "a.go").unwrap(),
            "e3b0c44298fc1c149afbf4c8996fb92427ae41e4649b934ca495991b7852b855"
        );
    }

    #[test]
    fn missing_files_and_symbols() {
        assert_eq!(one(None, "a.go"), Err(Missing::NoFile));
        assert_eq!(one(Some(b"x"), "b.go"), Err(Missing::NoFile));
        assert_eq!(one(Some(b"x"), "a.go#Approve"), Err(Missing::NoSymbol));
    }
}
