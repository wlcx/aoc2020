use std::collections::HashMap;
use std::convert::TryFrom;
use std::env;
use std::fs::File;
use std::io::{BufRead, BufReader};
use std::iter::FromIterator;

#[derive(Copy, Clone, Debug, PartialEq)]
enum CubeState {
    Active,
    Inactive,
}

impl TryFrom<char> for CubeState {
    type Error = &'static str;
    fn try_from(c: char) -> Result<CubeState, &'static str> {
        match c {
            '.' => Ok(CubeState::Inactive),
            '#' => Ok(CubeState::Active),
            _ => Err("No"),
        }
    }
}

type Dimension = HashMap<i32, HashMap<i32, HashMap<i32, HashMap<i32, CubeState>>>>;

fn parse_input<T>(r: T) -> Dimension
where
    T: BufRead,
{
    // Pretty gross huh? This parses our 2d input and returns a 4d Dimension.
    let mut m = HashMap::new();
    m.insert(0, {
        let mut m = HashMap::new();
        m.insert(
            0 as i32,
            HashMap::from_iter(
                r.lines()
                    .map(Result::unwrap)
                    .map(|l| {
                        HashMap::from_iter(
                            l.chars()
                                .map(CubeState::try_from)
                                .map(Result::unwrap)
                                .enumerate()
                                .map(|(i, s)| (i as i32, s)),
                        )
                    })
                    .enumerate()
                    .map(|(i, s)| (i as i32, s)),
            ),
        );
        m
    });
    m
}

fn initmap<T>(min: i32, max: i32, val: T) -> HashMap<i32, T>
where
    T: Clone,
{
    let mut m = HashMap::new();
    for i in min..=max {
        m.insert(i, val.clone());
    }
    m
}

// find the bounds of the dimension, adding an additional "shell" if there are active cubes in
// the outer shell.
fn expand(d: &mut Dimension) {
    // transform the dimension into a vec of (x, y, z, state), and filter for active cubes only
    let cells = d
        .iter()
        .map(|(w, e)| {
            e.iter().map(move |(z, e)| {
                e.iter()
                    .map(move |(y, f)| f.iter().map(move |(x, state)| (*x, *y, *z, *w, state)))
            })
        })
        .flatten()
        .flatten()
        .flatten()
        .filter(|(_, _, _, _, state)| **state == CubeState::Active)
        .collect::<Vec<(_, _, _, _, _)>>();

    // this relies on all hashmaps being equal length!
    let max_w = *d.keys().max().unwrap();
    let min_w = *d.keys().min().unwrap();

    let max_z = *d[&0].keys().max().unwrap();
    let min_z = *d[&0].keys().min().unwrap();

    let max_y = *d[&0][&0].keys().max().unwrap();
    let min_y = *d[&0][&0].keys().min().unwrap();

    let max_x = *d[&0][&0][&0].keys().max().unwrap();
    let min_x = *d[&0][&0][&0].keys().min().unwrap();

    let active_max_x = cells.iter().max_by_key(|(x, _, _, _, _)| x).unwrap().0;
    let active_min_x = cells.iter().min_by_key(|(x, _, _, _, _)| x).unwrap().0;

    let active_max_y = cells.iter().max_by_key(|(_, y, _, _, _)| y).unwrap().1;
    let active_min_y = cells.iter().min_by_key(|(_, y, _, _, _)| y).unwrap().1;

    let active_max_z = cells.iter().max_by_key(|(_, _, z, _, _)| z).unwrap().2;
    let active_min_z = cells.iter().min_by_key(|(_, _, z, _, _)| z).unwrap().2;

    let active_max_w = cells.iter().max_by_key(|(_, _, _, w, _)| w).unwrap().3;
    let active_min_w = cells.iter().min_by_key(|(_, _, _, w, _)| w).unwrap().3;

    if max_w == active_max_w || min_w == active_min_w {
        // expand a
        println!("expanding a (old min/max: {}/{})", min_w, max_w);
        d.insert(
            max_w + 1,
            initmap(
                min_z,
                max_z,
                initmap(min_y, max_y, initmap(min_x, max_x, CubeState::Inactive)),
            ),
        );
        d.insert(
            min_w - 1,
            initmap(
                min_z,
                max_z,
                initmap(min_y, max_y, initmap(min_x, max_x, CubeState::Inactive)),
            ),
        );
    }

    if max_z == active_max_z || min_z == active_min_z {
        // expand z
        println!("expanding z (old min/max: {}/{})", min_z, max_z);
        for zyx in d.iter_mut() {
            zyx.1.insert(
                max_z + 1,
                initmap(min_y, max_y, initmap(min_x, max_x, CubeState::Inactive)),
            );
            zyx.1.insert(
                min_z - 1,
                initmap(min_y, max_y, initmap(min_x, max_x, CubeState::Inactive)),
            );
        }
    }

    if max_y == active_max_y || min_y == active_min_y {
        // expand y
        println!("expanding y (old min/max: {}/{})", min_y, max_y);
        for zyx in d.iter_mut() {
            for yx in zyx.1 {
                yx.1.insert(max_y + 1, initmap(min_x, max_x, CubeState::Inactive));
                yx.1.insert(min_y - 1, initmap(min_x, max_x, CubeState::Inactive));
            }
        }
    }

    if max_x == active_max_x || min_x == active_min_x {
        // expand x
        println!("expanding x (old min/max: {}/{})", min_x, max_x);
        for zyx in d.iter_mut() {
            for yx in zyx.1 {
                for x in yx.1 {
                    x.1.insert(max_x + 1, CubeState::Inactive);
                    x.1.insert(min_x - 1, CubeState::Inactive);
                }
            }
        }
    }
}

