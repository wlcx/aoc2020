use std::convert::TryInto;
use std::env;
use std::fs::File;
use std::io::{prelude::*, BufReader};

fn main() {
    let fname = env::args()
        .skip(1)
        .next()
        .expect("Please provide path to input file!");
    let file = BufReader::new(File::open(fname).unwrap());
    let mut ids = file
        .lines()
        .map(Result::unwrap)
        .map(|s| parse_seat(&s).unwrap())
        .map(|(_, _, id)| id)
        .collect::<Vec<u16>>();
    ids.sort();

    for slice in ids.windows(2) {
        if slice[1] - slice[0] != 1 {
            println!("Found gap between {} and {}", slice[0], slice[1]);
        }
    }
    
    println!("Highest: {}", ids.iter().max().unwrap());
}

enum BSP {
    Lower,
    Upper,
}

fn do_bsp(input: &[BSP]) -> u8 {
    let initial_h = 2u8.pow(input.len().try_into().unwrap()) - 1;
    let (l, h) = input.iter().fold((0, initial_h), |(l, h), c| {
        let step = ((h - l) / 2) + 1;
        match c {
            BSP::Lower => (l, h - step),
            BSP::Upper => (l + step, h),
        }
    });
    if l != h {
        panic!("High ({}) != low {} !", h, l);
    }
    l
}

fn parse_seat(in_str: &str) -> Result<(u8, u8, u16), &str> {
    if in_str.len() != 10 {
        return Err("unexpected seat input length");
    }
    let chars = in_str
        .chars()
        .map(|c| match c {
            'F' | 'L' => BSP::Lower,
            'B' | 'R' => BSP::Upper,
            _ => unreachable!(),
        })
        .collect::<Vec<BSP>>();

    let row = do_bsp(&chars[0..=6]);
    let col = do_bsp(&chars[7..=9]);

    Ok((row, col, ((row as u16 * 8) + col as u16)))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_do_bsp() {
        for test in [
            (vec![BSP::Lower, BSP::Lower, BSP::Lower], 0),
            (vec![BSP::Upper, BSP::Upper, BSP::Upper], 7),
        ]
        .iter()
        {
            assert_eq!(do_bsp(&test.0), test.1);
        }
    }

    #[test]
    fn test_parse_seat() {
        for test in [
            ("BFFFBBFRRR", Ok((70, 7, 567))),
            ("FFFBBBFRRR", Ok((14, 7, 119))),
            ("BBFFBBFRLL", Ok((102, 4, 820))),
        ]
        .iter()
        {
            let got = parse_seat(test.0);
            assert_eq!(got, test.1);
        }
    }
}
