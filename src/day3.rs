use std::env;
use std::fs::File;
use std::io::{prelude::*, BufReader};

struct Direction(i8, i8);

fn calculate<I>(lines: I, direction: Direction) -> usize
where
    I: Iterator<Item = String>,
{
    lines
        .step_by(direction.1 as usize) // Step every n lines, where n is our "down" step
        // Zip with our "x position" - an iterator starting at 0 and skipping our
        //"right" step each time
        .zip((0..).step_by(direction.0 as usize))
        // Each line repeats infinitely to the right - represent this as a cycling
        // iterator over the chars in the line. Skip along our x position and return
        // whether we collided or not.
        .map(|(line, x)| line.chars().cycle().skip(x).next().unwrap() == '#')
        .filter(|collision| *collision) // Filter for lines we collided in
        .count() // return a count of how many collisions there were
}
fn main() {
    // The usual - open a file from the 1st cli argument
    let fname = env::args()
        .skip(1)
        .next()
        .expect("Please provide path to input file!");
    let file = BufReader::new(File::open(fname).unwrap());

    let lines = file
        .lines() // Split the file into lines
        .map(Result::unwrap) // Unwrap errors
        .collect::<Vec<String>>();

    let collision_product = [(1, 1), (3, 1), (5, 1), (7, 1), (1, 2)]
        .into_iter()
        .map(|dir| calculate(lines.clone().into_iter(), Direction(dir.0, dir.1)))
        .product::<usize>();
    println!("Collision product: {}", collision_product);
}
