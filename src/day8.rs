use nom::branch::alt;
use nom::bytes::complete::tag;
use nom::character::complete::{digit1, newline};
use nom::combinator::{all_consuming, map, recognize};
use nom::multi::separated_list1;
use nom::sequence::{separated_pair, terminated, tuple};
use nom::IResult;
use std::collections::HashSet;
use std::convert::{TryFrom, TryInto};
use std::env;
use std::fs::File;
use std::io::prelude::*;
use std::str::FromStr;

#[derive(Debug, PartialEq, Eq)]
enum Op {
    Acc(i16),
    Jmp(i16),
    Nop,
}

impl TryFrom<(&str, i16)> for Op {
    type Error = String;
    fn try_from(i: (&str, i16)) -> Result<Op, String> {
        match i.0 {
            "acc" => Ok(Op::Acc(i.1)),
            "jmp" => Ok(Op::Jmp(i.1)),
            "nop" => Ok(Op::Nop),
            s => Err(format!("Unknown opcode '{}'!", s)),
        }
    }
}

fn op(input: &str) -> IResult<&str, Op> {
    let (rem, op_tup) = separated_pair(
        alt((tag("acc"), tag("jmp"), tag("nop"))),
        tag(" "),
        map(
            recognize(tuple((alt((tag("+"), tag("-"))), digit1))),
            |raw| i16::from_str(raw).unwrap(),
        ),
    )(input)?;
    Ok((rem, op_tup.try_into().expect("Invalid instruction")))
}

fn program(input: &str) -> IResult<&str, Vec<Op>> {
    all_consuming(terminated(separated_list1(newline, op), newline))(input)
}

fn main() {
    let fname = env::args()
        .skip(1)
        .next()
        .expect("Please provide a path to the input file!");
    let mut input = String::new();
    File::open(fname)
        .unwrap()
        .read_to_string(&mut input)
        .expect("error reading file");
    let (_, prog) = program(&input).unwrap();

    // Let's run the program
    let mut pc = 0; // Program counter
    let mut acc = 0; // Accumulator
    let mut seen: HashSet<i16> = HashSet::new();
    loop {
        if seen.contains(&pc) {
            println!("About to run instruction twice. Acc: {}, PC: {}", acc, pc);
            return;
        }
        seen.insert(pc);
        match prog[pc as usize] {
            Op::Acc(n) => {
                acc += n;
                pc += 1;
            }
            Op::Jmp(n) => pc += n,
            Op::Nop => pc += 1,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_op() {
        assert_eq!(op("acc +69"), Ok(("", Op::Acc(69))));
        assert_eq!(op("jmp -257"), Ok(("", Op::Jmp(-257))));
        assert_eq!(op("nop -123"), Ok(("", Op::Nop)));
    }
}
