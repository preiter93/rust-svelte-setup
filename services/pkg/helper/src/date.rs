//! Convenient helper methods to deal with chrono datetime.
use chrono::{DateTime, NaiveDate, TimeZone, Utc};
use tonic::{Code, Status};

/// Converts a day, month, and year to a UNIX timestamp (seconds since epoch)
pub fn date_to_timestamp(day: u32, month: u32, year: i32) -> Result<i64, DateError> {
    let timestamp = date_to_datetime(day, month, year).map(|e| e.timestamp())?;

    Ok(timestamp)
}

/// Convert a `NaiveDate` to a UNIX timestamp at midnight UTC
pub fn naive_date_to_timestamp(date: NaiveDate) -> i64 {
    let datetime = date.and_hms_opt(0, 0, 0).unwrap(); // 00:00:00
    Utc.from_utc_datetime(&datetime).timestamp()
}

/// Converts a day, month, and year to a chrono datetime
pub fn date_to_datetime(day: u32, month: u32, year: i32) -> Result<DateTime<Utc>, DateError> {
    let date = NaiveDate::from_ymd_opt(year, month, day)
        .ok_or_else(|| DateError::InvalidDate(format!("{}/{}/{}", day, month, year)))?;

    let datetime = date.and_hms_opt(0, 0, 0).ok_or_else(|| {
        DateError::InvalidDate(format!("Invalid time for {}/{}/{}", day, month, year))
    })?;

    Ok(Utc.from_utc_datetime(&datetime))
}

/// Converts a `DateTime<Utc>` to a `NaiveDate` (day, month, year)
pub fn datetime_to_date(dt: DateTime<Utc>) -> NaiveDate {
    dt.date_naive()
}

/// Converts a UNIX timestamp (seconds since epoch) to (day, month, year)
pub fn timestamp_to_datetime(timestamp: i64) -> Result<DateTime<Utc>, DateError> {
    let datetime =
        DateTime::from_timestamp(timestamp, 0).ok_or(DateError::InvalidTimestamp(timestamp))?;

    Ok(datetime)
}

/// Converts a UNIX timestamp (seconds since epoch) to a `NaiveDate` (day, month, year)
pub fn timestamp_to_date(timestamp: i64) -> Result<NaiveDate, DateError> {
    let datetime =
        DateTime::from_timestamp(timestamp, 0).ok_or(DateError::InvalidTimestamp(timestamp))?;

    Ok(datetime.date_naive())
}

/// Converts a datetime timestamp to a day timestamp by stripping hours, minutes and seconds.
pub fn datetime_timestamp_to_date_timestamp(timestamp: i64) -> Result<i64, DateError> {
    let datetime = Utc
        .timestamp_opt(timestamp, 0)
        .single()
        .ok_or(DateError::InvalidTimestamp(timestamp))?;

    let date = datetime
        .date_naive()
        .and_hms_opt(0, 0, 0)
        .ok_or(DateError::InvalidDate(datetime.to_string()))?;

    Ok(date.and_utc().timestamp())
}

/// Returns the number of days of a given month in a given year.
pub fn number_days_of_month(year: i32, month: u32) -> Result<u8, DateError> {
    Ok(chrono::Month::try_from(month as u8)
        .map_err(DateError::ConvertMonth)?
        .num_days(year)
        .unwrap_or_default())
}

#[derive(Debug, thiserror::Error)]
pub enum DateError {
    #[error("Invalid date provided: {0}")]
    InvalidDate(String),

    #[error("Invalid timestamp: {0}")]
    InvalidTimestamp(i64),

    #[error("Convert month: {0}")]
    ConvertMonth(chrono::OutOfRange),
}

impl From<DateError> for Status {
    fn from(err: DateError) -> Self {
        let code = match err {
            DateError::InvalidDate(_)
            | DateError::InvalidTimestamp(_)
            | DateError::ConvertMonth(_) => Code::Internal,
        };
        Status::new(code, err.to_string())
    }
}
