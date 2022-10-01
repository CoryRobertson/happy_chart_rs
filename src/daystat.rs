
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
    }

    impl DayStat {
        pub fn get_date_time(&self) -> DateTime<Tz> {
            // pacific time zone conversion
            let utc = DateTime::<Utc>::from_utc(NaiveDateTime::from_timestamp(self.date, 0), Utc).naive_utc();
            Pacific.from_utc_datetime(&utc)
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
            state.end()
        }
    }

    
    
}