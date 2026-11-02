# Conversion overlay

The overlay is a small frameless popup that appears next to the caret (or the
mouse cursor) whenever a conversion completes, showing the result as
`from → to`:

```
ghbdtn → привет
```

## Behavior

- Shown for all conversion flows: AutoConvert, last word, last sequence and
  selection conversion.
- The window is layered (`WS_EX_LAYERED`), topmost, click-through and never
  takes focus, so it cannot steal input from the application being typed in.
- Auto-hides after 1.4 seconds; rapid consecutive conversions replace the
  visible text.
- Follows the application theme (light/dark).
- Clamped to the monitor work area; placed above the caret when possible.

## Failure handling

If the overlay window cannot be created (for example a restricted desktop or a
window-station failure), the result falls back to a tray balloon notification.

## Configuration

- `overlay_enabled` in `%APPDATA%\RustSwitcher\config.json`, default `true`;
  also the **Show conversion overlay** checkbox in Settings (applied on Apply).
- When the overlay is enabled, the separate "Converted. Press Ctrl+Z to undo."
  last-word tray balloon is suppressed to avoid double feedback. Disabling the
  overlay restores the balloon.

## Privacy

The overlay only renders text that was just converted locally. Nothing is
logged, persisted or sent anywhere; see `docs/security/offline-first-mvp.md`.
