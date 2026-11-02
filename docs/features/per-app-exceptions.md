# Per-app exceptions

AutoConvert is disabled in applications where an unexpected automated
correction could be harmful — first of all terminals and console hosts, where
a rewritten word becomes part of a shell command.

## Default exclusions

Matched by exact executable file name (case-insensitive, `.exe` optional):

```
cmd.exe, conhost.exe, powershell.exe, pwsh.exe, windowsterminal.exe,
alacritty.exe, wezterm.exe, ghostty.exe, soffice.exe
```

These mirror the defaults of commercial switchers (KeyRay) for the same
safety reason.

## Customizing

Edit `%APPDATA%\RustSwitcher\config.json`:

```json
{
  "excluded_apps": ["cmd.exe", "conhost.exe", "myterm.exe"]
}
```

- Set to `[]` to disable all exclusions.
- The list is matched against the **file name only** (e.g. `wt.exe` for
  Windows Terminal Store builds — add it if you use the Store version).
- Manual hotkeys (double Shift, selection conversion, case switch) are **not**
  blocked in excluded apps: exclusions only stop the automatic correction.
  This matches KeyRay's per-feature policy.

## How it works

At the moment AutoConvert attempts a correction, the foreground window's
process is resolved (`GetWindowThreadProcessId` → `OpenProcess` →
`QueryFullProcessImageNameW`) and the executable file name is checked against
the list. The resolution runs only on a delimiter-triggered attempt (not per
keystroke) and is deliberately uncached to avoid stale `pid → exe` mappings
after PID reuse.

## Privacy

Only the executable file name of the foreground app is inspected, locally;
nothing is recorded or transmitted.
