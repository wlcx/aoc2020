use std::env;
use std::fs::File;
use std::io::{prelude::*, BufReader};

type Line = (u8, u8, char, String);

fn parse_line(line: String) -> Line {
    // our input is of the form <lower>-<upper> <char>: <password>
    let split_input = line.split_whitespace().collect::<Vec<&str>>();
    // That gets us a vec of 3 strings
    // Lets split the limits into lower and upper, parsing to u8 as we go
    let split_limits = split_input[0]
        .split("-")
        .map(|i| str::parse::<u8>(i).unwrap())
        .collect::<Vec<u8>>();
    // And finally return everything in a convenient type
    (
        split_limits[0],
        split_limits[1],
        split_input[1].chars().next().unwrap(),
        split_input[2].to_owned(),
    )
}

fn validate_line(line: Line) -> bool {
    let range = line.0..=line.1; // Inclusive range!
    range.contains(&(line.3.chars().filter(|c| *c == line.2).count() as u8))
}

fn main() {
    let fname = env::args()
        .skip(1)
        .next()
        .expect("Please provide path to input file!");
    let file = BufReader::new(File::open(fname).unwrap());
    let valid_count = file
        .lines()
        .map(Result::unwrap)
        .map(parse_line)
        .map(validate_line)
        .filter(|x| *x)
        .count();
    println!("Valid count: {}", valid_count)
}
