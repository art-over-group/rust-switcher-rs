# Sticky Shift normalization

Fixes words typed with a stuck or accidentally held Shift, where the first
two letters come out capitalized:

```
СЛово → Слово        STrict → Strict
```

## Rule

The heuristic is deliberately conservative — it fires only when

- the word has at least 3 characters,
- exactly the first two letters are uppercase,
- all remaining letters are lowercase,
- the word is not protected in the user-terms dictionary
  (`%APPDATA%\RustSwitcher\user_terms.json`).

`Слово`, `слово` and `СЛОВО` are left untouched; digits and punctuation make
the pattern not apply.

## When it runs

Sticky-Shift normalization takes precedence over layout conversion inside the
AutoConvert pipeline: a mixed-case word typed in the *correct* layout is an
unambiguous typo signal, so the fix applies directly (no lingua confidence
check, and — importantly — **no layout switch**, since the layout was right
all along).

The trigger is the same as AutoConvert: a word completed by a delimiter
(space, punctuation) while the low-level keyboard hook is active. If
AutoConvert is disabled (feature off or paused), no sticky normalization runs.

## Configuration

- `sticky_shift_enabled` in `%APPDATA%\RustSwitcher\config.json`, default
  `true`; also the **Fix sticky Shift (СЛово → Слово)** checkbox in Settings
  (applied on Apply).

## Privacy

Pure local text transform; nothing is persisted or transmitted.
