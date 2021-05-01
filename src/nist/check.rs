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

impl From<i32> for TimeStamp {
    fn from(mjd: i32) -> TimeStamp {
        let epoch = i32::from(Gregorian(1900, 1, 1));
        let date = Gregorian::from(mjd);
        let ntp = (mjd - epoch) as u64 * 86400;
        TimeStamp { ntp, mjd, date }
    }
}

#[derive(Error, Debug)]
pub enum NISTerror {
    #[error("checksum failed {0:?} <> {1:?} data {2}")]
    Checksum([u8; 20], [u8; 20], String),
    #[error("leap seconds file has expired ({0:?})")]
    Expired(TimeStamp),
    #[error("starts with {1} seconds at {0:?}")]
    FalseStart(TimeStamp, u16),
    #[error("format error {0}")]
    Format(#[from] std::fmt::Error),
    #[error("timestamp is not midnight ({0:?})")]
    Fractional(TimeStamp),
    #[error("leap more than one second ({1} -> {2} at {0:?})")]
    LargeLeap(TimeStamp, u16, u16),
    #[error("timestamp and date do not match ({0:?} <> {1:?})")]
    Mismatch(TimeStamp, TimeStamp),
    #[error("lack of leap ({1} at {0:?})")]
    NoLeap(TimeStamp, u16),
    #[error("leap seconds are disordered ({0:?} > {1:?})")]
    OutOfOrder(TimeStamp, TimeStamp),
    #[error("DTAI is too large ({0:?})")]
    Spinny(u64),
    #[error("leap second is after expiry time ({0:?})")]
    TooLate(TimeStamp),
    #[error("timestamp is before 1972 ({0:?})")]
    TooSoon(TimeStamp),
}

// https://www.ucolick.org/~sla/leapsecs/dutc.html
//
// Before the year 4000 we expect there will be more than one leap
// second each month, at which point UTC as currently defined will no
// longer work. At that time DTAI is expected to be less than 4 hours,
// i.e. 14,400 seconds, which is less than 2^16.

#[derive(Copy, Clone, Debug, Eq, PartialEq)]
pub enum LeapSecond {
    Zero { mjd: i32, dtai: u16 },
    Neg { mjd: i32, dtai: u16 },
    Pos { mjd: i32, dtai: u16 },
    Exp { mjd: i32 },
}

impl LeapSecond {
    fn mjd(self) -> i32 {
        match self {
            Self::Zero { mjd, .. } => mjd,
            Self::Neg { mjd, .. } => mjd,
            Self::Pos { mjd, .. } => mjd,
            Self::Exp { mjd } => mjd,
        }
    }
    fn dtai(self) -> u16 {
        match self {
            Self::Zero { dtai, .. } => dtai,
            Self::Neg { dtai, .. } => dtai,
            Self::Pos { dtai, .. } => dtai,
            Self::Exp { .. } => panic!(),
        }
    }
}

fn sha1(input: &str) -> [u8; 20] {
    let hash = digest(&SHA1_FOR_LEGACY_USE_ONLY, input.as_bytes());
    // panic if sha1 is not the standard size
    hash.as_ref().try_into().unwrap()
}

fn timestamp(ntp: u64) -> Result<TimeStamp, NISTerror> {
    let ts = TimeStamp::from(ntp);
    if ts.date.year() < 1972 {
        Err(NISTerror::TooSoon(ts))
    } else if ts != TimeStamp::from(ts.date) {
        Err(NISTerror::Fractional(ts))
    } else {
        Ok(ts)
    }
}

type CheckOne = Result<(i32, u16), NISTerror>;

fn leapsecond(expires: i32, (ntp, dtai64, date): UncheckedLeap) -> CheckOne {
    let ts = timestamp(ntp)?;
    let dtai16 = dtai64 as u16;
    if dtai64 != dtai16 as u64 {
        Err(NISTerror::Spinny(dtai64))
    } else if ts.date != date {
        Err(NISTerror::Mismatch(ts, TimeStamp::from(date)))
    } else if ts.mjd >= expires {
        Err(NISTerror::TooLate(ts))
    } else {
        Ok((ts.mjd, dtai16))
    }
}

type Checked = Result<Vec<LeapSecond>, NISTerror>;

fn sequence(acc: Checked, u: CheckOne) -> Checked {
    let mut list = acc?;
    let (mjd, dtai) = u?;
    let next = if let Some(last) = list.last() {
        if mjd <= last.mjd() {
            return Err(NISTerror::OutOfOrder(
                TimeStamp::from(last.mjd()),
                TimeStamp::from(mjd),
            ));
        } else if dtai == last.dtai() {
            return Err(NISTerror::NoLeap(TimeStamp::from(mjd), dtai));
        } else if dtai == last.dtai() - 1 {
            LeapSecond::Neg { mjd, dtai }
        } else if dtai == last.dtai() + 1 {
            LeapSecond::Pos { mjd, dtai }
        } else {
            return Err(NISTerror::LargeLeap(
                TimeStamp::from(mjd),
                last.dtai(),
                dtai,
            ));
        }
    } else {
        if mjd != i32::from(Gregorian(1972, 1, 1)) || dtai != 10 {
            return Err(NISTerror::FalseStart(TimeStamp::from(mjd), dtai));
        } else {
            LeapSecond::Zero { mjd, dtai }
        }
    };
    list.push(next);
    Ok(list)
}

pub fn check(u: UncheckedNIST) -> Checked {
    let _updated = timestamp(u.updated)?.mjd;
    let expires_ts = timestamp(u.expires)?;
    let expires = expires_ts.mjd;
    if expires < today() {
        return Err(NISTerror::Expired(expires_ts));
    }
    let mut hashin = String::new();
    write!(hashin, "{}{}", u.updated, u.expires)?;
    for leap in &u.leapsecs {
        write!(hashin, "{}{}", leap.0, leap.1)?;
    }
    let hash = sha1(&hashin);
    if u.hash != hash {
        return Err(NISTerror::Checksum(u.hash, hash, hashin));
    }
    let mut leapsecs = u
        .leapsecs
        .iter()
        .map(|&uls| leapsecond(expires, uls))
        .fold(Ok(Vec::new()), sequence)?;
    leapsecs.push(LeapSecond::Exp { mjd: expires });
    Ok(leapsecs)
}
