use time::OffsetDateTime;

pub fn systemtime_strftime(system_time: std::time::SystemTime) -> String {
   let offset_date_time = OffsetDateTime::from(system_time);
   let date = offset_date_time.date();
   format!("{}", date).to_string()
}

/// format file size to human readable string 
/// B, KB, MB, GB, TB
pub fn format_size(f: u64) -> String {
    if f < 1024 { return format!("{} B", f).to_string(); }
    if f < 1024 * 1024 { return format!("{:.2} KB", f / 1024).to_string(); }
    if f < 1024 * 1024 * 1024 { return format!("{:.2} MB", f / 1024 / 1024).to_string(); }
    if f < 1024 * 1024 * 1024 * 1024 { return format!("{:.2} GB", f / 1024 / 1024 / 1024).to_string(); }
    format!("{:.2} TB", f / 1024 / 1024 / 1024 / 1024).to_string()
}
