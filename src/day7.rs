use nom::branch::alt;
use nom::bytes::complete::tag;
use nom::character::complete::{alpha1, digit1, line_ending, space1};
use nom::combinator::{all_consuming, opt, recognize};
use nom::multi::{separated_list0, separated_list1};
use nom::sequence::{separated_pair, terminated, tuple};
use nom::IResult;
use std::convert::TryInto;
use std::env;
use std::fs::File;
use std::io::{prelude::*, BufReader};

type BaggageRule = (&'static str, Vec<(u8, &'static str)>);
type BaggageRules = Vec<BaggageRule>;

fn bag_specifier(input: &str) -> IResult<&str, &str> {
    terminated(
        recognize(separated_pair(alpha1, space1, alpha1)), // two alphabetic words, separated by a space..
        tuple((tag(" bag"), opt(tag("s")))),               // ..terminated by " bag(s)"
    )(input)
}

fn baggage_rule(input: &'static str) -> IResult<&str, BaggageRule> {
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

fn parse_baggage_rules(input: &'static str) -> IResult<&str, BaggageRules> {
    all_consuming(separated_list1(line_ending, baggage_rule))(input)
}

fn main() {
    let fname = env::args().skip(1).next().expect("Please provide path to input file!");
    let rules = BufReader::new(File::open(fname).unwrap())
}

#[cfg(test)]
mod tests {
    use super::*;

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
        )
    }
}
