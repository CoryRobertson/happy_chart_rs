
pub mod daystat {

    use chrono::{DateTime, NaiveDateTime, Utc};
    use serde::{Serialize, Serializer};
    use serde::ser::SerializeStruct;

    pub struct DayStat {
        pub rating: f32,
        pub date: i64,
    }

    impl DayStat {
        //let dt = DateTime::<Utc>::from_utc(NaiveDateTime::from_timestamp(day.date, 0), Utc);
        pub fn getDateTime(&self) -> DateTime<Utc> {
            DateTime::<Utc>::from_utc(NaiveDateTime::from_timestamp(self.date, 0), Utc)
        }
    }

    impl ToString for DayStat {
        fn to_string(&self) -> String {
            self.getDateTime().to_string()
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