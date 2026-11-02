# Grammar correction: safe default

Grammar correction is **disabled by default**:
- Config: `grammar_correction_enabled` (default `false`)

This setting intentionally leaves grammar behavior optional to avoid silently
sending keystrokes or selected text to any external LLM. For an offline-first
KeyRay-style experience, local rules and protected user terms should be used
first; network-based correction should be behind *explicit user-supplied*
secrets and a separate opt-in screen.

Current implementation intentionally does not:
- invoke external LLM
- send keystrokes
- harmonize formatting across target classes

If a production grammar feature is added later, it should:
- operate only on selected text behind explicit hotkey
- validate target before/after substitution
- maintain undo semantics
- preserve user clipboard and dictionaries