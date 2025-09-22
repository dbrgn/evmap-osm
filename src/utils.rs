use std::path::Path;

pub fn log_info(msg: &str) {
    println!("\x1b[32m{}\x1b[0m", msg);
}

pub fn log_error(msg: &str) {
    eprintln!("\x1b[31m{}\x1b[0m", msg);
}

pub fn format_bytes(size: u64) -> String {
    const UNITS: &[(&str, u64)] = &[
        ("B", 1),
        ("K", 1024),
        ("M", 1024 * 1024),
        ("G", 1024 * 1024 * 1024),
    ];

    // Find the appropriate unit
    for i in (1..UNITS.len()).rev() {
        if size >= UNITS[i].1 {
            let size_f = size as f64;
            let unit_f = UNITS[i].1 as f64;
            return format!("{:.1}{}", size_f / unit_f, UNITS[i].0);
        }
    }

    // Fallback to bytes for small files
    format!("{}{}", size, UNITS[0].0)
}

pub fn get_file_size_human(path: &Path) -> String {
    match std::fs::metadata(path) {
        Ok(metadata) => format_bytes(metadata.len()),
        Err(e) => {
            log_error(&format!(
                "Failed to get file size for '{}': {}",
                path.display(),
                e
            ));
            "unknown".to_string()
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use rstest::rstest;

    #[rstest]
    #[case(0, "0B")]
    #[case(1, "1B")]
    #[case(512, "512B")]
    #[case(1023, "1023B")]
    #[case(1024, "1.0K")]
    #[case(1536, "1.5K")]
    #[case(1024 * 1024 - 1, "1024.0K")]
    #[case(1024 * 1024, "1.0M")]
    #[case(1024 * 1024 + 512 * 1024, "1.5M")]
    #[case(1024 * 1024 * 1024, "1.0G")]
    #[case(1024 * 1024 * 1024 * 2 + 512 * 1024 * 1024, "2.5G")]
    #[case(1024 * 1024 * 1024 * 1024, "1024.0G")]
    fn test_format_bytes(#[case] size: u64, #[case] expected: &str) {
        assert_eq!(format_bytes(size), expected);
    }
}
