pub fn format_bytes(bytes: u64) -> String {
    format_byte_value(bytes as f64)
}

pub fn format_rate(bytes_per_second: f64) -> String {
    if !bytes_per_second.is_finite() || bytes_per_second < 0.0 {
        return "-".to_owned();
    }
    format!("{}/s", format_byte_value(bytes_per_second))
}

pub fn format_uptime(seconds: u64) -> String {
    let days = seconds / 86_400;
    let hours = (seconds % 86_400) / 3_600;
    let minutes = (seconds % 3_600) / 60;
    let secs = seconds % 60;
    if days > 0 {
        format!("{days}d {hours}h {minutes}m")
    } else if hours > 0 {
        format!("{hours}h {minutes}m")
    } else if minutes > 0 {
        format!("{minutes}m {secs}s")
    } else {
        format!("{secs}s")
    }
}

fn format_byte_value(mut value: f64) -> String {
    const UNITS: [&str; 4] = ["B", "KiB", "MiB", "GiB"];
    let mut unit = 0;
    while value >= 1024.0 && unit < UNITS.len() - 1 {
        value /= 1024.0;
        unit += 1;
    }
    format!("{value:.1} {}", UNITS[unit])
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn formats_bytes_across_units() {
        assert_eq!(format_bytes(0), "0.0 B");
        assert_eq!(format_bytes(512), "512.0 B");
        assert_eq!(format_bytes(1024), "1.0 KiB");
        assert_eq!(format_bytes(1536), "1.5 KiB");
        assert_eq!(format_bytes(512 * 1024 * 1024), "512.0 MiB");
        assert_eq!(format_bytes(1024 * 1024 * 1024), "1.0 GiB");
    }

    #[test]
    fn formats_rates() {
        assert_eq!(format_rate(0.0), "0.0 B/s");
        assert_eq!(format_rate(1536.0), "1.5 KiB/s");
        assert_eq!(format_rate(-1.0), "-");
        assert_eq!(format_rate(f64::NAN), "-");
        assert_eq!(format_rate(f64::INFINITY), "-");
    }

    #[test]
    fn formats_uptime_scales() {
        assert_eq!(format_uptime(45), "45s");
        assert_eq!(format_uptime(90), "1m 30s");
        assert_eq!(format_uptime(3_700), "1h 1m");
        assert_eq!(format_uptime(90_000), "1d 1h 0m");
    }
}
