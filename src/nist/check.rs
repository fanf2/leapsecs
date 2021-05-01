use super::parse::*;
use crate::date::*;
use ring::digest::*;
use std::convert::TryInto;
use std::fmt::Write;
use thiserror::Error;

#[derive(Debug, Eq, PartialEq)]
pub struct TimeStamp {
    ntp: u64,
    mjd: i32,
    date: Gregorian,
}

impl From<u64> for TimeStamp {
    fn from(ntp: u64) -> TimeStamp {
        let epoch = i32::from(Gregorian(1900, 1, 1));
        let mjd = (ntp / 86400) as i32 + epoch;
        let date = Gregorian::from(mjd);
        TimeStamp { ntp, mjd, date }
    }
}

impl From<Gregorian> for TimeStamp {
    fn from(date: Gregorian) -> TimeStamp {
        let epoch = i32::from(Gregorian(1900, 1, 1));
        let mjd = i32::from(date);
        let ntp = (mjd - epoch) as u64 * 86400;
        TimeStamp { ntp, mjd, date }
    }
}

#[derive(Error, Debug)]
pub enum NISTerror {
    #[error("leap seconds file has expired ({0:?})")]
    Expired(TimeStamp),
    #[error("timestamp is before 1972 ({0:?})")]
    TooOld(TimeStamp),
    #[error("timestamp is not midnight ({0:?})")]
    Fractional(TimeStamp),
    #[error("DTAI is too large ({0:?})")]
    Spinny(u64),
    #[error("timestamp and date do not match ({0:?} <> {1:?})")]
    Mismatch(TimeStamp, TimeStamp),
    #[error("checksum failed {0:?} <> {1:?} data {2}")]
    Checksum([u8; 20], [u8; 20], String),
    #[error("format error {0}")]
    Format(#[from] std::fmt::Error),
}

// https://www.ucolick.org/~sla/leapsecs/dutc.html
//
// Before the year 4000 we expect there will be more than one leap
// second each month, at which point UTC as currently defined will no
// longer work. At that time DTAI is expected to be less than 4 hours,
// i.e. 14,400 seconds, which is less than 2^16.

#[derive(Debug, Eq, PartialEq)]
pub enum LeapSecond {
    Zero { mjd: i32, dtai: u16 },
    Neg { mjd: i32, dtai: u16 },
    Pos { mjd: i32, dtai: u16 },
    Exp { mjd: i32 },
}

fn timestamp(ntp: u64) -> Result<TimeStamp, NISTerror> {
    let ts = TimeStamp::from(ntp);
    if ts.date.year() < 1972 {
        Err(NISTerror::TooOld(ts))
    } else if ts != TimeStamp::from(ts.date) {
        Err(NISTerror::Fractional(ts))
    } else {
        Ok(ts)
    }
}

fn leapsecond(
    (ntp, dtai64, date): UncheckedLeap,
) -> Result<(i32, u16), NISTerror> {
    let ts = timestamp(ntp)?;
    let dtai16 = dtai64 as u16;
    if dtai64 != dtai16 as u64 {
        Err(NISTerror::Spinny(dtai64))
    } else if ts.date != date {
        Err(NISTerror::Mismatch(ts, TimeStamp::from(date)))
    } else {
        Ok((ts.mjd, dtai16))
    }
}

fn sha1(input: &str) -> [u8; 20] {
    let hash = digest(&SHA1_FOR_LEGACY_USE_ONLY, input.as_bytes());
    // panic if sha1 is not the standard size
    hash.as_ref().try_into().unwrap()
}

pub fn check(u: UncheckedNIST) -> Result<Vec<LeapSecond>, NISTerror> {
    let updated = timestamp(u.updated)?.mjd;
    let expires_ts = timestamp(u.expires)?;
    let expires = expires_ts.mjd;
    if expires < today() {
        return Err(NISTerror::Expired(expires_ts));
    }
    let mut hashin = String::new();
    write!(hashin, "{}{}", u.updated, u.expires)?;
    for leap in u.leapsecs {
        write!(hashin, "{}{}", leap.0, leap.1)?;
    }
    let hash = sha1(&hashin);
    if u.hash != hash {
        return Err(NISTerror::Checksum(u.hash, hash, hashin));
    }
    let leapsecs = Vec::new();
    Ok(leapsecs)
}
