use super::{UncheckedLeap, UncheckedNIST};
use crate::date::*;

use nom::branch::*;
use nom::bytes::complete::*;
use nom::character::complete::*;
use nom::combinator::*;
use nom::multi::*;
use nom::sequence::*;
use std::str::FromStr;

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

fn hash<'a>(input: &'a str) -> Result<'a, [u8; 20]> {
    let mut hash32: [u32; 5] = Default::default();
    let (rest, ()) = delimited(
        tag("#h"),
        fill(preceded(space1, hex32), &mut hash32),
        line_ending,
    )(input)?;
    let mut hash8: [u8; 20] = Default::default();
    for word in 0..5 {
        for byte in 0..4 {
            let it = (hash32[word] << byte * 8) >> 24;
            hash8[word * 4 + byte] = it as u8;
        }
    }
    Ok((rest, hash8))
}

pub(super) fn parse<'a>(input: &'a str) -> Result<'a, UncheckedNIST> {
    map(
        tuple((
            preceded(ignore, updated),
            preceded(ignore, expires),
            preceded(ignore, leapsecs),
            preceded(ignore, hash),
        )),
        |(updated, expires, leapsecs, hash)| UncheckedNIST {
            updated,
            expires,
            leapsecs,
            hash,
        },
    )(input)
}
