use nom::branch::*;
use nom::bytes::complete::*;
use nom::character::complete::*;
use nom::combinator::*;
use nom::multi::*;
use nom::sequence::*;

use super::{Hash, UncheckedLeap, UncheckedList};
use crate::date::*;

type Result<'a, O> =
    nom::IResult<&'a str, O, nom::error::VerboseError<&'a str>>;

fn decimal<T: std::str::FromStr>(input: &str) -> Result<T> {
    map_res(digit1, T::from_str)(input)
}

fn hexword(input: &str) -> Result<u32> {
    preceded(space1, map_res(hex_digit1, |s| u32::from_str_radix(s, 16)))(input)
}

fn month(input: &str) -> Result<i32> {
    alt((
        value(1, tag("Jan")),
        value(2, tag("Feb")),
        value(3, tag("Mar")),
        value(4, tag("Apr")),
        value(5, tag("May")),
        value(6, tag("Jun")),
        value(7, tag("Jul")),
        value(8, tag("Aug")),
        value(9, tag("Sep")),
        value(10, tag("Oct")),
        value(11, tag("Nov")),
        value(12, tag("Dec")),
    ))(input)
}

fn date(input: &str) -> Result<Gregorian> {
    map(
        tuple((
            preceded(space1, decimal),
            preceded(space1, month),
            preceded(space1, decimal),
        )),
        |(d, m, y)| Gregorian(y, m, d),
    )(input)
}

fn empty(input: &str) -> Result<()> {
    value((), pair(tag("#"), line_ending))(input)
}

fn comment(input: &str) -> Result<()> {
    value((), tuple((tag("#"), space1, not_line_ending, line_ending)))(input)
}

fn ignore(input: &str) -> Result<()> {
    value((), many0_count(alt((empty, comment))))(input)
}

fn updated(input: &str) -> Result<i64> {
    delimited(pair(tag("#$"), space1), decimal, line_ending)(input)
}

fn expires(input: &str) -> Result<i64> {
    delimited(pair(tag("#@"), space1), decimal, line_ending)(input)
}

fn leapsecs(input: &str) -> Result<Vec<UncheckedLeap>> {
    many1(tuple((
        terminated(decimal, space1),
        terminated(decimal, space1),
        delimited(tag("#"), date, line_ending),
    )))(input)
}

fn hash(input: &str) -> Result<Hash> {
    let mut hash: Hash = Default::default();
    let (rest, ()) =
        delimited(tag("#h"), fill(hexword, &mut hash.0), line_ending)(input)?;
    Ok((rest, hash))
}

pub(super) fn parse(input: &str) -> Result<UncheckedList> {
    map(
        tuple((
            preceded(ignore, updated),
            preceded(ignore, expires),
            preceded(ignore, leapsecs),
            preceded(ignore, hash),
        )),
        |(updated, expires, leapsecs, hash)| UncheckedList {
            updated,
            expires,
            leapsecs,
            hash,
        },
    )(input)
}
