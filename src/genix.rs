use std::fs::File;
use std::io::{BufRead, BufReader};

use clap::{ArgAction, Parser, Subcommand};
use rand::distributions::Uniform;
use rand::{thread_rng, Rng};

use base64::{engine::general_purpose, Engine as _};

/// Small, focused CLI implementation for genix used by tests and the binary.
#[derive(Parser, Debug)]
#[command(author, version, about, long_about = None)]
struct Cli {
    #[command(subcommand)]
    command: Commands,
}

#[derive(Subcommand, Debug)]
enum Commands {
    /// Generate passwords or passphrases
    Generate {
        /// Length (characters or bytes depending on style)
        #[arg(short = 'l', long = "length", default_value_t = 20usize)]
        length: usize,

        /// Number of items to generate
        #[arg(short = 'n', long = "count", default_value_t = 1usize)]
        count: usize,

        /// Style: random, passphrase, pin, hex, base64
        #[arg(long = "style", default_value = "random")]
        style: String,

        /// Copy first result to clipboard
        #[arg(long = "clipboard", action = ArgAction::SetTrue)]
        clipboard: bool,

        /// Use a custom wordlist file for passphrase style
        #[arg(long = "wordlist")]
        wordlist: Option<String>,

        /// Avoid ambiguous characters (1,l,I,0,O,|)
        #[arg(long = "no-ambiguous", action = ArgAction::SetTrue)]
        no_ambiguous: bool,

        /// Minimum entropy (bits). If provided, length may be auto-increased.
        #[arg(long = "min-entropy")]
        min_entropy: Option<f64>,
    },
}

pub fn run() {
    let cli = Cli::parse();
    match cli.command {
        Commands::Generate { length, count, style, clipboard, wordlist, no_ambiguous, min_entropy } => {
            let results = generate_many(&style, length, count, wordlist.as_deref(), no_ambiguous, min_entropy).unwrap_or_else(|e| {
                eprintln!("error: {}", e);
                std::process::exit(1);
            });

            for line in &results {
                println!("{}", line);
            }

            if clipboard && !results.is_empty() {
                if let Err(e) = copy_to_clipboard(&results[0]) {
                    eprintln!("warning: failed to copy to clipboard: {}", e);
                }
            }
        }
    }
}

fn copy_to_clipboard(s: &str) -> Result<(), String> {
    let mut ctx = arboard::Clipboard::new().map_err(|e| format!("clipboard init: {}", e))?;
    ctx.set_text(s.to_owned()).map_err(|e| format!("clipboard set: {}", e))
}

/// Public entry used by tests: generate `count` items with given style.
pub fn generate_many(
    style: &str,
    mut length: usize,
    count: usize,
    wordlist: Option<&str>,
    no_ambiguous: bool,
    min_entropy: Option<f64>,
) -> Result<Vec<String>, String> {
    // If min_entropy is set and style has a charset, adjust length.
    if let Some(bits) = min_entropy {
        if let Some(charset_size) = charset_size_for_style(style, no_ambiguous) {
            let per_char = (charset_size as f64).log2();
            if per_char <= 0.0 {
                return Err("invalid charset size for entropy calculation".into());
            }
            let needed = (bits / per_char).ceil() as usize;
            if needed > length {
                eprintln!("info: increasing length from {} to {} to satisfy min-entropy {} bits", length, needed, bits);
                length = needed;
            }
        }
    }

    match style {
        "random" => Ok((0..count).map(|_| random_string(length, no_ambiguous)).collect()),
        "pin" => Ok((0..count).map(|_| pin_string(length)).collect()),
        "hex" => Ok((0..count).map(|_| hex_string(length)).collect()),
        "base64" => Ok((0..count).map(|_| base64_string(length)).collect()),
        "passphrase" => {
            let words = load_wordlist(wordlist)?;
            if words.is_empty() {
                return Err("wordlist is empty".into());
            }
            Ok((0..count).map(|_| passphrase_from(&words, length)).collect())
        }
        _ => Err(format!("unknown style: {}", style)),
    }
}

fn charset_size_for_style(style: &str, no_ambiguous: bool) -> Option<usize> {
    match style {
        "random" => {
            let mut set = DEFAULT_PRINTABLE.chars().count();
            if no_ambiguous {
                // remove ambiguous: 1,l,I,0,O,|
                set -= 6;
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

const DEFAULT_PRINTABLE: &str = "ABCDEFGHIJKLMNOPQRSTUVWXYZabcdefghijklmnopqrstuvwxyz0123456789!@#$%&*()-_=+[]{};:,.<>?/`~";
const AMBIGUOUS: &str = "1lI0O|";

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

fn pin_string(len: usize) -> String {
    let mut rng = thread_rng();
    let dist = Uniform::from(0..10);
    (0..len).map(|_| char::from(b'0' + rng.sample(dist) as u8)).collect()
}

fn hex_string(bytes: usize) -> String {
    let mut rng = thread_rng();
    let mut buf = vec![0u8; bytes];
    rng.fill(&mut buf[..]);
    buf.iter().map(|b| format!("{:02x}", b)).collect()
}

fn base64_string(bytes: usize) -> String {
    let mut rng = thread_rng();
    let mut buf = vec![0u8; bytes];
    rng.fill(&mut buf[..]);
    general_purpose::STANDARD.encode(&buf)
}

fn load_wordlist(path: Option<&str>) -> Result<Vec<String>, String> {
    if let Some(p) = path {
        let file = File::open(p).map_err(|e| format!("failed to open wordlist {}: {}", p, e))?;
        let reader = BufReader::new(file);
        Ok(reader.lines().filter_map(Result::ok).map(|s| s.trim().to_string()).filter(|s| !s.is_empty()).collect())
    } else {
        // Fallback small built-in list — useful for smoke testing.
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

fn passphrase_from(words: &[String], target_words: usize) -> String {
    let mut rng = thread_rng();
    let dist = Uniform::from(0..words.len());
    (0..target_words).map(|_| words[rng.sample(dist)].clone()).collect::<Vec<_>>().join("-")
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
        // hex_string takes bytes -> 2 hex chars per byte
        let s = hex_string(4);
        assert_eq!(s.len(), 8);
    }

    #[test]
    fn test_base64() {
        let s = base64_string(3);
        // 3 bytes -> 4 base64 chars
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
        // For pin (10 symbols), per char entropy = log2(10) ~3.3219
        // For 40 bits, needed chars = ceil(40/3.3219) = 13
        let mut len = 6usize;
        let res = generate_many("pin", len, 1, None, false, Some(40.0)).unwrap();
        assert_eq!(res.len(), 1);
        assert!(res[0].len() >= 13);
    }
}
