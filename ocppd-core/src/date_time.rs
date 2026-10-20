use serde::{Deserialize, Deserializer, Serialize, Serializer, de};
use std::fmt;
use std::str::FromStr;

const MS_PER_DAY: i64 = 86_400_000;

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
pub struct DateTime {
    pub unix_ms: u64,
}

#[derive(Debug, Clone, PartialEq)]
pub struct InvalidDateTime(pub String);

impl fmt::Display for InvalidDateTime {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "invalid RFC 3339 date-time: {:?}", self.0)
    }
}

impl std::error::Error for InvalidDateTime {}

impl fmt::Display for DateTime {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        let days = self.unix_ms / MS_PER_DAY as u64;
        let ms_of_day = self.unix_ms % MS_PER_DAY as u64;
        let (year, month, day) = civil_from_days(days as i64);
        let seconds = ms_of_day / 1000;
        write!(
            f,
            "{year:04}-{month:02}-{day:02}T{:02}:{:02}:{:02}",
            seconds / 3600,
            seconds % 3600 / 60,
            seconds % 60
        )?;
        let ms = ms_of_day % 1000;
        if ms != 0 {
            write!(f, ".{ms:03}")?;
        }
        write!(f, "Z")
    }
}

impl FromStr for DateTime {
    type Err = InvalidDateTime;

    fn from_str(text: &str) -> Result<Self, InvalidDateTime> {
        parse(text).ok_or_else(|| InvalidDateTime(text.to_owned()))
    }
}

impl Serialize for DateTime {
    fn serialize<S: Serializer>(&self, serializer: S) -> Result<S::Ok, S::Error> {
        serializer.collect_str(self)
    }
}

impl<'de> Deserialize<'de> for DateTime {
    fn deserialize<D: Deserializer<'de>>(deserializer: D) -> Result<Self, D::Error> {
        let text = String::deserialize(deserializer)?;
        text.parse().map_err(de::Error::custom)
    }
}

fn parse(text: &str) -> Option<DateTime> {
    let bytes = text.as_bytes();
    let separators = [(4, b'-'), (7, b'-'), (13, b':'), (16, b':')];
    if bytes.len() < 20 || separators.iter().any(|&(at, byte)| bytes[at] != byte) {
        return None;
    }
    if !matches!(bytes[10], b'T' | b't') {
        return None;
    }
    let year = digits(text, 0, 4)?;
    let month = digits(text, 5, 7)?;
    let day = digits(text, 8, 10)?;
    let hour = digits(text, 11, 13)?;
    let minute = digits(text, 14, 16)?;
    let second = digits(text, 17, 19)?;
    if !(1..=12).contains(&month) || !(1..=days_in_month(year, month)).contains(&day) {
        return None;
    }
    if hour > 23 || minute > 59 || second > 59 {
        return None;
    }
    let (ms, zone) = fraction(text.get(19..)?)?;
    let offset_minutes = offset(zone)?;
    let seconds = hour * 3600 + minute * 60 + second - offset_minutes * 60;
    let unix_ms = days_from_civil(year, month, day) * MS_PER_DAY + seconds * 1000 + ms;
    u64::try_from(unix_ms)
        .ok()
        .map(|unix_ms| DateTime { unix_ms })
}

fn fraction(rest: &str) -> Option<(i64, &str)> {
    let Some(after_dot) = rest.strip_prefix('.') else {
        return Some((0, rest));
    };
    let end = after_dot
        .find(|c: char| !c.is_ascii_digit())
        .unwrap_or(after_dot.len());
    if end == 0 {
        return None;
    }
    let millis = &after_dot[..end.min(3)];
    let scale = 10_i64.pow(3 - millis.len() as u32);
    Some((millis.parse::<i64>().ok()? * scale, &after_dot[end..]))
}

fn offset(zone: &str) -> Option<i64> {
    if zone == "Z" || zone == "z" {
        return Some(0);
    }
    let sign = match zone.as_bytes() {
        [b'+', _, _, b':', _, _] => 1,
        [b'-', _, _, b':', _, _] => -1,
        _ => return None,
    };
    let hours = digits(zone, 1, 3)?;
    let minutes = digits(zone, 4, 6)?;
    if hours > 23 || minutes > 59 {
        return None;
    }
    Some(sign * (hours * 60 + minutes))
}

fn digits(text: &str, start: usize, end: usize) -> Option<i64> {
    let part = text.get(start..end)?;
    if !part.bytes().all(|byte| byte.is_ascii_digit()) {
        return None;
    }
    part.parse().ok()
}

fn days_in_month(year: i64, month: i64) -> i64 {
    let leap = year % 4 == 0 && (year % 100 != 0 || year % 400 == 0);
    match month {
        2 if leap => 29,
        2 => 28,
        4 | 6 | 9 | 11 => 30,
        _ => 31,
    }
}

fn days_from_civil(year: i64, month: i64, day: i64) -> i64 {
    let year = if month <= 2 { year - 1 } else { year };
    let era = year.div_euclid(400);
    let year_of_era = year - era * 400;
    let shifted_month = (month + 9) % 12;
    let day_of_year = (153 * shifted_month + 2) / 5 + day - 1;
    let day_of_era = year_of_era * 365 + year_of_era / 4 - year_of_era / 100 + day_of_year;
    era * 146_097 + day_of_era - 719_468
}

fn civil_from_days(days: i64) -> (i64, i64, i64) {
    let days = days + 719_468;
    let era = days.div_euclid(146_097);
    let day_of_era = days - era * 146_097;
    let year_of_era =
        (day_of_era - day_of_era / 1460 + day_of_era / 36_524 - day_of_era / 146_096) / 365;
    let day_of_year = day_of_era - (365 * year_of_era + year_of_era / 4 - year_of_era / 100);
    let shifted_month = (5 * day_of_year + 2) / 153;
    let day = day_of_year - (153 * shifted_month + 2) / 5 + 1;
    let month = if shifted_month < 10 {
        shifted_month + 3
    } else {
        shifted_month - 9
    };
    let year = year_of_era + era * 400 + i64::from(month <= 2);
    (year, month, day)
}

#[cfg(test)]
#[path = "date_time.test.rs"]
mod tests;
