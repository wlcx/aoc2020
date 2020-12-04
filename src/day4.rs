use lazy_static::lazy_static;
use regex::Regex;
use std::collections::HashMap;
use std::env;
use std::fs::File;
use std::io::{prelude::*, BufReader};

fn year_between(value: &str, lower: u16, upper: u16) -> bool {
    // value must be 4 digits, and lower <= value <= upper. Could have leading zeroes,
    // so check length explicitly
    if value.len() != 4 {
        return false;
    }
    if let Ok(year) = str::parse::<u16>(value) {
        return year >= lower && year <= upper;
    }
    false
}

#[derive(Hash, PartialEq, Eq)]
enum PassportKey {
    Byr,
    Iyr,
    Eyr,
    Hgt,
    Hcl,
    Ecl,
    Pid,
    Cid,
}

impl From<&str> for PassportKey {
    fn from(item: &str) -> Self {
        match item {
            "byr" => PassportKey::Byr,
            "iyr" => PassportKey::Iyr,
            "eyr" => PassportKey::Eyr,
            "hgt" => PassportKey::Hgt,
            "hcl" => PassportKey::Hcl,
            "ecl" => PassportKey::Ecl,
            "pid" => PassportKey::Pid,
            "cid" => PassportKey::Cid,
            _ => panic!("Unknown passport key!"),
        }
    }
}

impl PassportKey {
    fn validate(&self, value: &str) -> bool {
        match self {
            PassportKey::Byr => year_between(value, 1920, 2002),
            PassportKey::Iyr => year_between(value, 2010, 2020),
            PassportKey::Eyr => year_between(value, 2020, 2030),
            PassportKey::Hgt => {
                lazy_static! {
                    static ref RE: Regex = Regex::new(r"^(\d{2,3})(cm|in)$").unwrap();
                }
                if let Some(split_height) = RE.captures_iter(value).next() {
                    if let Ok(height) = split_height[1].parse::<u8>() {
                        return match &split_height[2] {
                            "cm" => height >= 150 && height <= 193,
                            "in" => height >= 59 && height <= 76,
                            _ => false, // Should never be reached
                        };
                    }
                }
                false
            }
            PassportKey::Hcl => {
                lazy_static! {
                    static ref RE: Regex = Regex::new(r"^#[0-9a-f]{6}$").unwrap();
                }
                RE.is_match(value)
            }
            PassportKey::Ecl => ["amb", "blu", "brn", "gry", "grn", "hzl", "oth"].contains(&value),
            PassportKey::Pid => {
                lazy_static! {
                    static ref RE: Regex = Regex::new(r"^\d{9}$").unwrap();
                }
                RE.is_match(value)
            }
            PassportKey::Cid => true,
        }
    }
}

struct Passport {
    data: HashMap<PassportKey, String>,
}

impl Passport {
    fn is_valid(&self) -> bool {
        match self.data.len() {
            8 => {} // we have every key
            7 => {
                // If we have Cid, we are missing one other mandatory key - so, invalid.
                if self.data.contains_key(&PassportKey::Cid) {
                    return false;
                }
            }
            _ => return false, // Any other number means we're missing mandatory keys.
        }
        self.data.iter().all(|(k, v)| k.validate(v))
    }
}

struct PassportParser<V>
where
    V: BufRead,
{
    source: V,
}

impl<U> PassportParser<U>
where
    U: BufRead,
{
    fn new(source: U) -> Self {
        PassportParser { source }
    }
    fn parse(buf: &String) -> Passport {
        Passport {
            data: buf
                .split_whitespace()
                .map(|s| {
                    let kv = s.split(":").collect::<Vec<&str>>();
                    (kv[0].into(), kv[1].to_owned())
                })
                .collect(),
        }
    }
}

impl<T> Iterator for PassportParser<T>
where
    T: BufRead,
{
    type Item = Passport;

    fn next(&mut self) -> Option<Self::Item> {
        let mut buf = String::new();
        loop {
            match self.source.read_line(&mut buf) {
                Ok(0) => {
                    // EOF
                    if buf.len() != 0 {
                        // If there's anything in the buffer, let's parse it...
                        return Some(PassportParser::<T>::parse(&buf));
                    }
                    return None; // ...otherwise we're finished.
                }
                Ok(_) => {
                    // We read a line, check to see if our last two chars in the buffer
                    // are newlines (which means we just had a blank line), in which
                    // case we have a complete passport to parse. Otherwise, loop  and
                    // read another line.
                    if buf.chars().rev().take(2).collect::<Vec<char>>() == ['\n', '\n'] {
                        return Some(PassportParser::<T>::parse(&buf));
                    }
                }
                Err(e) => panic!(e),
            }
        }
    }
}

fn main() {
    let fname = env::args()
        .skip(1)
        .next()
        .expect("Please provide path to input file!");
    let file = BufReader::new(File::open(fname).unwrap());

    let passport_count = PassportParser::new(file).filter(|p| p.is_valid()).count();
    println!("Valid passport count: {}", passport_count);
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn test_byr() {
        for test in [
            ("1920", true),
            ("2002", true),
            ("1919", false),
            ("2003", false),
            ("02002", false),
        ]
        .iter()
        {
            assert_eq!(PassportKey::Byr.validate(test.0), test.1);
        }
    }

    #[test]
    fn test_iyr() {
        for test in [
            ("2010", true),
            ("2020", true),
            ("2009", false),
            ("2021", false),
        ]
        .iter()
        {
            assert_eq!(PassportKey::Iyr.validate(test.0), test.1);
        }
    }

    #[test]
    fn test_eyr() {
        for test in [
            ("2020", true),
            ("2030", true),
            ("2019", false),
            ("2031", false),
        ]
        .iter()
        {
            assert_eq!(PassportKey::Eyr.validate(test.0), test.1);
        }
    }

    #[test]
    fn test_hgt() {
        for test in [
            ("", false),
            ("169", false),
            ("in", false),
            ("cm", false),
            ("150cm", true),
            ("193cm", true),
            ("149cm", false),
            ("194cm", false),
            ("59in", true),
            ("76in", true),
            ("58in", false),
            ("77in", false),
            ("160cms", false),
            ("60i", false),
        ]
        .iter()
        {
            assert_eq!(
                PassportKey::Hgt.validate(test.0),
                test.1,
                "{} did not validate to {}!",
                test.0,
                test.1
            );
        }
    }

    #[test]
    fn test_hcl() {
        for test in [
            ("#1a2b3f", true),
            ("1a2b3f", false),
            ("#12", false),
            ("#aaaaaaa", false),
            ("#11111g", false),
        ]
        .iter()
        {
            assert_eq!(
                PassportKey::Hcl.validate(test.0),
                test.1,
                "{} did not validate to {}!",
                test.0,
                test.1
            );
        }
    }

    #[test]
    fn test_ecl() {
        for test in [("gry", true), ("aaa", false), ("lolo", false)].iter() {
            assert_eq!(
                PassportKey::Ecl.validate(test.0),
                test.1,
                "{} did not validate to {}!",
                test.0,
                test.1
            );
        }
    }

    #[test]
    fn test_pid() {
        for test in [
            ("000000000", true),
            ("123456789", true),
            ("1232", false),
            ("7382197389217", false),
        ]
        .iter()
        {
            assert_eq!(
                PassportKey::Pid.validate(test.0),
                test.1,
                "{} did not validate to {}!",
                test.0,
                test.1
            );
        }
    }
}
