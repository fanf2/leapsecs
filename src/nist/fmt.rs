use ring::digest::*;
use std::convert::TryInto;
use std::fmt::Write;

use super::{Error, Hash};
use crate::date::*;
use crate::leap::*;

pub fn format(list: &LeapSecs, updated: i32) -> Result<String, Error> {
    let mut out = String::new();
    let updated = mjd2ntp(updated);
    let expires = mjd2ntp(list.last().unwrap().mjd());
    write!(out, "#$\t{}\n", updated)?;
    write!(out, "#@\t{}\n", expires)?;
    for leap in list.iter().take(list.len() - 1) {
        write!(out, "{}\t{}\t", mjd2ntp(leap.mjd()), leap.dtai())?;
        let date = Gregorian::from(leap.mjd());
        let month = [
            "Jan", "Feb", "Mar", "Apr", "May", "Jun", "Jul", "Aug", "Sep",
            "Oct", "Nov", "Dec",
        ][(date.month() - 1) as usize];
        write!(out, "# {} {} {}\n", date.day(), month, date.year())?;
    }
    let hash = sha1(&hashin(list, updated)?);
    write!(
        out,
        "#h\t{:08x} {:08x} {:08x} {:08x} {:08x}\n",
        hash[0], hash[1], hash[2], hash[3], hash[4],
    )?;
    Ok(out)
}

fn hashin(list: &LeapSecs, updated: i64) -> Result<String, Error> {
    let expires = mjd2ntp(list.last().unwrap().mjd());
    let mut hashin = String::new();
    write!(hashin, "{}{}", updated, expires)?;
    for leap in list.iter().take(list.len() - 1) {
        write!(hashin, "{}{}", mjd2ntp(leap.mjd()), leap.dtai())?;
    }
    Ok(hashin)
}

pub(super) fn checksum(
    list: LeapSecs,
    updated: i32,
    input: Hash,
) -> Result<LeapSecs, Error> {
    let updated = mjd2ntp(updated);
    let hashin = hashin(&list, updated)?;
    let output = sha1(&hashin);
    if input != output {
        Err(Error::Checksum(input, output, hashin))
    } else {
        Ok(list)
    }
}

fn sha1(input: &str) -> Hash {
    let hash = digest(&SHA1_FOR_LEGACY_USE_ONLY, input.as_bytes());
    // panic if sha1 is not the standard size
    let hash8: [u8; 20] = hash.as_ref().try_into().unwrap();
    let mut hash32: Hash = Default::default();
    for i in 0..5 {
        let word: [u8; 4] = hash8[i * 4..i * 4 + 4].try_into().unwrap();
        hash32[i] = u32::from_be_bytes(word);
    }
    hash32
}
