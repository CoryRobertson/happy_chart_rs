#![allow(deprecated)]

use chrono::{DateTime, NaiveDateTime, TimeZone, Utc};
use chrono_tz::Tz;
use chrono_tz::US::Pacific;
use serde::ser::SerializeStruct;
use serde::{Deserialize, Serialize, Serializer};
use std::fmt;
use std::fmt::Formatter;

#[derive(Deserialize, Clone, Debug)]
#[deprecated]
pub struct DayStat {
    pub rating: f32,
    pub date: i64,
    pub note: String,
}

impl DayStat {
    /// Returns the date of this `DayStat` modified to pacific time, this can be made to support more time zones if needed.
    #[tracing::instrument]
    pub fn get_date_time(&self) -> DateTime<Tz> {
        // pacific time zone conversion
        let utc = DateTime::<Utc>::from_utc(
            NaiveDateTime::from_timestamp_opt(self.date, 0).unwrap_or_default(),
            Utc,
        )
        .naive_utc();
        Pacific.from_utc_datetime(&utc)
    }
}

impl fmt::Display for DayStat {
    #[tracing::instrument(skip_all)]
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        //let mut str = "";

        f.write_str("Date: ")?;
        f.write_str(&self.get_date_time().to_string())?;
        f.write_str("\t")?;
        f.write_str("Rating: ")?;
        f.write_str(&self.rating.to_string())?;
        f.write_str("\t")?;
        f.write_str(&self.note)?;

        Ok(())
    }
}

impl Serialize for DayStat {
    #[tracing::instrument(skip_all)]
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        let mut state = serializer.serialize_struct("DayStat", 2)?;
        state.serialize_field("rating", &self.rating)?;
        state.serialize_field("date", &self.date)?;
        state.serialize_field("note", &self.note)?;
        state.end()
    }
}
