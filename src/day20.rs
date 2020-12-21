use bitvec::prelude::*;
use nom::branch::alt;
use nom::bytes::complete::tag;
use nom::character::complete::{char, digit1, newline};
use nom::combinator::{all_consuming, map, map_res};
use nom::multi::many1;
use nom::sequence::{delimited, separated_pair, terminated};
use nom::IResult;
use std::collections::HashMap;
use std::convert::{TryFrom, TryInto};
use std::env;
use std::fs;

struct Tile {
    id: u16,
    data: [BitVec; 10],
}

impl Tile {
    fn edges(&self) -> [u16; 8] {
        [
            // going CW from the top...
            self.data[0].load(),
            self.data.iter().map(|bv| bv[9]).collect::<BitVec>().load(),
            self.data[9].iter().rev().collect::<BitVec>().load(),
            self.data
                .iter()
                .rev()
                .map(|bv| bv[0])
                .collect::<BitVec>()
                .load(),
            // And flipped...
            self.data[0].iter().rev().collect::<BitVec>().load(),
            self.data
                .iter()
                .rev()
                .map(|bv| bv[9])
                .collect::<BitVec>()
                .load(),
            self.data[9].load(),
            self.data.iter().map(|bv| bv[0]).collect::<BitVec>().load(),
        ]
    }
}

impl TryFrom<(u16, Vec<Vec<bool>>)> for Tile {
    type Error = &'static str;

    fn try_from(input: (u16, Vec<Vec<bool>>)) -> Result<Tile, &'static str> {
        Ok(Tile {
            id: input.0,
            data: input
                .1
                .iter()
                .map(|s| s.iter().collect::<BitVec>())
                .collect::<Vec<BitVec>>()
                .try_into()
                .unwrap(),
        })
    }
}

fn parse_input(input: &str) -> IResult<&str, Vec<Tile>> {
    all_consuming(many1(terminated(
        map_res(
            separated_pair(
                delimited(tag("Tile "), map_res(digit1, str::parse::<u16>), tag(":")),
                newline,
                many1(terminated(
                    many1(map(alt((char('#'), char('.'))), |c| match c {
                        '#' => true,
                        _ => false,
                    })),
                    newline,
                )),
            ),
            TryInto::try_into,
        ),
        newline,
    )))(input)
}

fn main() {
    let fname = env::args()
        .skip(1)
        .next()
        .expect("Please provide path to input file!");
    let data = parse_input(&fs::read_to_string(fname).unwrap()).unwrap().1;
    let mut edgecounts = HashMap::new();
    for tile in &data {
        for edge in tile.edges().iter() {
            *edgecounts.entry(*edge).or_insert(0) += 1;
        }
    }

    let unique_edges = edgecounts
        .iter()
        .filter(|(_, count)| **count == 1)
        .map(|(edge, _)| edge)
        .collect::<Vec<_>>();

    let corner_tiles = data
        .iter()
        .map(|t| {
            (
                t,
                t.edges()
                    .iter()
                    .map(|e| unique_edges.contains(&e))
                    .filter(|v| *v)
                    .count(),
            )
        })
        .filter(|(_, count)| *count == 4)
        .collect::<Vec<_>>();

    assert_eq!(corner_tiles.len(), 4);
    println!(
        "product: {}",
        corner_tiles
            .iter()
            .map(|(t, _)| t.id as u64)
            .product::<u64>()
    )
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parse() {
        let input = "Tile 2311:
..##.#..#.
##..#.....
#...##..#.
####.#...#
##.##.###.
##...#.###
.#.#.#..##
..#....#..
###...#.#.
..###..###

Tile 3061:
####..#.##
#.....##..
..........
......#...
..#.#..###
.#.#..#..#
.#...#...#
#........#
.....#.#..
#..#....##

";
        let tiles = parse_input(&input).unwrap().1;
        assert_eq!(tiles[0].id, 2311);
        assert_eq!(tiles[0].data[0], bitvec![0, 0, 1, 1, 0, 1, 0, 0, 1, 0]);
        assert_eq!(tiles[0].edges(), [300, 616, 231, 498, 210, 89, 924, 318]);
    }
}
