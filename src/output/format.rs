use chrono::{DateTime, Utc};

pub(crate) fn thousands(n: usize) -> String {
    let digits = n.to_string();
    let mut groups = Vec::new();
    let mut end = digits.len();
    while end > 3 {
        groups.push(&digits[end - 3..end]);
        end -= 3;
    }
    groups.push(&digits[..end]);
    groups.reverse();
    groups.join(",")
}

pub(crate) fn day(date: DateTime<Utc>) -> String {
    date.format("%Y-%m-%d").to_string()
}

pub(crate) fn short_hash(hash: &str) -> &str {
    hash.get(..7).unwrap_or(hash)
}

pub(crate) fn signed(sign: char, n: usize) -> String {
    format!("{sign}{}", thousands(n))
}

pub(crate) fn bar(value: usize, max: usize, width: usize) -> String {
    if value == 0 || max == 0 {
        return String::new();
    }
    let length = ((value as f64 / max as f64 * width as f64).round() as usize).max(1);
    "█".repeat(length)
}

pub(crate) fn bar_with_count(value: usize, max: usize, width: usize) -> String {
    let bar = bar(value, max, width);
    if bar.is_empty() {
        format!("{:<width$} {value}", "")
    } else {
        format!("{bar:<width$} {value}")
    }
}

pub(crate) fn sparkline(values: &[usize]) -> String {
    const LEVELS: [char; 8] = ['▁', '▂', '▃', '▄', '▅', '▆', '▇', '█'];
    let max = values.iter().copied().max().unwrap_or(0);
    values
        .iter()
        .map(|&v| {
            if v == 0 || max == 0 {
                '·'
            } else {
                let level = (v * LEVELS.len()).div_ceil(max).clamp(1, LEVELS.len());
                LEVELS[level - 1]
            }
        })
        .collect()
}

pub(crate) fn fit_right(text: &str, max: usize) -> String {
    if text.chars().count() <= max {
        return text.to_owned();
    }
    let kept: String = text.chars().take(max.saturating_sub(1)).collect();
    format!("{kept}…")
}

pub(crate) fn fit_left(text: &str, max: usize) -> String {
    let len = text.chars().count();
    if len <= max {
        return text.to_owned();
    }
    let kept: String = text.chars().skip(len - max.saturating_sub(1)).collect();
    format!("…{kept}")
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn thousands_separators() {
        assert_eq!(thousands(0), "0");
        assert_eq!(thousands(999), "999");
        assert_eq!(thousands(1_000), "1,000");
        assert_eq!(thousands(1_842), "1,842");
        assert_eq!(thousands(1_234_567), "1,234,567");
    }

    #[test]
    fn days_are_iso_dates() {
        let date = DateTime::from_timestamp(1_788_000_000, 0).unwrap();
        assert_eq!(day(date), "2026-08-29");
    }

    #[test]
    fn bars_scale_and_keep_small_values_visible() {
        assert_eq!(bar(0, 10, 20), "");
        assert_eq!(bar(10, 10, 20).chars().count(), 20);
        assert_eq!(bar(5, 10, 20).chars().count(), 10);
        assert_eq!(bar(1, 1000, 20).chars().count(), 1);
        assert_eq!(bar(5, 0, 20), "");
    }

    #[test]
    fn bar_with_count_aligns_the_number() {
        assert_eq!(bar_with_count(0, 10, 4), "     0");
        assert_eq!(bar_with_count(10, 10, 4), "████ 10");
        assert_eq!(bar_with_count(5, 10, 4), "██   5");
    }

    #[test]
    fn sparkline_levels() {
        assert_eq!(sparkline(&[0, 1, 8]), "·▁█");
        assert_eq!(sparkline(&[0, 0]), "··");
        assert_eq!(sparkline(&[]), "");
    }

    #[test]
    fn truncation_respects_the_limit() {
        assert_eq!(fit_right("short", 10), "short");
        assert_eq!(fit_right("abcdefghij", 5), "abcd…");
        assert_eq!(fit_left("src/output/terminal.rs", 12), "…terminal.rs");
        assert_eq!(fit_left("a.rs", 12), "a.rs");
        assert_eq!(fit_right("añb", 3), "añb");
    }

    #[test]
    fn short_hash_is_seven_characters() {
        assert_eq!(short_hash("8f3a21c9d0e1"), "8f3a21c");
        assert_eq!(short_hash("abc"), "abc");
    }
}
