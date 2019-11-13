use crate::filetools;

use std::io;
use std::path::Path;
use std::process::Command;
use std::string::FromUtf8Error;
use regex::Regex;

pub struct VideoHandler {
    pattern: Regex
}

impl VideoHandler {
    pub fn new() -> VideoHandler {
        VideoHandler {
            pattern: Regex::new(
                r"creation_time\s+[:]\s+(\d\d\d\d-\d\d-\d\d)T(\d\d:\d\d:\d\d)[.]\d+Z").unwrap()
        }
    }

    pub fn get_date_time<P: AsRef<Path>>(&self, p: P) -> io::Result<String> {
        let output = VideoHandler::get_ffmpeg_output(p.as_ref());

        let x = match output {
            Err(_) => filetools::get_time_from_file(p.as_ref()),
            Ok(s) => match self.get_creationtime_from_string(s) {
                Some(ct) => Ok(ct), // update time on file too TODO
                None => filetools::get_time_from_file(p.as_ref())
            }
        };

        x
    }

    fn get_creationtime_from_string(&self, s: String) -> Option<String> {
        for line in s.lines() {
            if line.trim().starts_with("creation_time") {
                return Some(self.parse_creation_time(line.to_string()));
            }
        }
        
        None
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

    #[test]
    fn test_video_date_time_metadata() {
        let s = testtools::get_base_dir() + "src/test/creation-time.mp4";
        let p1 = Path::new(s.as_str());
        assert_eq!("2019-05-01 17:40:16", VideoHandler::new().get_date_time(p1).unwrap());
    }

    #[test]
    fn test_video_date_time_file() {
        let s = testtools::get_base_dir() + "src/test/NO_METADATA.M4V";
        let p1 = Path::new(s.as_str());
        assert_eq!("2019-08-09 21:14:03", VideoHandler::new().get_date_time(p1).unwrap());
    }
}