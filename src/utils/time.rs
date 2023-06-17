use chrono::{DateTime, TimeZone, Utc, ParseError};

pub fn parse_timestamp(timestamp: &str) -> Result<DateTime<Utc>, ParseError> {
    let format = "%a, %d %b %Y %H:%M:%S %Z";
    Utc.datetime_from_str(timestamp, format)
}