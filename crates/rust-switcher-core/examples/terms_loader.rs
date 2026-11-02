use std::fs;

use rust_switcher_core::dictionary::TermDictionary;

fn main() {
    let path = std::env::args()
        .nth(1)
        .unwrap_or_else(|| "assets/user_terms.example.json".to_string());
    let input = fs::read_to_string(&path).unwrap_or_else(|error| {
        eprintln!("failed to read {path}: {error}");
        std::process::exit(1);
    });
    let dictionary = TermDictionary::from_json_str(&input).unwrap_or_else(|error| {
        eprintln!("invalid dictionary: {error}");
        std::process::exit(1);
    });

    for term in &dictionary.entries {
        println!(
            "{} -> {}",
            term.word,
            dictionary
                .forms_for(&term.word)
                .unwrap_or_default()
                .join(", ")
        );
        if dictionary.is_protected(&term.word) {
            println!("  protected");
        }
    }
}
