use std::collections::HashSet;
use std::env;
use std::fs::File;
use std::io::{prelude::*, BufReader};

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
            _ => panic!("Unknown passport key!")
        }
    }
}

struct Passport {
    keys: HashSet<PassportKey>,
}

impl Passport {
    fn is_valid(&self) -> bool {
        match self.keys.len() {
            8 => true,
            7 => !self.keys.contains(&PassportKey::Cid),
            _ => false,
        }
    }
}

struct PassportParser<V> where V:BufRead {
    source: V,
}

impl<U> PassportParser<U> where U:BufRead {
    fn new(source: U) -> Self {
        PassportParser{
            source,
        }
    }
    fn parse(buf: &String) -> Passport {
        Passport{
            keys: buf.split_whitespace().map(|s| {
                PassportKey::from(s.split(":").next().unwrap())
            }).collect(),
        }
    }
}

impl<T> Iterator for PassportParser<T> where T:BufRead {
    type Item = Passport;

    fn next(&mut self) -> Option<Self::Item> {
        let mut buf = String::new();
        loop {
            match self.source.read_line(&mut buf) {
                Ok(0) => {  // EOF
                    if buf.len() == 0 {
                        return None
                    }
                    return Some(PassportParser::<T>::parse(&buf))  // parse whatever's left in the buffer
                },
                Ok(_) => { // We read a line
                    if buf.chars().rev().take(2).collect::<Vec<char>>()  == ['\n', '\n'] {
                        return Some(PassportParser::<T>::parse(&buf))
                    }
                },
                Err(e) => panic!(e),
            }
        }
    }

}

fn main() {
    let fname = env::args().skip(1).next().expect("Please provide path to input file!");
    let file = BufReader::new(File::open(fname).unwrap());

    let passport_count = PassportParser::new(file).filter(|p| p.is_valid()).count();
    println!("Passport count: {}", passport_count);
}