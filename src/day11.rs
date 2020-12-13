use nom::character::complete::{newline, one_of};
use nom::combinator::{all_consuming, map, opt};
use nom::multi::{many1, separated_list1};
use nom::sequence::terminated;
use nom::IResult;
use std::convert::{From, TryFrom, TryInto};
use std::env;
use std::fs;

#[derive(PartialEq, Eq, Debug, Clone, Copy)]
enum SeatState {
    Occupied,
    Empty,
    Floor,
}

impl TryFrom<char> for SeatState {
    type Error = String;

    fn try_from(c: char) -> Result<SeatState, String> {
        match c {
            'L' => Ok(SeatState::Empty),
            '#' => Ok(SeatState::Occupied),
            '.' => Ok(SeatState::Floor),
            _ => Err(format!("Unknown seat state '{}'", c)),
        }
    }
}

#[derive(PartialEq, Eq, Debug)]
struct Grid(Vec<Vec<SeatState>>);

impl From<Vec<Vec<SeatState>>> for Grid {
    fn from(vec: Vec<Vec<SeatState>>) -> Grid {
        return Grid(vec);
    }
}

impl Grid {
    fn seats_to_check_part1(&self, x: usize, y: usize) -> Vec<(usize, usize)> {
        // Build a list of seats to check for the given x, y.
        // In the simple case this will be 8 coords in the range x/y +- 1 excluding
        // (x, y) itself, however we also need to handle coords in the first/last col/
        // row, which have fewer than 8 neighbors.
        let mut to_check = vec![];

        // Check for row above
        if y > 0 {
            if x > 0 {
                to_check.push((x - 1, y - 1))
            };
            to_check.push((x, y - 1));
            if x < self.0[y - 1].len() - 1 {
                to_check.push((x + 1, y - 1))
            };
        }
        // Same row
        if x > 0 {
            to_check.push((x - 1, y))
        };
        if x < self.0[y].len() - 1 {
            to_check.push((x + 1, y))
        };
        // Row below
        if y < self.0.len() - 1 {
            if x > 0 {
                to_check.push((x - 1, y + 1))
            };
            to_check.push((x, y + 1));
            if x < self.0[y + 1].len() - 1 {
                to_check.push((x + 1, y + 1))
            };
        }
        to_check
    }

    fn seats_to_check_part2(&self, x: usize, y: usize) -> Vec<(usize, usize)> {
        // Check each of the 8 directions from seat (x, y) for the next seat.
        [
            (-1, -1),
            (0, -1),
            (1, -1),
            (-1, 0),
            (1, 0),
            (-1, 1),
            (0, 1),
            (1, 1),
        ]
        .iter()
        .map(|(a, b)| {
            let mut seat = (x as i16, y as i16);
            return loop {
                seat.0 += a;
                seat.1 += b;
                if seat.0 < 0 || seat.1 < 0 {
                    // We've gone past the top/left, so there's no seat in this dir.
                    break None;
                }
                match self.get(seat.0 as usize, seat.1 as usize) {
                    Some(SeatState::Occupied) | Some(SeatState::Empty) => {
                        break Some((seat.0 as usize, seat.1 as usize))
                    }
                    Some(SeatState::Floor) => {}
                    None => break None,
                }
            };
        })
        .filter_map(|s| s)
        .collect()
    }

    fn count_occupied_neighbors(&self, x: usize, y: usize, part2: bool) -> u8 {
        let to_check = if part2 {
            self.seats_to_check_part2(x, y)
        } else {
            self.seats_to_check_part1(x, y)
        };
        to_check
            .iter()
            .map(|(x, y)| self.get(*x, *y))
            .filter(|state| *state == Some(SeatState::Occupied))
            .count() as u8
    }
    fn get(&self, x: usize, y: usize) -> Option<SeatState> {
        if let Some(row) = self.0.get(y) {
            if let Some(state) = row.get(x) {
                return Some(*state);
            }
        }
        None
    }

