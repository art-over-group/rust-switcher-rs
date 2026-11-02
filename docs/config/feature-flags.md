# Feature gates

The current MVP uses lightweight configuration fields instead of exposing a full
UI toggle. You can set these in `%APPDATA%\RustSwitcher\config.json`:

- `clipboard_fallback_enabled` defaults to `false`
- `tray_feedback_enabled` defaults to `true`
- `richer_undo_enabled` defaults to `true`

Notes:
- `clipboard_fallback_enabled` gates `convert_selection_clipboard_fallback`
  (`src/domain/text/convert.rs`): when the selection probe reports no selection,
  the fallback copies the selection via Ctrl+C, converts it, pastes it back, and
  restores the clipboard. Off by default because it sends keystrokes to the
  foreground app.
- `richer_undo_enabled` controls whether selection replacement reselects the
  converted span (`replace_selection_unicode(..., reselect)`) so that Ctrl+Z
  undoes the conversion.
- `grammar_correction_enabled` is reserved: no grammar feature is implemented
  (see `docs/features/grammar-correction.md`)
- all flags are absent in older config files and receive safe defaults via
  `#[serde(default)]`