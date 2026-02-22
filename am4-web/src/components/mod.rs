pub mod aircraft;
pub mod airport;
pub mod console;
pub mod help;
pub mod icons;
pub mod nav;
pub mod route;
pub mod search;
pub mod settings;

use am4::route::FlightTime;
use am4::user::CsvTimeFormat;

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

pub fn format_flight_time_hms(flight_time: FlightTime) -> String {
    let total_seconds = (flight_time.get() * 3600.0).round() as u64;
    let h = total_seconds / 3600;
    let m = (total_seconds % 3600) / 60;
    let s = total_seconds % 60;
    format!("{:02}:{:02}:{:02}", h, m, s)
}

pub fn format_flight_time_csv(flight_time: FlightTime, format: CsvTimeFormat) -> String {
    match format {
        CsvTimeFormat::HhMmSs => format_flight_time_hms(flight_time),
        CsvTimeFormat::Decimal => format!("{:.4}", flight_time.get()),
    }
}
