use std::env;
use std::fs::File;
use std::io::{prelude::*, BufReader};

fn main() {
    let fname = env::args()
        .skip(1)
        .next()
        .expect("Please provide path to input file!");
    let file = BufReader::new(File::open(fname).unwrap());
    let input: Vec<i32> = file
        .lines()
        .map(|val| i32::from_str_radix(&val.unwrap(), 10).unwrap())
        .collect();
    for x in 0..input.len() {
        for y in x + 1..input.len() {
            if input[x] + input[y] == 2020 {
                println!("part 1: {}", input[x] * input[y]);
            }
            for z in y + 1..input.len() {
                if input[x] + input[y] + input[z] == 2020 {
                    println!("part 2: {}", input[x] * input[y] * input[z])
                }
            }
        }
    }
}
