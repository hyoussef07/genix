//! Generation helpers for Genix.
//!
//! This module exposes a single public function, `generate_many`, which
//! supports several generation styles: `random`, `pin`, `hex`, `base64`, and
//! `passphrase`. For `passphrase` a wordlist may be provided; otherwise a small
//! built-in list is used for examples and tests.
//!
//! The generator keeps a clear separation between entropy calculation and byte
//! / character generation so other modules can test and reuse the logic.

use std::fs::File;
use std::io::{BufRead, BufReader};

use base64::{Engine as _, engine::general_purpose};
use rand::distributions::Uniform;
use rand::{Rng, thread_rng};

use crate::entropy::charset_size_for_style;

const DEFAULT_PRINTABLE: &str =
    "ABCDEFGHIJKLMNOPQRSTUVWXYZabcdefghijklmnopqrstuvwxyz0123456789!@#$%&*()-_=+[]{};:,.<>?/`~";
const AMBIGUOUS: &str = "1lI0O|";

/// Generate `count` items using `style` with optional `wordlist`.
///
/// Parameters
/// - `style`: generation style (`random`, `pin`, `hex`, `base64`, `passphrase`).
/// - `length`: length meaning depends on style (characters for `random`/`pin`,
///   bytes for `hex`/`base64`, word count for `passphrase`).
/// - `count`: how many items to produce.
/// - `wordlist`: optional path to a newline-delimited wordlist file (for
///   `passphrase`). If `None` a small builtin list is used.
/// - `no_ambiguous`: if true, removes ambiguous characters for `random` style.
/// - `min_entropy`: optional minimum entropy target (bits). If provided and the
///   style supports a charset hint, the function may increase `length` to
///   satisfy the requested entropy.
///
/// Returns
/// - `Ok(Vec<String>)` on success with `count` generated items.
/// - `Err(String)` on fatal errors (for example, unknown style or missing
///   wordlist file).
pub fn generate_many(
    style: &str,
    mut length: usize,
    count: usize,
    wordlist: Option<&str>,
    no_ambiguous: bool,
    min_entropy: Option<f64>,
) -> Result<Vec<String>, String> {
    if let Some(bits) = min_entropy
        && let Some(charset_size) = charset_size_for_style(style, no_ambiguous)
    {
        let per_char = (charset_size as f64).log2();
        if per_char <= 0.0 {
            return Err("invalid charset size for entropy calculation".into());
        }
        let needed = (bits / per_char).ceil() as usize;
        if needed > length {
            eprintln!(
                "info: increasing length from {} to {} to satisfy min-entropy {} bits",
                length, needed, bits
            );
            length = needed;
        }
    }

    match style {
        "random" => Ok((0..count)
            .map(|_| random_string(length, no_ambiguous))
            .collect()),
        "pin" => Ok((0..count).map(|_| pin_string(length)).collect()),
        "hex" => Ok((0..count).map(|_| hex_string(length)).collect()),
        "base64" => Ok((0..count).map(|_| base64_string(length)).collect()),
        "passphrase" => {
            let words = load_wordlist(wordlist)?;
            if words.is_empty() {
                return Err("wordlist is empty".into());
            }
            Ok((0..count)
                .map(|_| passphrase_from(&words, length))
                .collect())
        }
        _ => Err(format!("unknown style: {}", style)),
    }
}

/// Generate a random string using the default printable set.
///
/// This helper is intentionally small and deterministic in its contract: it
/// returns a string of length `len`, optionally filtering ambiguous chars.
fn random_string(len: usize, no_ambiguous: bool) -> String {
    let mut rng = thread_rng();
    let mut pool: Vec<char> = DEFAULT_PRINTABLE.chars().collect();
    if no_ambiguous {
        pool.retain(|c| !AMBIGUOUS.contains(*c));
    }
    if pool.is_empty() {
        return String::new();
    }
    let dist = Uniform::from(0..pool.len());
    (0..len).map(|_| pool[rng.sample(dist)]).collect()
}

/// Generate a numeric PIN of length `len`.
fn pin_string(len: usize) -> String {
    let mut rng = thread_rng();
    let dist = Uniform::from(0..10);
    (0..len)
        .map(|_| char::from(b'0' + rng.sample(dist) as u8))
        .collect()
}

/// Generate a hex string representing `bytes` random bytes.
fn hex_string(bytes: usize) -> String {
    let mut rng = thread_rng();
    let mut buf = vec![0u8; bytes];
    rng.fill(&mut buf[..]);
    buf.iter().map(|b| format!("{:02x}", b)).collect()
}

/// Generate a base64 encoding of `bytes` random bytes.
fn base64_string(bytes: usize) -> String {
    let mut rng = thread_rng();
    let mut buf = vec![0u8; bytes];
    rng.fill(&mut buf[..]);
    general_purpose::STANDARD.encode(&buf)
}

/// Load a newline-delimited wordlist from `path` or return a built-in list.
fn load_wordlist(path: Option<&str>) -> Result<Vec<String>, String> {
    if let Some(p) = path {
        let file = File::open(p).map_err(|e| format!("failed to open wordlist {}: {}", p, e))?;
        let reader = BufReader::new(file);
        Ok(reader
            .lines()
            .map_while(Result::ok)
            .map(|s| s.trim().to_string())
            .filter(|s| !s.is_empty())
            .collect())
    } else {
        Ok(vec![
            "alpha".into(),
            "bravo".into(),
            "charlie".into(),
            "delta".into(),
            "echo".into(),
            "foxtrot".into(),
            "golf".into(),
            "hotel".into(),
            "india".into(),
            "juliet".into(),
        ])
    }
}

/// Build a dash-separated passphrase from `target_words` randomly sampled words.
fn passphrase_from(words: &[String], target_words: usize) -> String {
    let mut rng = thread_rng();
    let dist = Uniform::from(0..words.len());
    (0..target_words)
        .map(|_| words[rng.sample(dist)].clone())
        .collect::<Vec<_>>()
        .join("-")
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_random_length() {
        let out = random_string(16, false);
        assert_eq!(out.len(), 16);
    }

    #[test]
    fn test_hex_length() {
        let s = hex_string(4);
        assert_eq!(s.len(), 8);
    }

    #[test]
    fn test_base64() {
        let s = base64_string(3);
        assert!(s.len() >= 4);
    }

    #[test]
    fn test_passphrase_default() {
        let words = load_wordlist(None).unwrap();
        let p = passphrase_from(&words, 4);
        assert!(p.split('-').count() == 4);
    }

    #[test]
    fn test_min_entropy_increases_length() {
        let res = generate_many("pin", 6, 1, None, false, Some(40.0)).unwrap();
        assert_eq!(res.len(), 1);
        assert!(res[0].len() >= 13);
    }
}
