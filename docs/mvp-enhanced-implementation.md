# Enhanced MVP implementation plan

Branch: `enhanced-mvp`

## Phase A: Reliability fix layer

- `A1`: selection conversion clipboard fallback for unreadable targets
- `A2`: typed input diagnostics over tracing
- `A3`: decrease/retain minimum typed word threshold
- `A4`: documented reproduction path
- `A5`: regression smoke tests

## Phase B: Feedback overlay layer — DONE (`codex/overlay-feedback`)

- `B1`: lightweight Win32 overlay (`WS_EX_LAYERED`) without focus steal — done
  (`src/platform/win/overlay.rs`)
- `B2`: conversion feedback auto-hide (1.4 s timer) — done
- `B3`: undo trigger — Ctrl+Z stays available; the overlay is informational and
  click-through, undo is covered by `richer_undo_enabled` reselection
- `B4`: toast fallback if overlay creation fails — done (tray balloon)
- `B5`: settings toggle for overlay enable — done (`overlay_enabled` checkbox)

## Phase C: Dictionary + forms

- `C1`: `user_terms.json` local loader
- `C2`: hot-reload by file mtime
- `C3`: case-sensitive override forms
- `C4`: ignore hard-protected feature terms
- `C5`: export/import settings UI

## Phase D: Release hardening

- `D1`: signed builds script (`src/scripts/release.*`)
- `D2`: reproducible build notes
- `D3`: marketplace distribution manifest
- `D4`: optional update feed doc with hashes

## Branch strategy

Per repository AGENTS.md, use one worktree/task/branch and keep primary local main only for sync.

Current tasks sequence:

1. start `enhanced-mvp` diagnostics patch
2. run:
   - `cargo +nightly fmt --check`
   - `cargo +nightly clippy --all-targets --all-features -- -D warnings`
   - `cargo +nightly build --features debug-tracing`
   - `cargo +nightly test --locked`
3. commit each feature when green