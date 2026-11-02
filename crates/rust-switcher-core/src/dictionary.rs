//! Lightweight user-term dictionary support.
//!
//! The dictionary is plain JSON and intentionally supports offline-only use.
//! No network input or telemetry are required to load or consult it.

use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Term {
    pub word: String,
    pub forms: Vec<String>,
    #[serde(default)]
    pub case_sensitive: bool,
}

#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct TermDictionary {
    #[serde(default)]
    pub schema_version: u8,
    #[serde(default)]
    pub entries: Vec<Term>,
}

impl TermDictionary {
    pub fn from_json_str(input: &str) -> Result<Self, serde_json::Error> {
        serde_json::from_str(input)
    }

    /// Returns synonym or inflection forms when the exact term is known.
    ///
    /// Terms with `case_sensitive = false` are compared case-insensitively.
    pub fn forms_for(&self, input: &str) -> Option<Vec<String>> {
        self.entries.iter().find_map(|term| {
            let normalized = if term.case_sensitive {
                input.to_owned()
            } else {
                input.to_lowercase()
            };
            let expected = if term.case_sensitive {
                term.word.clone()
            } else {
                term.word.to_lowercase()
            };

            if normalized != expected {
                return None;
            }

            let mut forms = term.forms.clone();
            if !forms.iter().any(|form| form == &term.word) {
                forms.insert(0, term.word.clone());
            }
            Some(forms)
        })
    }

    /// Checks whether a word is explicitly protected and should not be converted.
    pub fn is_protected(&self, input: &str) -> bool {
        self.entries.iter().any(|term| {
            let normalized = if term.case_sensitive {
                input.to_owned()
            } else {
                input.to_lowercase()
            };
            let expected = if term.case_sensitive {
                term.word.clone()
            } else {
                term.word.to_lowercase()
            };

            normalized == expected && term.forms.is_empty()
        })
    }
}

impl From<&str> for TermDictionary {
    fn from(input: &str) -> Self {
        Self::from_json_str(input).unwrap_or_default()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn forms_are_returned_case_insensitively() {
        let dictionary = TermDictionary {
            schema_version: 1,
            entries: vec![Term {
                word: "Swift".to_string(),
                forms: vec!["swifts".to_string()],
                case_sensitive: false,
            }],
        };

        let values = dictionary.forms_for("SWIFT").unwrap();
        assert!(values.iter().any(|value| value == "Swift"));
        assert!(values.iter().any(|value| value == "swifts"));
    }

    #[test]
    fn empty_forms_protects_word() {
        let dictionary = TermDictionary {
            schema_version: 1,
            entries: vec![Term {
                word: "Xcode".to_string(),
                forms: vec![],
                case_sensitive: false,
            }],
        };

        assert!(dictionary.is_protected("xcode"));
    }

    #[test]
    fn json_can_be_loaded() {
        let json =
            r#"{"schema_version":1,"entries":[{"word":"vpn","forms":[],"case_sensitive":false}]}"#;
        let dictionary = TermDictionary::from_json_str(json).unwrap();
        assert!(dictionary.is_protected("vpn"));
    }
}
