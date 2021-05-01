use crate::date::*;
use nom::branch::*;
use nom::bytes::complete::*;
use nom::character::complete::*;
use nom::combinator::*;
use nom::multi::*;
use nom::sequence::*;
use std::str::FromStr;

type UncheckedLeap = (u64, u64, Gregorian);

#[derive(Clone, Debug, Default)]
pub struct UncheckedNIST {
    pub updated: u64,
    pub expires: u64,
    pub leapsecs: Vec<UncheckedLeap>,
    pub hashval: [u32; 5],
}

type Result<'a, O> = nom::IResult<&'a str, O, nom::error::Error<&'a str>>;

fn dec64<'a>(input: &'a str) -> Result<'a, u64> {
    map_res(digit1, |s| u64::from_str(s))(input)
}

fn hex32<'a>(input: &'a str) -> Result<'a, u32> {
    map_res(hex_digit1, |s| u32::from_str_radix(s, 16))(input)
}

fn month<'a>(input: &'a str) -> Result<'a, i32> {
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

fn date<'a>(input: &'a str) -> Result<'a, Gregorian> {
    map(
        tuple((
            preceded(space1, dec64),
            preceded(space1, month),
            preceded(space1, dec64),
        )),
        |(d, m, y)| Gregorian(y as i32, m, d as i32),
    )(input)
}

fn empty<'a>(input: &'a str) -> Result<'a, ()> {
    value((), pair(tag("#"), line_ending))(input)
}

fn comment<'a>(input: &'a str) -> Result<'a, ()> {
    value((), tuple((tag("#"), space1, not_line_ending, line_ending)))(input)
}

fn ignore<'a>(input: &'a str) -> Result<'a, ()> {
    value((), many0_count(alt((empty, comment))))(input)
}

fn updated<'a>(input: &'a str) -> Result<'a, u64> {
    delimited(pair(tag("#$"), space1), dec64, line_ending)(input)
}

fn expires<'a>(input: &'a str) -> Result<'a, u64> {
    delimited(pair(tag("#@"), space1), dec64, line_ending)(input)
}

fn leapsecs<'a>(input: &'a str) -> Result<'a, Vec<UncheckedLeap>> {
    many1(tuple((
        terminated(dec64, space1),
        terminated(dec64, space1),
        delimited(tag("#"), date, line_ending),
    )))(input)
}

fn hashval<'a>(input: &'a str) -> Result<'a, [u32; 5]> {
    let mut hashval: [u32; 5] = Default::default();
    let (rest, ()) = delimited(
        tag("#h"),
        fill(preceded(space1, hex32), &mut hashval),
        line_ending,
    )(input)?;
    Ok((rest, hashval))
}

pub fn parse<'a>(input: &'a str) -> Result<'a, UncheckedNIST> {
    map(
        tuple((
            preceded(ignore, updated),
            preceded(ignore, expires),
            preceded(ignore, leapsecs),
            preceded(ignore, hashval),
        )),
        |(updated, expires, leapsecs, hashval): (
            u64,
            u64,
            Vec<UncheckedLeap>,
            [u32; 5],
        )| UncheckedNIST { updated, expires, leapsecs, hashval },
    )(input)
}