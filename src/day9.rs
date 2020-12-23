use std::env;
use std::fs;

fn main() {
    let fname = env::args()
        .skip(1)
        .next()
        .expect("Please provide path to input file!");
    let input = fs::read_to_string(fname)
        .unwrap()
        .lines()
        .map(str::parse::<u64>)
        .collect::<Result<Vec<u64>, _>>()
        .unwrap();

    let mut invalid = None;
    for window in input.windows(26) {
        if !window[0..25]
            .iter()
            .enumerate()
            .flat_map(|(i, x)| window[i + 1..25].iter().map(move |y| x + y))
            .any(|v| v == window[25])
        {
            println!("First non-sum of preamble: {}", window[25]);
            invalid = Some(window[25]);
        }
    }

    for i in 0..input.len() {
        for j in (i + 1)..input.len() {
            let slice = &input[i..=j];
            if slice.iter().sum::<u64>() == invalid.unwrap() {
                let minmax = slice.iter().fold((slice[0], slice[0]), |minmax, v| {
                    (minmax.0.min(*v), minmax.1.max(*v))
                });
                println!("part2: {}", minmax.0 + minmax.1);
                return;
            }
        }
    }
    println!("no value found :(");
}
