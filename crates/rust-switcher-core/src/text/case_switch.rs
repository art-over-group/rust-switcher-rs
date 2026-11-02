//! Case switching for selected text (pure logic).
//!
//! Deterministic cycle: `lower → UPPER → Sentence-case → lower`.
//! The current state is derived from the text itself, so the transform is
//! idempotent-safe and needs no external state.

/// Applies one step of the case cycle to `text`.
pub fn switch_case(text: &str) -> String {
    if is_all_lower(text) {
        text.to_uppercase()
    } else if is_all_upper(text) {
        to_sentence_case(text)
    } else {
        text.to_lowercase()
    }
}

fn is_all_lower(text: &str) -> bool {
    !text.chars().any(char::is_uppercase)
}

fn is_all_upper(text: &str) -> bool {
    let has_letter = text.chars().any(char::is_alphabetic);
    has_letter && !text.chars().any(char::is_lowercase)
}

/// First letter uppercase, the rest lowercase (only letters are considered).
fn to_sentence_case(text: &str) -> String {
    let mut out = String::with_capacity(text.len());
    let mut upper_done = false;
    for ch in text.chars() {
        if !upper_done && ch.is_alphabetic() {
            out.extend(ch.to_uppercase());
            upper_done = true;
        } else {
            out.extend(ch.to_lowercase());
        }
    }
    out
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn lower_goes_to_upper() {
        assert_eq!(switch_case("hello"), "HELLO");
        assert_eq!(switch_case("привет"), "ПРИВЕТ");
    }

    #[test]
    fn upper_goes_to_sentence_case() {
        assert_eq!(switch_case("HELLO"), "Hello");
        assert_eq!(switch_case("ПРИВЕТ"), "Привет");
    }

    #[test]
    fn sentence_case_goes_to_lower() {
        assert_eq!(switch_case("Hello"), "hello");
        assert_eq!(switch_case("Привет"), "привет");
    }

    #[test]
    fn mixed_text_normalizes_to_lower_first() {
        assert_eq!(switch_case("hELLO"), "hello");
        assert_eq!(switch_case("ПРивет"), "привет");
    }

    #[test]
    fn multi_word_selections_work() {
        assert_eq!(switch_case("hello world"), "HELLO WORLD");
        assert_eq!(switch_case("HELLO WORLD"), "Hello world");
        assert_eq!(switch_case("Hello world"), "hello world");
    }

    #[test]
    fn punctuation_and_digits_are_preserved() {
        assert_eq!(switch_case("hello, world 123!"), "HELLO, WORLD 123!");
        assert_eq!(switch_case("HELLO, WORLD 123!"), "Hello, world 123!");
    }

    #[test]
    fn text_without_letters_is_unchanged() {
        assert_eq!(switch_case("123 !?"), "123 !?");
        assert_eq!(switch_case(""), "");
    }

    #[test]
    fn cycle_is_idempotent_over_three_steps() {
        let mut text = String::from("проверка");
        let mut seen = Vec::new();
        for _ in 0..3 {
            text = switch_case(&text);
            seen.push(text.clone());
        }
        assert_eq!(seen, vec!["ПРОВЕРКА", "Проверка", "проверка"]);
        // Fourth step restarts the cycle.
        assert_eq!(switch_case(&text), "ПРОВЕРКА");
    }
}
