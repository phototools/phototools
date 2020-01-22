use crate::copier::DateResult;
use crate::filetools;
use crate::strings::Strings;

use log::debug;
use std::fs::File;
use std::path::Path;
use std::process::Command;
use regex::Regex;

pub struct PhotoHandler {}

impl PhotoHandler {
    // TODO refactor to get_date() as the time cannot always be obtained and we don't need it
    pub fn get_date_time(p: &Path) -> (DateResult, bool) {
        let f = File::open(p).unwrap();

        if let Ok(reader) = exif::Reader::new(&mut std::io::BufReader::new(&f)) {
            if let Some(v) = PhotoHandler::get_tag(&reader, exif::Tag::GPSTimeStamp) {
                if let Some(date) = PhotoHandler::get_tag(&reader, exif::Tag::GPSDateStamp) {
                    let date_time = date + " " + v.as_str();
                    return (DateResult::FromMetadata(Strings::truncate_at('.', date_time)), true);
                }
            }

            if let Some(v) = PhotoHandler::get_tag(&reader, exif::Tag::DateTimeOriginal) {
                return (DateResult::FromMetadata(v), true);
            }

            if let Some(v) = PhotoHandler::get_tag(&reader, exif::Tag::DateTime) {
                return (DateResult::FromMetadata(v), true);
            }
        }

        if let Some(v) = PhotoHandler::get_whatsapp_filename_date(p) {
            return (DateResult::Inferred(v), false);
        } else {
            debug!("No Exif tag found for date, using file date instead.");
            let md = f.metadata().unwrap();
            return (DateResult::Inferred(filetools::get_time_from_metadata(md).unwrap()), false);
        }
    }

    /*
    pub fn get_exif_reader(f: &File) -> Result<exif::Reader, exif::error::Error> {
        exif::Reader::new(&mut std::io::BufReader::new(&f))
    }
    */

    fn get_whatsapp_filename_date(path: &Path) -> Option<String> {
        let p = Regex::new(r"IMG-(\d{8})-WA\d{4}.jpg").unwrap(); // TODO make constant
        let f = &path.file_name().unwrap().to_string_lossy();
        let res = p.captures(f);
        if let Some(x) = res {
            let ds = x[1].to_string();
            debug!("Found a whatsapp file date! {:?}", ds);
            // TODO remove the dummy time of 1300 hrs
            let d = format!("{}-{}-{} 13:00:00", &ds[0..4], &ds[4..6], &ds[6..8]);
            return Some(d)
        }
        None
    }

    fn get_tag(reader: &exif::Reader, tag: exif::Tag) -> Option<String> {
        let field = reader.get_field(tag, false);
        if let None = field {
            None
        } else {
            let val = &field.unwrap().value;
            let t = *(&field.unwrap().tag);
            debug!("Value of tag {} is {}", tag, val.display_as(t));
            Some(format!("{}", val.display_as(t)))
        }
    }

    pub fn set_exif_date_time(file_name: & str, time_stamp: &str, create_exif: bool) {
        let ts = format!("-ts{}:{}:{}-{}:{}:{}", 
            &time_stamp[0..4], &time_stamp[5..7], &time_stamp[8..10],
            &time_stamp[11..13], &time_stamp[14..16], &time_stamp[17..19]);
        let mut jh = Command::new("jhead");
        let mut cmd = jh.arg(ts);
       
        if create_exif {
            cmd = cmd.arg("-mkexif");
        }
        cmd.arg(file_name).output().expect("Failed to execute jhead, is it installed?");
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::testtools;
    use chrono::DateTime;
    use chrono::offset::Utc;
    use std::io;
    use std::fs;
    
    #[test]
    fn test_photo_gps_date_time() {
        let s = String::from(testtools::get_base_dir() + "src/test/gps-date.jpg");
        let p = Path::new(&s);
        assert_eq!((DateResult::FromMetadata(String::from("2019-04-27 14:08:01")), true), 
            PhotoHandler::get_date_time(&p));
    }

    #[test]
    fn test_photo_date_time() -> io::Result<()> { 
        let filename = testtools::get_base_dir() + "src/test/NO_METADATA.JPEG";
        let md = fs::metadata(&filename)?;
        let created: DateTime<Utc> = DateTime::from(md.created()?);
        let expected = format!("{}", created.format("%Y-%m-%d %T"));
        assert_eq!((DateResult::Inferred(expected), false), 
            PhotoHandler::get_date_time(Path::new(&filename)));
        Ok(())
    }
}
