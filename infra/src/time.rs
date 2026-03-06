//! Time formatting helpers for human and machine-readable output.

use chrono::{DateTime, FixedOffset, Offset, Utc};

use crate::InfraError;

/// Formats UTC datetime as RFC3339/ISO 8601.
pub fn to_iso8601(value: DateTime<Utc>) -> String {
    value.to_rfc3339()
}

/// Formats a UTC datetime in the provided timezone offset.
pub fn format_in_timezone(value: DateTime<Utc>, offset: FixedOffset) -> String {
    value
        .with_timezone(&offset)
        .format("%Y-%m-%d %H:%M:%S %:z")
        .to_string()
}

/// Parses a timezone offset in `+HH:MM` / `-HH:MM` form.
pub fn parse_timezone_offset(raw: &str) -> Result<FixedOffset, InfraError> {
    let Some((sign, rest)) = raw.chars().next().map(|c| (c, &raw[1..])) else {
        return Err(InfraError::InvalidTimezoneOffset(raw.to_string()));
    };

    let (hours, mins) = rest
        .split_once(':')
        .ok_or_else(|| InfraError::InvalidTimezoneOffset(raw.to_string()))?;

    let hours: i32 = hours
        .parse()
        .map_err(|_| InfraError::InvalidTimezoneOffset(raw.to_string()))?;
    let mins: i32 = mins
        .parse()
        .map_err(|_| InfraError::InvalidTimezoneOffset(raw.to_string()))?;

    if hours > 23 || mins > 59 {
        return Err(InfraError::InvalidTimezoneOffset(raw.to_string()));
    }

    let total_secs = hours * 3600 + mins * 60;
    let signed = match sign {
        '+' => total_secs,
        '-' => -total_secs,
        _ => return Err(InfraError::InvalidTimezoneOffset(raw.to_string())),
    };

    FixedOffset::east_opt(signed).ok_or_else(|| InfraError::InvalidTimezoneOffset(raw.to_string()))
}

/// Formats a relative `from` -> `to` duration in compact human form.
pub fn human_duration_ago(from: DateTime<Utc>, to: DateTime<Utc>) -> String {
    let mut seconds = (to - from).num_seconds();
    if seconds < 0 {
        seconds = 0;
    }

    if seconds < 5 {
        return "just now".to_string();
    }

    let days = seconds / 86_400;
    seconds %= 86_400;
    let hours = seconds / 3_600;
    seconds %= 3_600;
    let mins = seconds / 60;

    let mut parts = Vec::new();
    if days > 0 {
        parts.push(format!("{days}d"));
    }
    if hours > 0 {
        parts.push(format!("{hours}h"));
    }
    if mins > 0 {
        parts.push(format!("{mins}m"));
    }

    if parts.is_empty() {
        "just now".to_string()
    } else {
        format!("{} ago", parts.join(" "))
    }
}

/// Returns local timezone offset of current process for display defaults.
pub fn local_offset() -> FixedOffset {
    chrono::Local::now().offset().fix()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn duration_formats_expected_values() {
        let from = DateTime::from_timestamp(1_700_000_000, 0).expect("from ts");
        let to = DateTime::from_timestamp(1_700_008_100, 0).expect("to ts");
        assert_eq!(human_duration_ago(from, to), "2h 15m ago");
    }

    #[test]
    fn parses_offset() {
        let offset = parse_timezone_offset("-05:30").expect("offset");
        assert_eq!(offset.local_minus_utc(), -(5 * 3600 + 30 * 60));
    }
}
