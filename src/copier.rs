use crate::image::PhotoHandler;
use crate::video::VideoHandler;
use crate::strings::Strings;

use filetime::{self, FileTime};
use log::{info, debug};
use std::io;
use std::fs::{self, DirEntry};
use std::path::Path;
use std::process::Command;

type GenError = Box<dyn std::error::Error>;
pub type GenResult<T> = Result<T, GenError>;

pub struct Copier {
    min_size: u64,
    shell_cp: bool,
    video_handler: VideoHandler
}

impl Copier {
    pub fn new(min_size: u64, shell_cp: bool) -> Copier {
        Copier {
            min_size, 
            shell_cp,
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
        let p = direntry.path();

        let file_size = self.file_size(&p);
        debug!("File {:?} size {}", p, file_size); 
        if file_size >= self.min_size {
            self.copy_file(p, target_dir)?;
        } else {
            info!("Skipping {:?} as its size {} is less than {}", p, file_size, self.min_size);
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
            "mp4" | "m4v" | "mov" => {
                    // video
                    self.video_handler.get_date_time(&p)?
            },
            _ => {
                info!("Cannot handle {} - skipping.", p.as_ref().to_string_lossy());
                return Ok(());
            }
        };

        debug!("Found timestamp: {:?}", ts);
        let ts_date = Strings::truncate_at_space(ts.clone());

        if let Some(stem) = p.as_ref().file_name() {
            let stem = stem.to_string_lossy();
            let src_file: &str = &p.as_ref().to_string_lossy();
            let target_dir = target_dir.to_string_lossy().into_owned() + "/" + &ts_date[0..4] + "/" + &ts_date;
            fs::create_dir_all(&target_dir)?;
            let org_target_file = target_dir + "/" + &stem;

            let mut counter = 1;
            let mut path = Path::new(&org_target_file);
            let mut target_file = org_target_file.clone();
            while path.exists() {
                if Copier::identical_file(&p.as_ref(), &path) {
                    info!("Identical file already exists {}", target_file);
                    return Ok(());
                }

                let base;
                let ext;
                if let Some(idx) = org_target_file.rfind('.') {
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

            info!("Copying {} to {}", src_file, target_file);

            if self.shell_cp {
                Command::new("cp")
                    .arg(&src_file)
                    .arg(&target_file)
                    .output()
                    .expect("Failed to execute cp.");
            } else {
                fs::copy(src_file, &target_file)?;
            }
            

            debug!("Setting file date and time to: {}", ts);
            let new_dt = chrono::NaiveDateTime::parse_from_str(&ts, "%Y-%m-%d %H:%M:%S")?;
            let unix_ts = FileTime::from_unix_time(new_dt.timestamp(), 0);
            filetime::set_file_times(&target_file, unix_ts, unix_ts)?;
            Ok(())
            // TODO can we somehow delete the file if the copy didn't fully succeed?
        } else {
            // TODO we should not need the GenError box
            Err(Box::new(io::Error::new(io::ErrorKind::InvalidData, format!("Problem with file: {:?}", p.as_ref()))))
        }
    }

    fn file_size<P: AsRef<Path>>(&self, p: P) -> u64 {
        if let Ok(md) = fs::metadata(p) {
            return md.len();
        }
        std::u64::MAX
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
    use crate::filetools;
    use crate::testtools::get_target_dir;
    use crate::testtools::assert_files_equal;
    use chrono::DateTime;
    use chrono::offset::Utc;
    use std::fs;

    #[test]
    fn test_copy() -> io::Result<()> {
        // TODO make sure different tests write to different locations
        // TODO empty target directories before test run
        let td = get_target_dir();
        assert!(td.ends_with("/phototools/target/"));
        println!("Target dir {} ", td);

        let copier = Copier::new(0, false);
        let sd = td.clone() + "../src/test";
        let tdp1 = td.clone() + "test_photo";
        copier.copy(&sd, &tdp1).unwrap();

        let no_md_filename = sd.clone() + "/NO_METADATA.JPEG";
        let created: DateTime<Utc> = DateTime::from(fs::metadata(&no_md_filename)?.created()?);
        let expected_dir = format!("{}", created.format("/%Y/%Y-%m-%d/"));
        assert_files_equal(no_md_filename, tdp1.clone() + &expected_dir + "NO_METADATA.JPEG");

        let no_md_filename1 = sd.clone() + "/NO_METADATA.M4V";
        let created1: DateTime<Utc> = DateTime::from(fs::metadata(&no_md_filename1)?.created()?);
        let expected_dir1 = format!("{}", created1.format("/%Y/%Y-%m-%d/"));
        assert_files_equal(no_md_filename1, tdp1.clone() + &expected_dir1 + "NO_METADATA.M4V");

        assert_files_equal(sd.clone() + "/creation-time.mp4", tdp1.clone() + "/2019/2019-05-01/creation-time.mp4");
        let file_time = filetools::get_time_from_file(tdp1.clone() + "/2019/2019-05-01/creation-time.mp4")?;
        assert_eq!("2019-05-01 17:40:16", file_time);
        assert_files_equal(sd.clone() + "/gps-date.jpg", tdp1.clone() + "/2019/2019-04-27/gps-date.jpg");
        assert_files_equal(sd.clone() + "/gps-date copy.jpg", tdp1.clone() + "/2019/2019-04-27/gps-date copy.jpg");

        // check whatsapp images for file time.
        assert_files_equal(sd.clone() + "/IMG-20170701-WA0002.jpg", tdp1.clone() + "/2017/2017-07-01/IMG-20170701-WA0002.jpg");
        let file_time2 = filetools::get_time_from_file(tdp1.clone() + "/2017/2017-07-01/IMG-20170701-WA0002.jpg")?;
        let file_date = Strings::truncate_at_space(file_time2);
        assert_eq!("2017-07-01", file_date);
        
        assert_files_equal(sd.clone() + "/subdir/VID-20181129-WA9876.mp4", 
            tdp1.clone() + "/2018/2018-11-29/VID-20181129-WA9876.mp4");
        let file_time3 = filetools::get_time_from_file(tdp1.clone() + "/2018/2018-11-29/VID-20181129-WA9876.mp4")?;
        let file_date3 = Strings::truncate_at_space(file_time3);
        assert_eq!("2018-11-29", file_date3);

        // The following file makes the Exif reader complain
        let no_md_filename2 = sd.clone() + "/NO_EXIF.JPG";
        let modified2: DateTime<Utc> = DateTime::from(fs::metadata(&no_md_filename2)?.modified()?);
        let expected_dir2 = format!("{}", modified2.format("/%Y/%Y-%m-%d/"));
        assert_files_equal(no_md_filename2, tdp1.clone() + &expected_dir2 + "NO_EXIF.JPG");

        Ok(())
    }
    
    #[test]
    fn test_copy_using_cp() -> io::Result<()> {
        // TODO make sure different tests write to different locations
        let td = get_target_dir();
        assert!(td.ends_with("/phototools/target/"));
        println!("Target dir {} ", td);

        let copier = Copier::new(0, true);
        let sd = td.clone() + "../src/test";
        let tdp1 = td.clone() + "test_photo0";
        copier.copy(&sd, &tdp1).unwrap();

        let no_md_filename = sd.clone() + "/NO_METADATA.JPEG";
        let created: DateTime<Utc> = DateTime::from(fs::metadata(&no_md_filename)?.created()?);
        let expected_dir = format!("{}", created.format("/%Y/%Y-%m-%d/"));
        assert_files_equal(no_md_filename, tdp1.clone() + &expected_dir + "NO_METADATA.JPEG");

        let no_md_filename1 = sd.clone() + "/NO_METADATA.M4V";
        let created1: DateTime<Utc> = DateTime::from(fs::metadata(&no_md_filename1)?.created()?);
        let expected_dir1 = format!("{}", created1.format("/%Y/%Y-%m-%d/"));
        assert_files_equal(no_md_filename1, tdp1.clone() + &expected_dir1 + "NO_METADATA.M4V");

        assert_files_equal(sd.clone() + "/creation-time.mp4", tdp1.clone() + "/2019/2019-05-01/creation-time.mp4");
        let file_time = filetools::get_time_from_file(tdp1.clone() + "/2019/2019-05-01/creation-time.mp4")?;
        assert_eq!("2019-05-01 17:40:16", file_time);
        assert_files_equal(sd.clone() + "/gps-date.jpg", tdp1.clone() + "/2019/2019-04-27/gps-date.jpg");
        assert_files_equal(sd.clone() + "/gps-date copy.jpg", tdp1.clone() + "/2019/2019-04-27/gps-date copy.jpg");

        // check whatsapp images for file time.
        assert_files_equal(sd.clone() + "/IMG-20170701-WA0002.jpg", tdp1.clone() + "/2017/2017-07-01/IMG-20170701-WA0002.jpg");
        let file_time2 = filetools::get_time_from_file(tdp1.clone() + "/2017/2017-07-01/IMG-20170701-WA0002.jpg")?;
        let file_date = Strings::truncate_at_space(file_time2);
        assert_eq!("2017-07-01", file_date);
        
        assert_files_equal(sd.clone() + "/subdir/VID-20181129-WA9876.mp4", 
            tdp1.clone() + "/2018/2018-11-29/VID-20181129-WA9876.mp4");
        let file_time3 = filetools::get_time_from_file(tdp1.clone() + "/2018/2018-11-29/VID-20181129-WA9876.mp4")?;
        let file_date3 = Strings::truncate_at_space(file_time3);
        assert_eq!("2018-11-29", file_date3);

        Ok(())
    }

    #[test]
    fn test_min_size() -> io::Result<()> {
        let copier = Copier::new(100000, false);
        let sd = get_target_dir() + "../src/test";
        let td = get_target_dir() + "test_min_size";
        copier.copy(&sd, &td).unwrap();

        let td0 = td + "/2019";
        let td1 = dir_has_file(&td0, "2019-05-01");
        dir_exact(&td1, &["creation-time.mp4"]);

        let td2 = dir_has_file(&td0, "2019-04-27");
        dir_exact(&td2, &["gps-date.jpg", "gps-date copy.jpg"]);

        // check dir names = ensure no extra files
        dir_exact(&td0, &["2019-04-27", "2019-05-01"]);
        Ok(())
    }

    #[test]
    fn test_dont_replace_same_file() {
        let td = get_target_dir();
        let copier = Copier::new(0, false);
        let source_dir_a = td.clone() + "../src/test1a";
        let target_dir = td.clone() + "test_photo1";
        copier.copy(&source_dir_a, &target_dir).unwrap();

        let td0 = target_dir + "/2019";
        let subdir = dir_has_file(&td0, "2019-04-27");
        let file = dir_has_file(&subdir, "myimg.jpg");
        assert_eq!(204636, get_file_size(&file).unwrap());
        
        // Copy again, should not create another file
        copier.copy(&source_dir_a, &td0).unwrap();
        let file = dir_has_file(&subdir, "myimg.jpg");
        assert_eq!(204636, get_file_size(&file).unwrap());

        // Copy file from different directory. File has the same name, but different content
        // this should create a separate file 
        let target_dir_b = td.clone() + "test_photo2";
        copier.copy(&source_dir_a, &target_dir_b).unwrap();
        let source_dir_b = td.clone() + "../src/test1b";
        copier.copy(&source_dir_b, &target_dir_b).unwrap();
        let td_b = target_dir_b.clone() + "/2019";
        let subdir2 = dir_has_file(&td_b, "2019-04-27");
        dir_exact(&subdir2, &["myimg.jpg", "myimg_001.jpg"]);
        let file = dir_has_file(&subdir2, "myimg.jpg");
        assert_eq!(204636, get_file_size(&file).unwrap());
        let file2 = dir_has_file(&subdir2, "myimg_001.jpg");
        assert_eq!(204717, get_file_size(&file2).unwrap());

        // Copy another different file with the same name, it should be kept separate
        let target_dir_c = td.clone() + "./test_photo2";
        let source_dir_c = td.clone() + "../src/test1c";
        copier.copy(&source_dir_c, &target_dir_c).unwrap();
        dir_exact(&subdir2, &["myimg.jpg", "myimg_001.jpg", "myimg_002.jpg"]);
        let file3 = dir_has_file(&subdir2, "myimg_002.jpg");
        assert_eq!(96593, get_file_size(&file3).unwrap());

        // Copy the file again that ended up as _001, it should not copy it again
        copier.copy(&source_dir_b, &target_dir_b).unwrap();
        dir_exact(&subdir2, &["myimg.jpg", "myimg_001.jpg", "myimg_002.jpg"]);
        // Check that all the file sizes are as before
        assert_eq!(204636, get_file_size(&file).unwrap());
        assert_eq!(204717, get_file_size(&file2).unwrap());
        assert_eq!(96593, get_file_size(&file3).unwrap());
    }

    #[test]
    fn test_iphone_mov() {
        let td = get_target_dir();
        assert!(td.ends_with("/phototools/target/"));

        let copier = Copier::new(0, false);
        let sd = td.clone() + "../src/test2";
        let tdp1 = td.clone() + "test_mov";
        copier.copy(&sd, &tdp1).unwrap();

        assert_files_equal(sd.clone() + "/FROM_IPHONE.MOV", tdp1.clone() + "/2018/2018-06-02/FROM_IPHONE.MOV");
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
