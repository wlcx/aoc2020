use std::collections::HashMap;
use std::convert::TryInto;
use std::env;
use std::fs::File;
use std::io::{prelude::*, BufReader};

struct BlankLineIter<T>
where
    T: Iterator<Item = String>,
{
    iter: T,
}

impl<T> BlankLineIter<T>
where
    T: Iterator<Item = String>,
{
    fn new(iter: T) -> Self {
        BlankLineIter { iter }
    }
}

impl<Iter: Iterator> Iterator for BlankLineIter<Iter>
where
    Iter: Iterator<Item = String>,
{
    type Item = Vec<String>;
    fn next(&mut self) -> Option<Vec<String>> {
        let mut buf = vec![];
        loop {
            match self.iter.next() {
                None => {
                    if buf.len() != 0 {
                        return Some(buf);
                    } else {
                        return None;
                    }
                }
                Some(s) if s == "" => {
                    // Handle multiple blank lines in a row
                    if buf.len() != 0 {
                        return Some(buf);
                    }
                }
                Some(s) => buf.push(s),
            }
        }
    }
}

fn main() {
    let fname = env::args()
        .skip(1)
        .next()
        .expect("Please provide a path to the input file!");
    let file = BufReader::new(File::open(fname).unwrap());

    let sum: usize = BlankLineIter::new(file.lines().map(Result::unwrap))
        .map(|group| {
            let mut map: HashMap<char, u16> = HashMap::new();
            let group_size = group.len();
            for s in group {
                for c in s.chars().filter(|c| c.is_alphabetic()) {
                    map.insert(c, map.get(&c).unwrap_or(&0u16) + 1);
                }
            }
            //map.len() // part 1
            map.iter()
                .map(|(_, count)| *count == group_size.try_into().unwrap())
                .filter(|x| *x)
                .count()
        })
        .sum();

    println!("Sum: {}", sum);
}

#[cfg(test)]
mod test {
    use super::*;

    #[test]
    fn test_blank_line_iter() {
        let input = "
hello
world

how are
you


today?";
        let mut i = BlankLineIter::new(input.lines().map(|s| s.to_owned().to_owned()));
        assert_eq!(
            i.next(),
            Some(vec!(String::from("hello"), String::from("world")))
        );
        assert_eq!(
            i.next(),
            Some(vec!(String::from("how are"), String::from("you")))
        );
        assert_eq!(i.next(), Some(vec!(String::from("today?"))));
    }
}
