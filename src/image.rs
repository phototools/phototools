use crate::filetools;

use log::debug;
use std::fs::File;
use std::path::Path;
use regex::Regex;

pub struct PhotoHandler {}

impl PhotoHandler {
    // TODO refactory to get_date() as the time cannot always be obtained and we don't need it
    pub fn get_date_time(p: &Path) -> String {
        let f = File::open(p).unwrap();

        // TODO get rid of all the unwraps here
        let reader = exif::Reader::new(&mut std::io::BufReader::new(&f)).unwrap();

        let v = PhotoHandler::get_tag(&reader, exif::Tag::GPSTimeStamp);
        if let None = v {
        } else {
            let date = PhotoHandler::get_tag(&reader, exif::Tag::GPSDateStamp);
            if let None = date {
            } else {
                return date.unwrap() + " " + v.unwrap().as_str();
            }
        }

        let v = PhotoHandler::get_tag(&reader, exif::Tag::DateTimeOriginal);
        if let None = v {
        } else {
            return v.unwrap();
        }

        let v = PhotoHandler::get_tag(&reader, exif::Tag::DateTime);
        if let None = v {
            let v = PhotoHandler::get_whatsapp_filename_date(p);
            if let None = v {
                debug!("No Exif tag found for date, using file date instead.");
                let md = f.metadata().unwrap();
                return filetools::get_time_from_metadata(md).unwrap()
            } else {
                return v.unwrap();
            }
        } else {
            v.unwrap()
        }
    }

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
        assert_eq!("2019-04-27 14:08:01", PhotoHandler::get_date_time(&p));
    }

    #[test]
    fn test_photo_date_time() -> io::Result<()> { 
        let filename = testtools::get_base_dir() + "src/test/NO_METADATA.JPEG";
        let md = fs::metadata(&filename)?;
        let created: DateTime<Utc> = DateTime::from(md.created()?);
        let expected = format!("{}", created.format("%Y-%m-%d %T"));
        assert_eq!(expected, PhotoHandler::get_date_time(Path::new(&filename)));
        Ok(())
    }
}