    fn generate(&self, part2: bool) -> Grid {
        self.0
            .iter()
            .enumerate()
            .map(|(y, row)| {
                return row
                    .iter()
                    .enumerate()
                    .map(|(x, c)| {
                        let occupied_count = self.count_occupied_neighbors(x, y, part2);
                        return match c {
                            SeatState::Floor => SeatState::Floor, // Floor never changes
                            SeatState::Empty => {
                                if occupied_count == 0 {
                                    SeatState::Occupied
                                } else {
                                    SeatState::Empty
                                }
                            }
                            SeatState::Occupied => {
                                if occupied_count >= if part2 {5} else {4} {
                                    SeatState::Empty
                                } else {
                                    SeatState::Occupied
                                }
                            }
                        };
                    })
                    .collect();
            })
            .collect::<Vec<Vec<SeatState>>>()
            .into()
    }
}

fn parse_grid(input: &str) -> IResult<&str, Grid> {
    map(
        all_consuming(terminated(
            separated_list1(
                newline,
                many1(map(one_of(".L#"), |c| c.try_into().unwrap())),
            ),
            opt(newline),
        )),
        From::from, // convert the Vec<Vec<_>> to a Grid
    )(input)
}

fn main() {
    let fname = env::args()
        .skip(1)
        .next()
        .expect("Please provide path to input data!");
    let input = fs::read_to_string(fname).unwrap();
    let mut last = parse_grid(&input).unwrap().1;
    let mut gen_count = 1;
    loop {
        let next = last.generate(true); // part2
        if next == last {
            println!("Grid stable at generation {}!", gen_count);
            let occupied_count = next
                .0
                .iter()
                .flatten()
                .filter(|s| **s == SeatState::Occupied)
                .count();
            println!("Occupied count: {}", occupied_count);
            return;
        }
        last = next;
        gen_count += 1;
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_generate_part1() {
        let gen1 = parse_grid(
            "L.LL.LL.LL
LLLLLLL.LL
L.L.L..L..
LLLL.LL.LL
L.LL.LL.LL
L.LLLLL.LL
..L.L.....
LLLLLLLLLL
L.LLLLLL.L
L.LLLLL.LL",
        )
        .unwrap()
        .1;

        let gen2 = parse_grid(
            "#.##.##.##
#######.##
#.#.#..#..
####.##.##
#.##.##.##
#.#####.##
..#.#.....
##########
#.######.#
#.#####.##",
        )
        .unwrap()
        .1;

        let gen3 = parse_grid(
            "#.LL.L#.##
#LLLLLL.L#
L.L.L..L..
#LLL.LL.L#
#.LL.LL.LL
#.LLLL#.##
..L.L.....
#LLLLLLLL#
#.LLLLLL.L
#.#LLLL.##",
        )
        .unwrap()
        .1;

        assert_eq!(gen1.generate(false), gen2);
        assert_eq!(gen2.generate(false), gen3);
    }
    #[test]
    fn test_count_occupied_part1() {
        let grid = parse_grid(
            "#.##.##.##
#######.##
#.#.#..#..
####.##.##
#.##.##.##
#.#####.##
..#.#.....
##########
#.######.#
#.#####.##",
        )
        .unwrap()
        .1;

        assert_eq!(grid.count_occupied_neighbors(0, 0, false), 2);
        assert_eq!(grid.count_occupied_neighbors(1, 0, false), 5);
        assert_eq!(grid.count_occupied_neighbors(2, 1, false), 5);
        assert_eq!(grid.count_occupied_neighbors(4, 8, false), 8);
    }

    #[test]
    fn test_count_occupied_part_2() {
        let grid = parse_grid(".......#.
...#.....
.#.......
.........
..#L....#
....#....
.........
#........
...#.....").unwrap().1;

        assert_eq!(grid.count_occupied_neighbors(3, 4, true), 8);
        assert_eq!(grid.count_occupied_neighbors(0, 1, true), 3);
    }

    #[test]
    fn test_grid_get() {
        let grid = parse_grid(
            "#.LL.L#.##
#LLLLLL.L#
L.L.L..L..
#LLL.LL.L#
#.LL.LL.LL
#.LLLL#.##
..L.L.....
#LLLLLLLL#
#.LLLLLL.L
#.#LLLL.##",
        )
        .unwrap()
        .1;

        assert_eq!(grid.get(0, 0), Some(SeatState::Occupied));
        assert_eq!(grid.get(10, 0), None);
        assert_eq!(grid.get(9, 8), Some(SeatState::Empty));
    }
}
