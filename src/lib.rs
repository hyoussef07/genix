//! Genix library crate
//!
//! This crate provides the core functionality for the `genix` CLI. It is
//! organized into small modules: `generate` (password/passphrase generation),
//! `clipboard` (cross-platform clipboard helper), and `entropy` (entropy
//! estimation and helpers). The binary `src/main.rs` calls `genix_lib::run()` to
//! execute the CLI.
//!
//! Public API
//!
//! - `run()` — CLI entrypoint used by the binary.
//!
//! See each module for detailed documentation on functions and behavior.

pub mod clipboard;
pub mod entropy;
pub mod generate;

use clap::{ArgAction, Parser, Subcommand};

use crate::clipboard::copy_to_clipboard;
use crate::generate::generate_many;

/// Top-level CLI types and runner. Keep `main.rs` thin.
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
    /// Estimate strength of a single string
    Check {
        /// Input string to check
        input: String,
        /// Optional style hint (random|passphrase|pin|hex|base64)
        #[arg(long = "style")]
        style: Option<String>,
    },
    /// Profile a password (gives entropy estimate and breakdown)
    Profile {
        input: String,
        #[arg(long = "style")]
        style: Option<String>,
    },
}

/// Run the Genix CLI.
///
/// This function is the high-level entrypoint used by the `genix` binary. It
/// parses CLI arguments (see `rules.md` for examples) and dispatches to module
/// functions. Errors are printed to stderr and cause the process to exit with
/// a non-zero code where appropriate.
///
/// Behavior summary:
/// - `generate` — produce one or more passwords/passphrases and optionally copy
///   the first result to the clipboard.
/// - `check` — print an estimated entropy (bits) for a single input string.
/// - `profile` — print a small profile (entropy and charset hint) for an input.
///
/// Example:
///
/// ```no_run
/// genix_lib::run(); // called from src/main.rs
/// ```
pub fn run() {
    let cli = Cli::parse();
    match cli.command {
        Commands::Generate {
            length,
            count,
            style,
            clipboard,
            wordlist,
            no_ambiguous,
            min_entropy,
        } => {
            let results = generate_many(
                &style,
                length,
                count,
                wordlist.as_deref(),
                no_ambiguous,
                min_entropy,
            )
            .unwrap_or_else(|e| {
                eprintln!("error: {}", e);
                std::process::exit(1);
            });

            for line in &results {
                println!("{}", line);
            }

            if clipboard && !results.is_empty() && let Err(e) = copy_to_clipboard(&results[0]) {
                eprintln!("warning: failed to copy to clipboard: {}", e);
            }
        }
        Commands::Check { input, style } => {
            let s = input;
            let st = style.as_deref().unwrap_or("random");
            match crate::entropy::estimate_entropy_detailed(&s, st) {
                Ok(profile) => {
                    println!("Estimated entropy: {:.2} bits", profile.bits);
                    let verdict = match profile.bits {
                        b if b < 40.0 => "very weak",
                        b if b < 64.0 => "weak",
                        b if b < 80.0 => "fair",
                        b if b < 128.0 => "strong",
                        _ => "very strong",
                    };
                    println!("Verdict: {}", verdict);
                }
                Err(e) => eprintln!("error estimating entropy: {}", e),
            }
        }
        Commands::Profile { input, style } => {
            let st = style.as_deref().unwrap_or("random");
            println!("Profile for: {} (style: {})", input, st);
            match crate::entropy::estimate_entropy_detailed(&input, st) {
                Ok(profile) => {
                    println!("Entropy: {:.2} bits", profile.bits);
                    if let Some(wc) = profile.word_count {
                        println!(
                            "Passphrase words: {} (assumed wordlist size: {})",
                            wc,
                            profile.assumed_wordlist_size.unwrap_or(0)
                        );
                        println!("Bits per word (assumed): {:.2}", profile.per_char);
                    } else {
                        println!("Length: {} chars", profile.length);
                        println!("Charset size (inferred): {} symbols", profile.charset_size);
                        println!("Bits/char: {:.3}", profile.per_char);
                        println!(
                            "Classes present: lower={}, upper={}, digits={}, symbols={}",
                            profile.has_lower,
                            profile.has_upper,
                            profile.has_digit,
                            profile.has_symbol
                        );
                    }
                    let verdict = match profile.bits {
                        b if b < 40.0 => "very weak",
                        b if b < 64.0 => "weak",
                        b if b < 80.0 => "fair",
                        b if b < 128.0 => "strong",
                        _ => "very strong",
                    };
                    println!("Verdict: {}", verdict);
                }
                Err(e) => eprintln!("error estimating entropy: {}", e),
            }
        }
    }
}
