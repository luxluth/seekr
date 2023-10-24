use time::OffsetDateTime;

pub fn systemtime_strftime(system_time: std::time::SystemTime) -> String {
   let offset_date_time = OffsetDateTime::from(system_time);
   let date = offset_date_time.date();
   format!("{}", date).to_string()
}
