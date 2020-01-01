pub struct Strings {}

impl Strings {
    pub fn truncate_at_space(s: String) -> String {
        Strings::truncate_at(' ', s)
    }

    pub fn truncate_at(c: char, mut s: String) -> String {
        let idx = s.find(c);
        match idx {
            Some(v) => s.truncate(v),
            None => {}
        }
        s
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_truncate() {
        assert_eq!("foo", Strings::truncate_at_space("foo bar tar".to_string()));
        assert_eq!("foo", Strings::truncate_at_space("foo".to_string()));
        assert_eq!("", Strings::truncate_at_space("".to_string()));
    }
}
