# Conversion not working: recovery notes

This page is for fast diagnostics of `rust-switcher` enhanced MVP behavior.

## 0. Quick checks

1. Build with tracing:

```powershell
$env:RUST_LOG="trace"
cargo +nightly run -F debug-tracing
```

2. Repro in `Notepad` first (cleanest target).
3. Keep a terminal open and watch output.

## 1. Manual convert last word

Observed sequence in logs should be:

- keyboard hook receives key events
- last word token posted
- `convert_last_word_if_any` called
- replacement applied via `SendInput`

If no token appears:
- make sure you typed at least 3 letters (default `MIN_WORD_LEN = 3`)
- do not press delimiters/newlines: tokenization can split before conversion
- verify your foreground window allows key input simulation

If token appears but no SendInput success:
- app target may reject Unicode input
- shift state may not release
- use Notepad as isolated reproduction

## 2. Convert selection

- focus a selection in `Notepad`
- run double-tap Left Shift
- The app should probe UI Automation / supported Edit control
- then apply replacement and reselect converted span

If probe is `Absent`:
- selection might be stale/clipboard-only/unreadable
- fallback clipboard selection runner should be the last path

If probe is `Blocked`:
- target intentionally independent / no accessible selection
- should not call fallback insert

## 3. CapsLock layout switch

- check foreground layout has EN/RU layouts installed
- CapsLock may be intercepted globally by another IME
- switch language manually, then retry

## 4. Common root causes

- hotkey not applied (need press `Apply`)
- invalid sequence validator rejected binding
- selection control is not accessible
- target window changed between probe and input
- autoconvert was left disabled by default
- short tokens (<4) blocked by trigger threshold

## Recommended test matrix

- Notepad, selection
- Notepad, typed last sequence
- Google Chrome textarea (single line)
- Word desktop (selection path)
- cycle CapsLock switches layout

Keep `RUST_LOG=trace` output for exact step comparison.