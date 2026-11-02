//! Sticky-Shift normalization (pure logic).
//!
//! Fixes words whose first two letters were typed with a stuck/pressed Shift:
//! `СЛово → Слово`, `STrict → Strict`. The rule is intentionally conservative:
//! exactly two leading uppercase letters, the rest lowercase, at least three
//! characters, and the word must be fully alphabetic.

/// Downcases the second letter of a word with a "СЛово" pattern.
///
/// Returns `None` when the pattern does not apply: fewer than three
/// characters, not exactly two leading uppercase letters followed by
/// lowercase letters, or non-alphabetic words. ALL-CAPS words are untouched.
pub fn fix_sticky_shift(word: &str) -> Option<String> {
    if word.chars().count() < 3 {
        return None;
    }

    let mut chars = word.chars();
    let first = chars.next()?;
    let second = chars.next()?;
    let rest: Vec<char> = chars.collect();

    if !first.is_uppercase() || !second.is_uppercase() {
        return None;
    }
    if !rest.iter().all(|ch| ch.is_lowercase()) {
        return None;
    }

    let mut fixed = String::with_capacity(word.len());
    fixed.extend(first.to_lowercase());
    fixed.extend(second.to_lowercase());
    fixed.extend(rest.iter().flat_map(|ch| ch.to_lowercase()));
    Some(fixed)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn fixes_double_capital_typos() {
        assert_eq!(fix_sticky_shift("СЛово"), Some("Слово".into()));
        assert_eq!(fix_sticky_shift("СТрока"), Some("Строка".into()));
        assert_eq!(fix_sticky_shift("STrict"), Some("Strict".into()));
    }

    #[test]
    fn leaves_normal_words_alone() {
        assert_eq!(fix_sticky_shift("слово"), None);
        assert_eq!(fix_sticky_shift("Слово"), None);
        assert_eq!(fix_sticky_shift("СЛОВО"), None);
    }

    #[test]
    fn requires_three_characters() {
        assert_eq!(fix_sticky_shift("СЛ"), None);
        assert_eq!(fix_sticky_shift("x"), None);
        assert_eq!(fix_sticky_shift(""), None);
    }

    #[test]
    fn rejects_non_alphabetic_words() {
        assert_eq!(fix_sticky_shift("СЛ123"), None);
        assert_eq!(fix_sticky_shift("СЛ_ово"), None);
    }

    #[test]
    fn handles_multibyte_input() {
        // Three Cyrillic letters where only the first two are uppercase.
        assert_eq!(fix_sticky_shift("СЛово"), Some("Слово".into()));
        // German lowercase umlaut stays intact.
        assert_eq!(fix_sticky_shift("STraße"), Some("Straße".into()));
    }
}