// Count how many neighbors are active
fn count_neighbors(d: &Dimension, p: (i32, i32, i32, i32)) -> usize {
    (p.0 - 1..=p.0 + 1)
        .map(|x| {
            (p.1 - 1..=p.1 + 1).map(move |y| {
                (p.2 - 1..=p.2 + 1).map(move |z| (p.3 - 1..=p.3 + 1).map(move |a| (x, y, z, a)))
            })
        })
        .flatten()
        .flatten()
        .flatten()
        .filter_map(|(x, y, z, a)| {
            if let Some(zxys) = d.get(&a) {
                if let Some(xys) = zxys.get(&z) {
                    if let Some(xs) = xys.get(&y) {
                        if let Some(state) = xs.get(&x) {
                            if *state == CubeState::Active && (x, y, z, a) != p {
                                return Some(());
                            }
                        }
                    }
                }
            }
            None
        })
        .count()
}

fn generate(d: &Dimension) -> Dimension {
    HashMap::from_iter(d.iter().map(|(a, e)| {
        (
            *a,
            HashMap::from_iter(e.iter().map(|(z, f)| {
                (
                    *z,
                    HashMap::from_iter(f.iter().map(move |(y, g)| {
                        (
                            *y,
                            HashMap::from_iter(g.iter().map(move |(x, state)| {
                                let neighbors = count_neighbors(d, (*x, *y, *z, *a));
                                (
                                    *x,
                                    match state {
                                        CubeState::Active => {
                                            if neighbors == 2 || neighbors == 3 {
                                                CubeState::Active
                                            } else {
                                                CubeState::Inactive
                                            }
                                        }
                                        CubeState::Inactive => {
                                            if neighbors == 3 {
                                                CubeState::Active
                                            } else {
                                                CubeState::Inactive
                                            }
                                        }
                                    },
                                )
                            })),
                        )
                    })),
                )
            })),
        )
    }))
}

fn main() {
    let fname = env::args()
        .skip(1)
        .next()
        .expect("Please provide path to input file!");
    let mut input = parse_input(BufReader::new(File::open(fname).unwrap()));
    for _ in 0..6 {
        expand(&mut input);
        input = generate(&input);
    }
    let active_count = input
        .iter()
        .map(|(_, maps)| maps)
        .flatten()
        .map(|(_, maps)| maps)
        .flatten()
        .map(|(_, maps)| maps)
        .flatten()
        .filter(|(_, state)| **state == CubeState::Active)
        .count();
    println!("Active: {}", active_count);
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parse_input() {
        let input = parse_input(
            ".#.
..#
###"
            .as_bytes(),
        );
        let mut expected = vec![HashMap::new(), HashMap::new(), HashMap::new()];
        expected[0].insert(0, CubeState::Inactive);
        expected[0].insert(1, CubeState::Active);
        expected[0].insert(2, CubeState::Inactive);
        expected[1].insert(0, CubeState::Inactive);
        expected[1].insert(1, CubeState::Inactive);
        expected[1].insert(2, CubeState::Active);
        expected[2].insert(0, CubeState::Active);
        expected[2].insert(1, CubeState::Active);
        expected[2].insert(2, CubeState::Active);
        assert_eq!(input, HashMap::from_iter(expected.into_iter().enumerate()));
    }
}
