/// Entropy-related helpers (charset sizing and simple estimators).
use std::f64;

const DEFAULT_PRINTABLE: &str =
    "ABCDEFGHIJKLMNOPQRSTUVWXYZabcdefghijklmnopqrstuvwxyz0123456789!@#$%&*()-_=+[]{};:,.<>?/`~";

/// Return a conservative charset size hint for a named style.
pub fn charset_size_for_style(style: &str, no_ambiguous: bool) -> Option<usize> {
    match style {
        "random" => {
            let mut set = DEFAULT_PRINTABLE.chars().count();
            if no_ambiguous {
                set = set.saturating_sub(6); // remove 1,l,I,0,O,|
            }
            Some(set)
        }
        "pin" => Some(10),
        "hex" => Some(16),
        "base64" => Some(64),
        "passphrase" => None,
        _ => None,
    }
}

/// Estimate the entropy (in bits) of a provided string using a lightweight
/// heuristic.
///
/// This estimator is intentionally conservative and fast — it's designed to be
/// useful for CLI feedback and tests, not a replacement for full-strength
/// password analysis libraries. The rules used are:
///
/// - For `style == "passphrase"`, split on `-` and assume a default wordlist
///   size (2048) when computing bits per word: bits = words * log2(wordlist_size).
/// - Otherwise, detect character classes used in the string (lowercase,
///   uppercase, digits, symbols) and compute bits = length * log2(charset_size),
///   where charset_size is the sum of the detected classes.
/// - If detection yields an implausibly small charset, the function falls back
///   to the named style hint (via `charset_size_for_style`) when available.
///
/// # Errors
/// Returns `Err(String)` when a charset cannot be determined (for example, an
/// empty input and no relevant style hint).
pub fn estimate_entropy_for_str(s: &str, style: &str) -> Result<f64, String> {
    if style == "passphrase" {
        let words: Vec<&str> = s.split('-').filter(|w| !w.is_empty()).collect();
        let wordlist_size = 2048.0f64; // reasonable default for Diceware/EFF-style lists
        return Ok((words.len() as f64) * wordlist_size.log2());
    }

    // Auto-detect character classes
    let mut has_lower = false;
    let mut has_upper = false;
    let mut has_digit = false;
    let mut has_symbol = false;
    for ch in s.chars() {
        if ch.is_ascii_lowercase() {
            has_lower = true;
        } else if ch.is_ascii_uppercase() {
            has_upper = true;
        } else if ch.is_ascii_digit() {
            has_digit = true;
        } else {
            // treat everything else as a symbol (space, punctuation, unicode)
            has_symbol = true;
        }
    }

    let mut charset = 0usize;
    if has_lower {
        charset += 26;
    }
    if has_upper {
        charset += 26;
    }
    if has_digit {
        charset += 10;
    }
    if has_symbol {
        // approximate number of printable symbols commonly available
        charset += 32;
    }

    // If detection failed (e.g., empty string), try style hint
    if charset < 2 && let Some(hint) = charset_size_for_style(style, false) {
        charset = hint;
    }

    if charset < 2 {
        return Err("cannot determine charset size for entropy estimation".into());
    }

    let per_char = (charset as f64).log2();
    Ok(per_char * (s.chars().count() as f64))
}

/// Detailed entropy profile structure returned by `estimate_entropy_detailed`.
#[derive(Debug)]
pub struct EntropyProfile {
    /// Estimated total entropy in bits
    pub bits: f64,
    /// Charset size inferred or hinted
    pub charset_size: usize,
    /// Per-character entropy (bits)
    pub per_char: f64,
    /// Length in characters (or words for passphrase)
    pub length: usize,
    /// Flags indicating which character classes were present
    pub has_lower: bool,
    pub has_upper: bool,
    pub has_digit: bool,
    pub has_symbol: bool,
    /// For passphrase: word count and assumed wordlist size
    pub word_count: Option<usize>,
    pub assumed_wordlist_size: Option<usize>,
}

/// Return a detailed entropy profile for `s` using heuristics tuned for the CLI.
pub fn estimate_entropy_detailed(s: &str, style: &str) -> Result<EntropyProfile, String> {
    if style == "passphrase" {
        let words: Vec<&str> = s.split('-').filter(|w| !w.is_empty()).collect();
        let wordlist_size = 2048usize;
        let bits = (words.len() as f64) * (wordlist_size as f64).log2();
        return Ok(EntropyProfile {
            bits,
            charset_size: wordlist_size,
            per_char: (wordlist_size as f64).log2(),
            length: words.len(),
            has_lower: false,
            has_upper: false,
            has_digit: false,
            has_symbol: false,
            word_count: Some(words.len()),
            assumed_wordlist_size: Some(wordlist_size),
        });
    }
    // Use a conservative class-based estimator.
    let mut has_lower = false;
    let mut has_upper = false;
    let mut has_digit = false;
    let mut has_symbol = false;
    for ch in s.chars() {
        if ch.is_ascii_lowercase() {
            has_lower = true;
        } else if ch.is_ascii_uppercase() {
            has_upper = true;
        } else if ch.is_ascii_digit() {
            has_digit = true;
        } else {
            has_symbol = true;
        }
    }

    let mut charset = 0usize;
    if has_lower {
        charset += 26;
    }
    if has_upper {
        charset += 26;
    }
    if has_digit {
        charset += 10;
    }
    if has_symbol {
        charset += 32;
    }

    if charset < 2 && let Some(hint) = charset_size_for_style(style, false) {
        charset = hint;
    }

    if charset < 2 {
        return Err("cannot determine charset size for entropy estimation".into());
    }

    let per_char = (charset as f64).log2();
    let length = s.chars().count();
    let bits = per_char * (length as f64);

    Ok(EntropyProfile {
        bits,
        charset_size: charset,
        per_char,
        length,
        has_lower,
        has_upper,
        has_digit,
        has_symbol,
        word_count: None,
        assumed_wordlist_size: None,
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_entropy_lowercase_only() {
        let s = "lowercaseonly";
        let bits = estimate_entropy_for_str(s, "random").unwrap();
        let per = (26f64).log2();
        assert!((bits - per * (s.len() as f64)).abs() < 1e-6);
    }

    #[test]
    fn test_entropy_mixed() {
        let s = "Ab3$";
        let bits = estimate_entropy_for_str(s, "random").unwrap();
        // should detect lower+upper+digit+symbol => charset >= 26+26+10+32
        let charset = 26 + 26 + 10 + 32;
        let per = (charset as f64).log2();
        assert!((bits - per * (s.len() as f64)).abs() < 1e-6);
    }

    #[test]
    fn test_entropy_passphrase() {
        let s = "apple-banana-orange";
        let bits = estimate_entropy_for_str(s, "passphrase").unwrap();
        let expected = 3.0 * 2048f64.log2();
        assert!((bits - expected).abs() < 1e-6);
    }
}
