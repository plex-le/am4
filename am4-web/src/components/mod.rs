pub mod aircraft;
pub mod airport;
pub mod console;
pub mod help;
pub mod nav;
pub mod route;
pub mod search;
pub mod settings;

/// Format a number with thousands separators
pub fn format_thousands<T: ToString>(n: T) -> String {
    let s = n.to_string();
    let (integer_part, _decimal_part) = s.split_once('.').unwrap_or((s.as_str(), ""));
    // ignoring formatting decimal part for now
    let mut result = String::new();
    for (i, c) in integer_part.chars().rev().enumerate() {
        if i > 0 && i % 3 == 0 {
            result.insert(0, ',');
        }
        result.insert(0, c);
    }
    result
}
