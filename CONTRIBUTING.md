# Contributing to Genix

Thanks for your interest in contributing! This document covers the most common contribution workflows and the project's expectations for changes.

Getting started
- Fork the repo and open a branch for your change: `git checkout -b feat/my-change`
- Keep changes focused: one logical change per PR.

Development checks (locally)
- Install the stable toolchain and components (recommended via `rustup`)
  - `rustup toolchain install stable`
  - `rustup component add rustfmt clippy`
- Run formatter and linter before opening a PR:
  - `cargo fmt --all`
  - `cargo clippy --all-targets --all-features -- -D warnings`
- Run tests: `cargo test`

Behavior and tests
- If you change CLI flags, update `rules.md` and add unit tests exercising the new behavior.
- Keep `src/main.rs` thin; add business logic to `src/lib.rs` and modules under `src/` so tests can call library functions.

PR etiquette
- Include a short description, motive, and testing steps in the PR body.
- Tag reviewers and reference any related issues.

CI and platform notes
- The GitHub Actions CI runs format, clippy, build, and tests on push/PR. Fix any CI failures before merging.
- If depending on native libraries (C/C++), prefer making the dependency optional behind a Cargo feature and document platform build requirements.

License
- By contributing you agree your changes are licensed under the project's license (see `LICENSE`).
