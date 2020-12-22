use nom::bytes::complete::tag;
use nom::character::complete::{alpha1, digit1, line_ending, space1};
use nom::combinator::{all_consuming, opt, recognize};
use nom::multi::{many1, separated_list0};
use nom::sequence::{separated_pair, terminated, tuple};
use nom::IResult;
use std::collections::{HashMap, HashSet};
use std::env;
use std::fs;
use std::iter::FromIterator;

type BaggageRule<'a> = (&'a str, Vec<(u8, &'a str)>);
type BaggageRules<'a> = Vec<BaggageRule<'a>>;

fn bag_specifier(input: &str) -> IResult<&str, &str> {
    terminated(
        recognize(separated_pair(alpha1, space1, alpha1)), // two alphabetic words, separated by a space..
        tuple((tag(" bag"), opt(tag("s")))),               // ..terminated by " bag(s)"
    )(input)
}

fn baggage_rule(input: &str) -> IResult<&str, BaggageRule> {
    match tuple((
        bag_specifier,
        tag(" contain "),
        separated_list0(tag(", "), separated_pair(digit1, space1, bag_specifier)),
        opt(tag("no other bags")),
        tag("."),
    ))(input)
    {
        Ok((
            remaining,
            (
                outer,    // The two words of the outer bag
                _,        // " contain "
                contents, // the vec of inner bags
                _,        // maybe "no other bags"
                _,        // ending full stop
            ),
        )) => Ok((
            remaining,
            (
                outer,
                contents
                    .iter()
                    .map(|(count, bag)| (str::parse::<u8>(*count).unwrap(), *bag))
                    .collect(),
            ),
        )),
        Err(e) => Err(e),
    }
}

fn parse_baggage_rules(input: &str) -> IResult<&str, BaggageRules> {
    all_consuming(many1(terminated(baggage_rule, line_ending)))(input)
}

// Given a list of BaggageRules, return a map from bag type to allowed outer bags
fn outer_bag_map<'a>(rules: &'a BaggageRules) -> HashMap<&'a str, Vec<&'a str>> {
    rules
        .iter()
        .map(|(outer, inners)| inners.iter().map(move |inner| (inner, outer)))
        .flatten()
        .fold(HashMap::new(), |mut m, ((count, inner), outer)| {
            if *count > 0 {
                m.entry(inner).or_insert(vec![]).push(outer);
            }
            m
        })
}

fn get_all_outer<'a>(target: &'a str, outer_map: &'a HashMap<&str, Vec<&str>>) -> HashSet<&'a str> {
    let mut all_outers = HashSet::new();
    if let Some(outers) = outer_map.get(target) {
        all_outers.extend(outers);
        for outer in outers {
            all_outers.extend(get_all_outer(outer, outer_map));
        }
    }
    all_outers
}

fn get_total_contained<'a>(specifier: &str, rules: &'a HashMap<&str, Vec<(u8, &str)>>) -> u32 {
    rules
        .get(specifier)
        .unwrap()
        .iter()
        .map(|(count, specifier)| *count as u32 * (1 + get_total_contained(specifier, rules)))
        .sum()
}

fn main() {
    let fname = env::args()
        .skip(1)
        .next()
        .expect("Please provide path to input file!");
    let input = fs::read_to_string(fname).unwrap();
    let rules = parse_baggage_rules(&input).unwrap().1;
    let map = outer_bag_map(&rules);
    let outers = get_all_outer("shiny gold", &map);
    println!("{} possible outers", outers.len());

    println!(
        "Shiny gold contains {} total.",
        get_total_contained("shiny gold", &HashMap::from_iter(rules))
    );
}

#[cfg(test)]
mod tests {
    use super::*;

    const input: &'static str = "light red bags contain 1 bright white bag, 2 muted yellow bags.
dark orange bags contain 3 bright white bags, 4 muted yellow bags.
bright white bags contain 1 shiny gold bag.
muted yellow bags contain 2 shiny gold bags, 9 faded blue bags.
shiny gold bags contain 1 dark olive bag, 2 vibrant plum bags.
dark olive bags contain 3 faded blue bags, 4 dotted black bags.
vibrant plum bags contain 5 faded blue bags, 6 dotted black bags.
faded blue bags contain no other bags.
dotted black bags contain no other bags.
";

    #[test]
    fn test_bag_specifer() {
        assert_eq!(
            bag_specifier("esoteric beige bag"),
            Ok(("", "esoteric beige"))
        );
        assert_eq!(
            bag_specifier("disappointing maroon bags"),
            Ok(("", "disappointing maroon"))
        );
    }

    #[test]
    fn test_baggage_rule() {
        assert_eq!(
            baggage_rule("lol blue bags contain 1 ayy lmao bag, 2 wtf lol bags."),
            Ok(("", ("lol blue", vec!((1, "ayy lmao"), (2, "wtf lol"))))),
        );
        assert_eq!(
            baggage_rule("ignominious spruce bags contain no other bags."),
            Ok(("", ("ignominious spruce", vec!()))),
        )
    }

    #[test]
    fn test_outer_bag_map() {
        let parsed = parse_baggage_rules(&input).unwrap().1;
        let mut expected = HashMap::new();
        expected.insert("shiny gold", vec!["bright white", "muted yellow"]);
        expected.insert("bright white", vec!["light red", "dark orange"]);
        expected.insert("muted yellow", vec!["light red", "dark orange"]);
        expected.insert(
            "faded blue",
            vec!["muted yellow", "dark olive", "vibrant plum"],
        );
        expected.insert("dark olive", vec!["shiny gold"]);
        expected.insert("vibrant plum", vec!["shiny gold"]);
        expected.insert("dotted black", vec!["dark olive", "vibrant plum"]);
        assert_eq!(outer_bag_map(&parsed), expected);
    }

    #[test]
    fn test_get_all_outer() {
        let parsed = parse_baggage_rules(&input).unwrap().1;
        let map = outer_bag_map(&parsed);
        assert_eq!(
            get_all_outer("shiny gold", &map),
            ["bright white", "muted yellow", "dark orange", "light red"]
                .iter()
                .cloned()
                .collect(),
        );
    }

    #[test]
    fn test_get_total_contained() {
        let parsed = parse_baggage_rules(&input).unwrap().1;
        assert_eq!(
            get_total_contained("shiny gold", &HashMap::from_iter(parsed)),
            32
        );
    }
}
