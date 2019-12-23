use crate::filetools;

use std::fs::File;

pub struct PhotoHandler {}

impl PhotoHandler {
    pub fn get_date_time(f: &File) -> String {
        // TODO get rid of all the unwraps here
        let reader = exif::Reader::new(&mut std::io::BufReader::new(f)).unwrap();

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
            println!("No Exif tag found for date, using file date instead.");
            let md = f.metadata().unwrap();
            filetools::get_time_from_metadata(md).unwrap()
        } else {
            v.unwrap()
        }
    }

    fn get_tag(reader: &exif::Reader, tag: exif::Tag) -> Option<String> {
        let field = reader.get_field(tag, false);
        if let None = field {
            None
        } else {
            let val = &field.unwrap().value;
            let t = *(&field.unwrap().tag);
            println!("Value of tag {} is {}", tag, val.display_as(t));
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
        let f1 = File::open(testtools::get_base_dir() + "src/test/gps-date.jpg").unwrap();
        assert_eq!("2019-04-27 14:08:01", PhotoHandler::get_date_time(&f1));
    }

    #[test]
    fn test_photo_date_time() -> io::Result<()> { 
        let filename = testtools::get_base_dir() + "src/test/NO_METADATA.JPEG";
        let f1 = File::open(&filename)?;
        let md = fs::metadata(&filename)?;
        let created: DateTime<Utc> = DateTime::from(md.created()?);
        let expected = format!("{}", created.format("%Y-%m-%d %T"));
        assert_eq!(expected, PhotoHandler::get_date_time(&f1));
        Ok(())
    }
}
