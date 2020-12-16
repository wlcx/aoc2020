use nom::branch::alt;
use nom::bytes::complete::tag;
use nom::character::complete::{char, digit1, newline};
use nom::combinator::{all_consuming, map, map_res};
use nom::multi::{count, many1};
use nom::sequence::{delimited, preceded, separated_pair, terminated};
use nom::IResult;
use std::collections::HashMap;
use std::convert::{TryFrom, TryInto};
use std::env;
use std::fs;

#[derive(Debug, Clone, Copy, PartialEq)]
enum MaskBit {
    Set,
    Clear,
    Ignore,
}

impl TryFrom<char> for MaskBit {
    type Error = &'static str;
    fn try_from(c: char) -> Result<MaskBit, &'static str> {
        match c {
            '1' => Ok(MaskBit::Set),
            '0' => Ok(MaskBit::Clear),
            'X' => Ok(MaskBit::Ignore),
            _ => Err("Unknown bit state"),
        }
    }
}

type BitMask = [MaskBit; 36];

#[derive(Debug, Clone, PartialEq)]
enum Instruction {
    UpdateMask(BitMask),
    WriteValue(u64, u64),
}

fn parse_mask(input: &str) -> IResult<&str, Instruction> {
    map(
        preceded(
            tag("mask = "),
            count(
                map(alt((char('0'), char('X'), char('1'))), |c| {
                    c.try_into().unwrap()
                }),
                36,
            ),
        ),
        |v| Instruction::UpdateMask(v.try_into().unwrap()),
    )(input)
}

fn parse_mem(input: &str) -> IResult<&str, Instruction> {
    separated_pair(
        delimited(tag("mem["), map_res(digit1, str::parse::<u64>), tag("]")),
        tag(" = "),
        map_res(digit1, str::parse::<u64>),
    )(input)
    .map(|(rem, w)| (rem, Instruction::WriteValue(w.0, w.1)))
}

fn parse_program(input: &str) -> IResult<&str, Vec<Instruction>> {
    all_consuming(many1(terminated(alt((parse_mask, parse_mem)), newline)))(input)
}

struct State {
    // Internally we store the mask as two u64s for ease of computation later on
    mask: (u64, u64),
    regs: HashMap<u64, u64>,
}

impl State {
    fn new() -> Self {
        Self {
            mask: (0, 0),
            regs: HashMap::new(),
        }
    }

    fn execute(&mut self, inst: &Instruction) {
        match inst {
            Instruction::UpdateMask(mask) => {
                self.mask = mask
                    .iter()
                    .rev()
                    .enumerate()
                    .fold((0, 0), |(set, clear), (i, bit)| match bit {
                        MaskBit::Set => (set | 1 << i, clear),
                        MaskBit::Clear => (set, clear | 1 << i),
                        _ => (set, clear),
                    });
            }
            Instruction::WriteValue(addr, val) => {
                self.regs.insert(*addr, (val | self.mask.0) & !self.mask.1);
            }
        }
    }
}

fn exec_prog(prog: &Vec<Instruction>) -> u64 {
    let mut state = State::new();
    for inst in prog {
        state.execute(inst);
    }
    state.regs.values().sum()
}

fn main() {
    let fname = env::args()
        .skip(1)
        .next()
        .expect("Please provide path to input file!");
    let prog = parse_program(&fs::read_to_string(fname).unwrap())
        .unwrap()
        .1;
    println!("{}", exec_prog(&prog));
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parse_program() {
        let input = concat!(
            "mask = XXXXXXXXXXXXXXXXXXXXXXXXXXXXX1XXXX0X\n",
            "mem[8] = 11\n",
            "mem[7] = 101\n",
            "mem[8] = 0\n",
        );
        let mut exp_mask = [MaskBit::Ignore; 36];
        exp_mask[29] = MaskBit::Set;
        exp_mask[34] = MaskBit::Clear;
        assert_eq!(
            parse_program(input),
            Ok((
                "",
                vec!(
                    Instruction::UpdateMask(exp_mask),
                    Instruction::WriteValue(8, 11),
                    Instruction::WriteValue(7, 101),
                    Instruction::WriteValue(8, 0),
                )
            ))
        );
    }

    #[test]
    fn test_state() {
        let mut state = State::new();
        state.execute(&Instruction::WriteValue(12, 69));
        assert_eq!(state.regs[&12], 69);

        let mut mask = [MaskBit::Ignore; 36];
        mask[34] = MaskBit::Clear;
        mask[29] = MaskBit::Set;
        state.execute(&Instruction::UpdateMask(mask));
        assert_eq!(state.mask.0, 1 << 6);
        assert_eq!(state.mask.1, 1 << 1);

        state.execute(&Instruction::WriteValue(8, 11));
        assert_eq!(state.regs[&8], 73);
        state.execute(&Instruction::WriteValue(7, 101));
        assert_eq!(state.regs[&7], 101);
        state.execute(&Instruction::WriteValue(8, 0));
        assert_eq!(state.regs[&8], 64);
    }

    #[test]
    fn test_exec_prog() {
        let input = concat!(
            "mask = XXXXXXXXXXXXXXXXXXXXXXXXXXXXX1XXXX0X\n",
            "mem[8] = 11\n",
            "mem[7] = 101\n",
            "mem[8] = 0\n",
        );

        let prog = parse_program(input).unwrap().1;
        assert_eq!(exec_prog(&prog), 165);
    }
}
