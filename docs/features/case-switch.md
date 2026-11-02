# Switch selection case

Cycles the case of the selected text in place:

```
hello → HELLO → Hello → hello
```

The current state is derived from the selection itself:

- all-lowercase → ALL UPPERCASE
- ALL UPPERCASE → Sentence case (first letter uppercase, the rest lowercase)
- mixed case → all lowercase

Punctuation, digits and spaces are preserved. Works in both Russian and
English; non-letter characters are never touched.

## How it works

The feature reuses the selection-conversion machinery: UI Automation / native
edit probe → deterministic case transform (`rust-switcher-core::text::case_switch`)
→ guarded `SendInput` replacement with optional reselection, so **Ctrl+Z
undoes** the change when `richer_undo_enabled` is on. The clipboard is never
touched and layouts are not switched.

## Configuration

- **Hotkey: unassigned by default.** This is deliberate: `RegisterHotKey`
  claims a chord system-wide, and popular defaults such as `Ctrl+Shift+Z`
  collide with "Redo" in Word, Chrome and VS Code. Open Settings, focus the
  **Switch selection case** field and press the chord you want (Backspace or
  Delete clears it). A sequence (for example a triple tap) can also be bound;
  it participates in the same duplicate-validation as other actions.
- The result is shown in the conversion overlay and confirmed with the sound
  tick like every other conversion.

## Privacy

Fully local: the transform is pure text processing; nothing leaves the
process.
