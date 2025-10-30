# Genix

Genix is a small Rust CLI for password generation and lightweight strength analysis.

Quick start (PowerShell):

```powershell
# Build
cargo build --release

# Run generator (dev build)
cargo run -- generate --length 32

# Run the example
cargo run --example generate_example

# Run tests
cargo test

# Run benchmarks (dev; may take time)
cargo bench
```

See `rules.md` for canonical CLI examples and expected behaviors (flag names and UX).

Key files
- `rules.md` — canonical CLI examples and behavior.
- `src/lib.rs` — library entrypoint and CLI runner.
- `src/generate.rs` — generation logic.
- `src/entropy.rs` — entropy helpers and estimators.
- `src/clipboard.rs` — clipboard wrapper.
- `assets/eff_sample.txt` — small sample wordlist.
- `tests/` — integration tests.
- `benches/` — benchmark harness (criterion).

Generating API documentation

Use `cargo doc` to generate HTML API docs from the crate and the doc comments added to
the source files. To build and open docs locally run:

```powershell
cargo doc --no-deps --open
```

This will render module-level and function-level documentation (the `///` and
`//!` comments) so you can browse the public API.

Production & release notes
- CI: The repository ships a basic GitHub Actions workflow at `.github/workflows/ci.yml` which now runs:
	- `cargo fmt -- --check` (formatting)
	- `cargo clippy --all-targets --all-features -- -D warnings` (linting)
	- `cargo build --release` and `cargo test --all`
- Release guide: See `docs/DEPLOY.md` for recommended release steps including changelog, version bump, and optional publishing to crates.io.
- Clipboard: Clipboard operations are best-effort (uses `arboard`) and may fail on headless CI. Avoid `--clipboard` in CI workflows.

Developer toolchain
- Use the pinned toolchain in `rust-toolchain.toml`. Install components for development:
	- `rustup component add rustfmt clippy`


License: MIT (see `LICENSE`)
