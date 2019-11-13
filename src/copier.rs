use crate::image::PhotoHandler;
use crate::video::VideoHandler;
use crate::strings::Strings;

use filetime::{self, FileTime};
use std::io;
use std::fs::{self, File, DirEntry};
use std::path::Path;

type GenError = Box<dyn std::error::Error>;
pub type GenResult<T> = Result<T, GenError>;

pub struct Copier {
    video_handler: VideoHandler
}

impl Copier {
    pub fn new() -> Copier {
        Copier {
            video_handler: VideoHandler::new()
        }
    }

    pub fn copy(&self, from: &str, to: &str) -> GenResult<()> {
        let dir = Path::new(from);
        let t_dir = Path::new(to);

        self.visit_dirs(dir, t_dir, &|f, t| self.copy_direntry(f, t))
    }

    fn visit_dirs(&self, dir: &Path, tgt_dir: &Path, cb: &dyn Fn(&DirEntry, &Path)->GenResult<()>) -> GenResult<()> {
        if dir.is_dir() {
            for entry in fs::read_dir(dir)? {
                let entry = entry?;
                let path = entry.path();
                if path.is_dir() {
                    self.visit_dirs(&path, tgt_dir, cb)?;
                } else {
                    cb(&entry, tgt_dir)?;
                }
            }
        }
        Ok(())
    }

    fn copy_direntry(&self, direntry: &DirEntry, target_dir: &Path) -> GenResult<()> {
        println!("Copying {:?}", direntry);
        let p = direntry.path();
        self.copy_file(p, target_dir)?;
        Ok(())
    }

    fn copy_file<P: AsRef<Path>>(&self, p: P, target_dir: &Path) -> GenResult<()> {
        let ext = p.as_ref().extension().unwrap().to_str().unwrap();
        let ext = ext.to_lowercase();

        let ts = match ext.as_str() {
            "jpeg" |
            "jpg" => {
                    // photo
                    let f = File::open(&p).unwrap();
                    PhotoHandler::get_date_time(&f)
                },
            "mp4" | "m4v" => {
                    // video
                    self.video_handler.get_date_time(&p)?
            },
            _ => "cannot handle".to_string()
            // TODO handle properly, maybe log a warning...
        };

        println!("Found timestamp: {:?}", ts);
        let ts_date = Strings::truncate_at_space(ts.clone());

        if let Some(stem) = p.as_ref().file_name() {
            let stem = stem.to_string_lossy();
            let src_file: &str = &p.as_ref().to_string_lossy();
            let target_dir = target_dir.to_string_lossy().into_owned() + "/" + &ts_date;
            fs::create_dir_all(&target_dir)?;
            let target_file = target_dir + "/" + &stem;
            println!("Copying {} to {}", src_file, target_file);
            fs::copy(src_file, &target_file)?;

            println!("Setting file date and time to: {}", ts);
            let new_dt = chrono::NaiveDateTime::parse_from_str(&ts, "%Y-%m-%d %H:%M:%S")?;
            let unix_ts = FileTime::from_unix_time(new_dt.timestamp(), 0);
            filetime::set_file_times(&target_file, unix_ts, unix_ts)?;
            println!("...done");
            Ok(())
        } else {
            // TODO we should not need the GenError box
            Err(Box::new(io::Error::new(io::ErrorKind::InvalidData, format!("Problem with file: {:?}", p.as_ref()))))
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::filetools;
    use crate::testtools::get_target_dir;
    use crate::testtools::assert_files_equal;

    #[test]
    fn test_copy() {
        let td = get_target_dir();
        assert!(td.ends_with("/phototools/target/"));
        println!("Target dir {} ", td);

        let copier = Copier::new();
        let sd = td.clone() + "../src/test";
        let tdp1 = td.clone() + "test_photo";
        copier.copy(&sd, &tdp1).unwrap();

        assert_files_equal(sd.clone() + "/NO_METADATA.JPEG", tdp1.clone() + "/2019-08-09/NO_METADATA.JPEG");
        assert_files_equal(sd.clone() + "/creation-time.mp4", tdp1.clone() + "/2019-05-01/creation-time.mp4");
        // TODO check that the file time is modified too
        let file_time = filetools::get_time_from_file(tdp1.clone() + "/2019-05-01/creation-time.mp4").unwrap();
        assert_eq!("2019-05-01 17:40:16", file_time);
        assert_files_equal(sd.clone() + "/gps-date.jpg", tdp1.clone() + "/2019-04-27/gps-date.jpg");
        assert_files_equal(sd.clone() + "/gps-date copy.jpg", tdp1.clone() + "/2019-04-27/gps-date copy.jpg");
        // TODO check whatsapp images for file time.
    }

    // TODO check that the file is not overwritten, if the same filename exists
    #[test]
    fn test_dont_replace_same_file() {
        let td = get_target_dir();
        let copier = Copier::new();
        let source_dir_a = td.clone() + "../src/test1a";
        let target_dir = td.clone() + "test_photo1";
        copier.copy(&source_dir_a, &target_dir).unwrap();

        let subdir = check_dir_name(&target_dir, "2019-04-27");
        let file = check_dir_name(&subdir, "myimg.jpg");
        assert_eq!(204636, get_file_size(&file).unwrap());
        
        // Copy again, should not create another file
        copier.copy(&source_dir_a, &target_dir).unwrap();
        let file = check_dir_name(&subdir, "myimg.jpg");
        assert_eq!(204636, get_file_size(&file).unwrap());

        // Copy file from different directory. File has the same name, but different content
        // this should create a separate file 
        let source_dir_b = td.clone() + "../src/test1b";
        copier.copy(&source_dir_b, &target_dir).unwrap();
        let file = check_dir_names(&subdir, &["myimg.jpg", "myimg_01.jpg"]);
        assert_eq!(204636, get_file_size(&file).unwrap());
        // check second file size too TODO

        // Do the second copy again, this should not change any files
    }

    fn get_file_size(p: &str) -> GenResult<u64> {
        let md = fs::metadata(p)?;
        Ok(md.len())
    }

    fn check_dir_names(dir: &str, names: &[&str]) -> String {
        let dir = fs::read_dir(dir).unwrap();
        let paths: Vec<_> = dir.map(|res| res.unwrap().path()).collect();
        let mut found = names.to_vec();

        for path in &paths {
            let ps = path.to_string_lossy();
            found.retain(|name| {
                let suffix = "/".to_string() + name;
                !ps.ends_with(&suffix)
            });
        }

        assert_eq!(0, found.len(), "Not all expected files found {:?}", found);
        assert_eq!(names.len(), paths.len(), "Incorrect number of files found.");
        let path = &paths[0];
        let res = path.to_string_lossy();
        res.into_owned()
    }

    fn check_dir_name(dir: &str, name: &str) -> String {
        check_dir_names(dir, &[name])
    }
    // TODO specify a minimum size
    // TODO specify a start date
}
