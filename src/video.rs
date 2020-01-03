use crate::filetools;

use log::debug;
use std::io;
use std::path::Path;
use std::process::Command;
use std::string::FromUtf8Error;
use regex::Regex;

pub struct VideoHandler {
    pattern: Regex,
    quicktime_pattern: Regex
}

impl VideoHandler {
    pub fn new() -> VideoHandler {
        VideoHandler {
            pattern: Regex::new(
                r"creation_time\s+[:]\s+(\d\d\d\d-\d\d-\d\d)T(\d\d:\d\d:\d\d)[.]\d+Z").unwrap(),
            quicktime_pattern: Regex::new(
                r"com.apple.quicktime.creationdate[:]\s+(\d\d\d\d-\d\d-\d\d)T(\d\d:\d\d:\d\d)+").unwrap()
        }
    }

    pub fn get_date_time<P: AsRef<Path>>(&self, p: P) -> io::Result<String> {
        let output = VideoHandler::get_ffmpeg_output(p.as_ref());

        if let Ok(ffmpeg_output) = output {
            if let Some(ct) = self.get_creationtime_from_string(ffmpeg_output) {
                return Ok(ct);
            }
        }

        if let Some(d) = VideoHandler::get_whatsapp_filename_date(p.as_ref()) {
            return Ok(d);
        }
        filetools::get_time_from_file(p.as_ref())
    }

    // TODO share with image via filetools?
    fn get_whatsapp_filename_date(path: &Path) -> Option<String> {
        let p = Regex::new(r"VID-(\d{8})-WA\d{4}.mp4").unwrap(); // TODO make constant
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

    fn get_creationtime_from_string(&self, s: String) -> Option<String> {
        // First let's see if there is quicktime creationdate information, as on IPhone-recorded movies that is 
        // more reliable than the 'creation_time' attribute...
        for line in s.lines() {
            if line.trim().starts_with("com.apple.quicktime.creationdate") {
                return Some(self.parse_quicktime_creation_date(line.to_string()));
            }
        }
        
        for line in s.lines() {
            if line.trim().starts_with("creation_time") {
                return Some(self.parse_creation_time(line.to_string()));
            }
        }
        
        None
    }

    fn parse_quicktime_creation_date(&self, s: String) -> String {
        if let Some(q) = self.quicktime_pattern.captures(s.as_str()) {
            let res = q[1].to_string() + " " + &q[2].to_string();
            res
        } else {
            "".to_string()
        }
    }

    fn parse_creation_time(&self, s: String) -> String {
        let x = self.pattern.captures(s.as_str());
        if let None = x {
            "".to_string()
        } else {
            let cap = x.unwrap();
            let res = cap[1].to_string() + " " + &cap[2].to_string();
            res
        }
    }

    fn get_ffmpeg_output(p: &Path) -> Result<String, FromUtf8Error> {
        let cmd = Command::new("ffmpeg")
            .arg("-i")
            .arg(p.to_str().unwrap())
            .arg("-dump")
            .output()
            .expect("Failed to execute ffmpeg, is it installed?");
        String::from_utf8(cmd.stderr)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::testtools;
    use chrono::DateTime;
    use chrono::offset::Utc;
    use std::fs;
    use std::io;

    #[test]
    fn test_video_date_time_metadata() {
        let s = testtools::get_base_dir() + "src/test/creation-time.mp4";
        let p1 = Path::new(s.as_str());
        assert_eq!("2019-05-01 17:40:16", VideoHandler::new().get_date_time(p1).unwrap());
    }

    #[test]
    fn test_video_date_time_file() -> io::Result<()> {
        let filename = testtools::get_base_dir() + "src/test/NO_METADATA.M4V";
        let p1 = Path::new(filename.as_str());
        let md = fs::metadata(&filename)?;
        let created: DateTime<Utc> = DateTime::from(md.created()?);
        let expected = format!("{}", created.format("%Y-%m-%d %T"));
        assert_eq!(expected, VideoHandler::new().get_date_time(p1)?);
        Ok(())
    }
}