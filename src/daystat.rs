
pub mod daystat {

    use std::fmt;
    use std::fmt::{Formatter};
    use chrono::{DateTime, NaiveDateTime, Utc, TimeZone};
    use serde::{Deserialize, Serialize, Serializer};
    use serde::ser::{SerializeStruct};
    use chrono_tz::Tz;
    use chrono_tz::US::Pacific;

    #[derive(Deserialize)]
    pub struct DayStat {
        pub rating: f32,
        pub date: i64,
        pub note: String,
    }
    //TODO: add some system for storing a note written for each day stat, allowing the user to write a reasoning for the rating or a description.

    impl DayStat {
        pub fn get_date_time(&self) -> DateTime<Tz> {
            // pacific time zone conversion
            let utc = DateTime::<Utc>::from_utc(NaiveDateTime::from_timestamp(self.date, 0), Utc).naive_utc();
            Pacific.from_utc_datetime(&utc)
        }

        pub fn get_hour_difference(&self, compare_day_stat: &DayStat) -> i64 {

            let difference = (self.date - compare_day_stat.date).abs();

            return difference;
        }

    }

    // impl ToString for DayStat {
    //     fn to_string(&self) -> String {
    //         self.get_date_time().to_string()
    //     }
    // }

    impl fmt::Display for DayStat {
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
        fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
            where
                S: Serializer,
        {
            let mut state = serializer.serialize_struct("DayStat",2)?;
            state.serialize_field("rating", &self.rating)?;
            state.serialize_field("date", &self.date)?;
            state.serialize_field("note", &self.note)?;
            state.end()
        }
    }

    
    
}