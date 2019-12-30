use chrono::offset::Utc;
use chrono::DateTime;
use std::fs::{self, Metadata};
use std::io;
use std::path::Path;
use std::time::SystemTime;

pub fn get_time_from_file<P: AsRef<Path>>(p: P) -> io::Result<String> {
    get_time_from_metadata(fs::metadata(p)?)
}

pub fn get_time_from_metadata(md: Metadata) -> io::Result<String> {
    // let ct = md.created();
    let ct = md.modified();
    if let Ok(creation_time) = ct {
        Ok(format_time(creation_time))
    } else {
        let mt = md.modified()?;
        Ok(format_time(mt))
    }
}

fn format_time(t: SystemTime) -> String {
    let datetime: DateTime<Utc> = DateTime::from(t);
    let dt = datetime.format("%Y-%m-%d %T");
    format!("{}", dt)
}

// TODO unit tests