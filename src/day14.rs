#![feature(destructuring_assignment)]
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
    One,
    Zero,
    X,
}

impl TryFrom<char> for MaskBit {
    type Error = &'static str;
    fn try_from(c: char) -> Result<MaskBit, &'static str> {
        match c {
            '1' => Ok(MaskBit::One),
            '0' => Ok(MaskBit::Zero),
            'X' => Ok(MaskBit::X),
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
    mask_one: u64,
    mask_zero: u64,
    mask_x: u64,
    regs: HashMap<u64, u64>,
    part2: bool,
}

impl State {
    fn new(part2: bool) -> Self {
        Self {
            mask_one: 0,
            mask_zero: 0,
            mask_x: 0,
            regs: HashMap::new(),
            part2: part2,
        }
    }

    fn execute(&mut self, inst: &Instruction) {
        match inst {
            Instruction::UpdateMask(mask) => {
                (self.mask_one, self.mask_zero, self.mask_x) =
                    mask.iter()
                        .rev()
                        .enumerate()
                        .fold((0, 0, 0), |(one, zero, x), (i, bit)| match bit {
                            MaskBit::One => (one | 1 << i, zero, x),
                            MaskBit::Zero => (one, zero | 1 << i, x),
                            MaskBit::X => (one, zero, x | 1 << i),
                        });
            }
            Instruction::WriteValue(addr, val) => {
                if self.part2 {
                    // mask mutates addr. The "ones" set bits, "zeros" are ignored,
                    // and "x"s cause permutation.
                    for mutated_addr in permutate(addr | self.mask_one, self.mask_x) {
                        self.regs.insert(mutated_addr, *val);
                    }
                } else {
                    // mask mutates value. The "ones" set bits, the "zeroes" clear them
                    self.regs
                        .insert(*addr, (val | self.mask_one) & !self.mask_zero);
                }
            }
        }
    }
}

fn exec_prog(prog: &Vec<Instruction>, part2: bool) -> u64 {
    let mut state = State::new(part2);
    for inst in prog {
        state.execute(inst);
    }
    state.regs.values().sum()
}

fn permutate(value: u64, permute_mask: u64) -> Vec<u64> {
    let mut to_do = vec![(value, permute_mask)];
    let mut done = vec![];
    'outer: while let Some((value, mask)) = to_do.pop() {
        for i in 0..64 {
            if ((1 << i) & mask) > 0 {
                // if the mask bit is set, permute this bit
                to_do.push((value | (1 << i), mask & !(1 << i))); // bit set
                to_do.push((value & !(1 << i), mask & !(1 << i))); // bit cleared
                continue 'outer;
            }
        }
        done.push(value); // we didn't find any mask bits so this is a fully permutated value
    }
    done.sort();
    done
}

fn main() {
    let fname = env::args()
        .skip(1)
        .next()
        .expect("Please provide path to input file!");
    let prog = parse_program(&fs::read_to_string(fname).unwrap())
        .unwrap()
        .1;
    println!("part 1: {}", exec_prog(&prog, false)); // Part1
    println!("part 2: {}", exec_prog(&prog, true)); // Part2
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
        let mut exp_mask = [MaskBit::X; 36];
        exp_mask[29] = MaskBit::One;
        exp_mask[34] = MaskBit::Zero;
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
    fn test_state_part1() {
        let mut state = State::new(false);
        state.execute(&Instruction::WriteValue(12, 69));
        assert_eq!(state.regs[&12], 69);

        let mut mask = [MaskBit::X; 36];
        mask[34] = MaskBit::Zero;
        mask[29] = MaskBit::One;
        state.execute(&Instruction::UpdateMask(mask));
        assert_eq!(state.mask_one, 1 << 6);
        assert_eq!(state.mask_zero, 1 << 1);

        state.execute(&Instruction::WriteValue(8, 11));
        assert_eq!(state.regs[&8], 73);
        state.execute(&Instruction::WriteValue(7, 101));
        assert_eq!(state.regs[&7], 101);
        state.execute(&Instruction::WriteValue(8, 0));
        assert_eq!(state.regs[&8], 64);
    }

    #[test]
    fn test_exec_prog_part1() {
        let input = concat!(
            "mask = XXXXXXXXXXXXXXXXXXXXXXXXXXXXX1XXXX0X\n",
            "mem[8] = 11\n",
            "mem[7] = 101\n",
            "mem[8] = 0\n",
        );

        let prog = parse_program(input).unwrap().1;
        assert_eq!(exec_prog(&prog, false), 165);
    }

    #[test]
    fn test_permutate() {
        assert_eq!(permutate(0b0100, 0b0), vec!(0b0100));
        assert_eq!(permutate(0b0100, 0b1), vec!(0b0100, 0b0101));
        assert_eq!(
            permutate(0b0010, 0b1010),
            vec!(0b0000, 0b0010, 0b1000, 0b1010)
        );
        assert_eq!(
            permutate(0b1111, 0b1011),
            vec!(0b0100, 0b0101, 0b0110, 0b0111, 0b1100, 0b1101, 0b1110, 0b1111),
        );
    }

    #[test]
    fn test_exec_prog_part2() {
        let input = concat!(
            "mask = 000000000000000000000000000000X1001X\n",
            "mem[42] = 100\n",
            "mask = 00000000000000000000000000000000X0XX\n",
            "mem[26] = 1\n",
        );

        let prog = parse_program(input).unwrap().1;
        assert_eq!(exec_prog(&prog, true), 208);
    }
}
