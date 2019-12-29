use crate::image::PhotoHandler;
use crate::video::VideoHandler;
use crate::strings::Strings;

use filetime::{self, FileTime};
use std::io;
use std::fs::{self, DirEntry};
use std::path::Path;

type GenError = Box<dyn std::error::Error>;
pub type GenResult<T> = Result<T, GenError>;

pub struct Copier {
    min_size: u64,
    _verbosity: u8,
    video_handler: VideoHandler
}

impl Copier {
    pub fn new(min_size: u64, verbosity: u8) -> Copier {
        Copier {
            min_size, 
            _verbosity: verbosity,
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

        if self.minimum_size(&p) {
            self.copy_file(p, target_dir)?;
        }
        Ok(())
    }

    fn copy_file<P: AsRef<Path>>(&self, p: P, target_dir: &Path) -> GenResult<()> {
        let ext = p.as_ref().extension().unwrap().to_str().unwrap();
        let ext = ext.to_lowercase();

        let ts = match ext.as_str() {
            "jpeg" |
            "jpg" => {
                    // photo
                    // let f = File::open(&p).unwrap();
                    PhotoHandler::get_date_time(p.as_ref())
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
            let org_target_file = target_dir + "/" + &stem;

            let mut counter = 1;
            let mut path = Path::new(&org_target_file);
            let mut target_file = org_target_file.clone();
            while path.exists() {
                if Copier::identical_file(&p.as_ref(), &path) {
                    println!("Identical file already exists {}", target_file);
                    return Ok(());
                }

                let base;
                let ext;
                if let Some(idx) = org_target_file.find('.') {
                    base = &org_target_file[..idx];
                    ext = &org_target_file[idx..];
                } else {
                    base = &org_target_file;
                    ext = "";
                }
                // if target file exists, add _001
                target_file = format!("{}_{:03}{}", base, counter, ext);
                path = Path::new(&target_file);
                counter += 1;
            }

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

    fn minimum_size<P: AsRef<Path>>(&self, p: P) -> bool {
        if let Ok(md) = fs::metadata(p) {
            return md.len() >= self.min_size;
        }
        false
    }

    fn identical_file(p1: &Path, p2: &Path) -> bool {
        // TODO better file compare, also based on content...
        if let Ok(md1) = fs::metadata(p1) {
            if let Ok(md2) = fs::metadata(p2) {
                return md1.len() == md2.len();
            }
        }
        false
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::testtools::get_target_dir;
    use crate::testtools::assert_files_equal;
    use chrono::DateTime;
    use chrono::offset::Utc;
    use std::fs;

    #[test]
    fn test_copy() -> io::Result<()> {
        // TODO make sure different tests write to different locations
        let td = get_target_dir();
        assert!(td.ends_with("/phototools/target/"));
        println!("Target dir {} ", td);

        let copier = Copier::new(0, 0);
        let sd = td.clone() + "../src/test";
        let tdp1 = td.clone() + "test_photo";
        copier.copy(&sd, &tdp1).unwrap();

        let no_md_filename = sd.clone() + "/NO_METADATA.JPEG";
        let md = fs::metadata(&no_md_filename)?;
        let created: DateTime<Utc> = DateTime::from(md.created()?);
        let expected_dir = format!("{}", created.format("/%Y-%m-%d/"));

        assert_files_equal(no_md_filename, tdp1.clone() + &expected_dir + "NO_METADATA.JPEG");
        assert_files_equal(sd.clone() + "/creation-time.mp4", tdp1.clone() + "/2019-05-01/creation-time.mp4");
        // TODO check that the file time is modified too
        //let file_time = filetools::get_time_from_file(tdp1.clone() + "/2019-05-01/creation-time.mp4").unwrap();
        //assert_eq!("2019-05-01 17:40:16", file_time);
        assert_files_equal(sd.clone() + "/gps-date.jpg", tdp1.clone() + "/2019-04-27/gps-date.jpg");
        assert_files_equal(sd.clone() + "/gps-date copy.jpg", tdp1.clone() + "/2019-04-27/gps-date copy.jpg");

        // check whatsapp images for file time.
        assert_files_equal(sd.clone() + "/IMG-20170701-WA002.jpg", tdp1.clone() + "/2017-07-01/IMG-20170701-WA002.jpg");

        Ok(())
    }

    #[test]
    fn test_min_size() -> io::Result<()> {
        let copier = Copier::new(100000, 0);
        let sd = get_target_dir() + "../src/test";
        let td = get_target_dir() + "test_min_size";
        copier.copy(&sd, &td).unwrap();

        let td1 = dir_has_file(&td, "2019-05-01");
        dir_exact(&td1, &["creation-time.mp4"]);

        let td2 = dir_has_file(&td, "2019-04-27");
        dir_exact(&td2, &["gps-date.jpg", "gps-date copy.jpg"]);

        // check dir names = ensure no extra files
        dir_exact(&td, &["2019-04-27", "2019-05-01"]);
        Ok(())
    }

    // TODO check that the file is not overwritten, if the same filename exists
    #[test]
    fn test_dont_replace_same_file() {
        let td = get_target_dir();
        let copier = Copier::new(0, 0);
        let source_dir_a = td.clone() + "../src/test1a";
        let target_dir = td.clone() + "test_photo1";
        copier.copy(&source_dir_a, &target_dir).unwrap();

        let subdir = dir_has_file(&target_dir, "2019-04-27");
        let file = dir_has_file(&subdir, "myimg.jpg");
        assert_eq!(204636, get_file_size(&file).unwrap());
        
        // Copy again, should not create another file
        copier.copy(&source_dir_a, &target_dir).unwrap();
        let file = dir_has_file(&subdir, "myimg.jpg");
        assert_eq!(204636, get_file_size(&file).unwrap());

        // Copy file from different directory. File has the same name, but different content
        // this should create a separate file 
        let target_dir_b = td.clone() + "test_photo2";
        copier.copy(&source_dir_a, &target_dir_b).unwrap();
        let source_dir_b = td.clone() + "../src/test1b";
        copier.copy(&source_dir_b, &target_dir_b).unwrap();
        let subdir2 = dir_has_file(&target_dir_b, "2019-04-27");
        dir_exact(&subdir2, &["myimg.jpg", "myimg_001.jpg"]);
        let file = dir_has_file(&subdir2, "myimg.jpg");
        assert_eq!(204636, get_file_size(&file).unwrap());
        // check second file size too TODO

        // Do the second copy again, this should not change any files
    }

    fn get_file_size(p: &str) -> GenResult<u64> {
        let md = fs::metadata(p)?;
        Ok(md.len())
    }

    fn dir_exact(dir: &str, names: &[&str]) {
        check_dir_names(dir, names, true);
    }

    fn check_dir_names(dir: &str, names: &[&str], check_filenum: bool) {
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

        if check_filenum {
            assert_eq!(names.len(), paths.len(), "Incorrect number of files found. Expected: {:?} was {:?}", 
                names, paths);
        }
    }

    fn dir_has_file(dir: &str, name: &str) -> String {
        let mut d = String::from(dir);
        if !d.ends_with("/") {
            d += "/";
        }

        check_dir_names(&d, &[name], false);

        d += name;
        d
    }
    // TODO specify a start date
}
