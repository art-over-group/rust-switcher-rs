# User terms dictionary

Path on Windows:

```text
%APPDATA%\RustSwitcher\user_terms.json
```

Example:

```json
{
  "schema_version": 1,
  "entries": [
    { "word": "vpn", "forms": [], "case_sensitive": false },
    { "word": "Xcode", "forms": ["xcode", "XCODE"], "case_sensitive": false }
  ]
}
```

Behavior:
- If `forms` is empty, the word is treated as protected and will not be converted.
- Other forms are accepted as variants for future dictionary-driven conversion.
- Missing or invalid JSON falls back to an empty dictionary.
- The loader uses only local files and never performs network access.