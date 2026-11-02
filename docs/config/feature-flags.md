# Feature gates

The current MVP uses lightweight configuration fields instead of exposing a full
UI toggle. You can set these in `%APPDATA%\RustSwitcher\config.json`:

- `clipboard_fallback_enabled` defaults to `false`
- `tray_feedback_enabled` defaults to `true`
- `richer_undo_enabled` defaults to `true`
- `overlay_enabled` defaults to `true` (also exposed as the
  "Show conversion overlay" checkbox in Settings)
- `sound_enabled` defaults to `true` (also exposed as the
  "Play conversion sound" checkbox in Settings)

Notes:
- `clipboard_fallback_enabled` gates `convert_selection_clipboard_fallback`
  (`src/domain/text/convert.rs`): when the selection probe reports no selection,
  the fallback copies the selection via Ctrl+C, converts it, pastes it back, and
  restores the clipboard. Off by default because it sends keystrokes to the
  foreground app.
- `richer_undo_enabled` controls whether selection replacement reselects the
  converted span (`replace_selection_unicode(..., reselect)`) so that Ctrl+Z
  undoes the conversion.
- `overlay_enabled` gates the near-caret conversion overlay
  (`src/platform/win/overlay.rs`). When the overlay cannot be created, the
  result falls back to a tray balloon. When the overlay is enabled, the
  separate "Converted. Press Ctrl+Z to undo." last-word balloon is suppressed
  to avoid double feedback.
- `sound_enabled` gates the quiet conversion tick
  (`src/platform/win/sound.rs`). The cue is synthesized procedurally in
  `rust-switcher-core::sound` (no audio assets); playback via `PlaySoundW`
  is asynchronous and best-effort. See `docs/features/sound-feedback.md`.
- `grammar_correction_enabled` is reserved: no grammar feature is implemented
  (see `docs/features/grammar-correction.md`)
- all flags are absent in older config files and receive safe defaults via
  `#[serde(default)]`