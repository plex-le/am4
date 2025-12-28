pub mod aircraft;
pub mod airport;
pub mod console;
pub mod help;
pub mod nav;
pub mod search;
pub mod settings;

/// Format a number with thousands separators
pub fn format_thousands(n: u32) -> String {
    let s = n.to_string();
    let mut result = String::new();
    for (i, c) in s.chars().rev().enumerate() {
        if i > 0 && i % 3 == 0 {
            result.insert(0, ',');
        }
        result.insert(0, c);
    }
    result
}
