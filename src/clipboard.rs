/// Copy `s` to the system clipboard.
///
/// This is a thin wrapper around the `arboard` crate. On some platforms or in
/// headless CI environments clipboard initialization may fail — callers should
/// treat errors as non-fatal (the CLI prints a warning on failure).
///
/// Returns `Ok(())` on success or `Err(String)` describing the failure.
pub fn copy_to_clipboard(s: &str) -> Result<(), String> {
    let mut ctx = arboard::Clipboard::new().map_err(|e| format!("clipboard init: {}", e))?;
    ctx.set_text(s.to_owned())
        .map_err(|e| format!("clipboard set: {}", e))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn clipboard_copy_no_panic() {
        // Best-effort test: on CI this might fail depending on platform; we just ensure function doesn't panic.
        let _ = copy_to_clipboard("test");
    }
}
