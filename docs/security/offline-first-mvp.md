# Offline-first MVP rules

These rules mirror the user privacy requirement for a KeyRay-inspired toolkit.

## Runtime behavior

- Keyboard and selected text stay on the device.
- no analytics, no telemetry, no data uploads by default.
- Any conversion decision is local.
- user dictionary storage is local JSON.
- network-dependent features (artifacts, updates) are opt-in.

## MVP offline contract

- Core conversion:
  - no network
- Dictionary handling:
  - local file only
- Crash logging:
  - local trace only; never submit automatically
- Hotkeys:
  - local Win32 message loop only
- Optional speech engine (future):
  - offline model only
  - cloud API becomes an explicit feature with user key

## Security and integrity

- build from source or verify published artifact against SHA256
- signed binary recommended but fallback manifest is metadata only
- keep dependency lock file via `--locked`
- run `cargo audit` in CI (already available in `.github/workflows/cargo-audit.yml`)

## Trust boundary checklist

- one machine, one config path: `%APPDATA%\RustSwitcher\config.json`
- no keylogger-style full event log persistence
- ring buffer is in-memory only
- any new feature that persists input must be in separate module and default off