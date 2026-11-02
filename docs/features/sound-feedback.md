# Sound feedback

A short quiet tick plays after every successful conversion, matching the
overlay feedback point (autoconvert, last word, last sequence, selection).

## Design

- The cue is **synthesized procedurally** in `rust-switcher-core::sound` —
  a 45 ms, 1320 Hz sine with exponential decay, 16-bit mono 22 050 Hz PCM,
  rendered into an in-memory WAV buffer at first use. No audio asset files
  ship with the binary.
- The peak amplitude is capped at 25% of full scale, so the tick is audible
  but never intrusive.
- Playback uses `PlaySoundW(SND_MEMORY | SND_ASYNC | SND_NODEFAULT)` and never
  blocks the UI thread; failures are silent by design (sound is best-effort
  feedback, not a status channel).
- A longer descending two-tone cue (`error_cue_wav`) exists in the synth
  module for future failure paths; it is not played yet.

## Configuration

- `sound_enabled` in `%APPDATA%\RustSwitcher\config.json`, default `true`;
  also the **Play conversion sound** checkbox in Settings (applied on Apply).

## Privacy

The cue is generated from fixed constants; nothing about the user's text,
timing, or hardware leaves the process. See `docs/security/offline-first-mvp.md`.
