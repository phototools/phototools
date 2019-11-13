use std::fs;

pub struct TestTools {}

pub const PROJECT_NAME: &'static str = "phototools";

pub fn get_base_dir() -> String {
    return find_base_dir(std::env::current_exe().unwrap().into_os_string().to_str().unwrap(), 
        PROJECT_NAME);
}

pub fn get_target_dir() -> String {
    return find_base_dir(std::env::current_exe().unwrap().into_os_string().to_str().unwrap(), 
        "target");
}

fn find_base_dir(path: &str, dirname: &str) -> String {
    let mut res = String::new();
    let mut found = false;
    for s in path.rsplit('/') {
        if s.eq(dirname) {
            found = true;
        }

        if found {
            res.insert_str(0, "/");
            res.insert_str(0, s);
        }
    }
    res
}

pub fn assert_files_equal<S1: Into<String>, S2: Into<String>>(f1: S1, f2: S2) {
    let fn1 = f1.into();
    let fn2 = f2.into();
    let bytes1 = fs::read(&fn1).unwrap();
    let bytes2 = fs::read(&fn2).unwrap();

    assert!(bytes1 == bytes2, "Files don't have the same content {} and {}", fn1, fn2);
}

mod tests {
    use super::*;

    #[test]
    fn test_get_base_dir() {
        assert_eq!("/foo/bar/phototools/", find_base_dir("/foo/bar/phototools/exec", "phototools"));
        assert_eq!("/foo/bar/phototools/", find_base_dir("/foo/bar/phototools/sub/sub/phototools-exec", "phototools"));
    }
}