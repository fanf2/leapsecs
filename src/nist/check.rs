use ring::digest::*;
use std::convert::TryInto;
use std::fmt::Write;

use super::{Error, TimeStamp};
use super::{UncheckedLeap, UncheckedNIST};
use crate::date::*;
use crate::leap::*;

impl From<i64> for TimeStamp {
    fn from(ntp: i64) -> TimeStamp {
        let mjd = ntp2mjd(ntp);
        let date = Gregorian::from(mjd);
        TimeStamp { ntp, mjd, date }
    }
}

impl From<i32> for TimeStamp {
    fn from(mjd: i32) -> TimeStamp {
        let ntp = mjd2ntp(mjd);
        let date = Gregorian::from(mjd);
        TimeStamp { ntp, mjd, date }
    }
}

fn sha1(input: &str) -> [u8; 20] {
    let hash = digest(&SHA1_FOR_LEGACY_USE_ONLY, input.as_bytes());
    // panic if sha1 is not the standard size
    hash.as_ref().try_into().unwrap()
}

fn timestamp(ntp: i64) -> Result<TimeStamp, Error> {
    let ts = TimeStamp::from(ntp);
    if ts.date.year() < 1972 {
        Err(Error::TooSoon(ts))
    } else if ts != TimeStamp::from(ts.mjd) {
        Err(Error::Fractional(ts))
    } else {
        Ok(ts)
    }
}

fn check_next(
    acc: Result<LeapSecs, Error>,
    &(ntp, dtai64, date): &UncheckedLeap,
) -> Result<LeapSecs, Error> {
    let mut list = acc?;
    let ts = timestamp(ntp)?;
    let mjd = ts.mjd;
    let dtai = dtai64 as i16;
    if dtai as i64 != dtai64 {
        return Err(Error::Spinny(ts, dtai64));
    }
    if ts.date != date {
        return Err(Error::Mismatch(ts, date));
    }
    let next = if let Some(last) = list.last() {
        if mjd <= last.mjd() {
            return Err(Error::OutOfOrder(TimeStamp::from(last.mjd()), ts));
        } else if dtai == last.dtai() - 1 {
            LeapSecond::Neg { mjd, dtai }
        } else if dtai == last.dtai() {
            return Err(Error::NoLeap(ts, dtai));
        } else if dtai == last.dtai() + 1 {
            LeapSecond::Pos { mjd, dtai }
        } else {
            return Err(Error::LargeLeap(ts, last.dtai(), dtai));
        }
    } else {
        if mjd != i32::from(Gregorian(1972, 1, 1)) || dtai != 10 {
            return Err(Error::FalseStart(ts, dtai));
        } else {
            LeapSecond::Zero { mjd, dtai }
        }
    };
    list.push(next);
    Ok(list)
}

pub(super) fn check(u: UncheckedNIST) -> Result<LeapSecs, Error> {
    // process the list of leap seconds
    let mut list = u.leapsecs.iter().fold(Ok(Vec::new()), check_next)?;
    let updated = timestamp(u.updated)?;
    let expires = timestamp(u.expires)?;
    if expires.mjd <= today() {
        return Err(Error::Expired(expires));
    } else if let Some(last) = list.last() {
        if expires.mjd <= last.mjd() {
            return Err(Error::TooLate(TimeStamp::from(last.mjd())));
        } else {
            list.push(LeapSecond::Exp { mjd: expires.mjd });
        }
    } else {
        return Err(Error::Empty(updated));
    }
    // calculate and compare checksum
    let mut hashin = String::new();
    write!(hashin, "{}{}", u.updated, u.expires)?;
    for leap in &u.leapsecs {
        write!(hashin, "{}{}", leap.0, leap.1)?;
    }
    let hash = sha1(&hashin);
    if u.hash != hash {
        return Err(Error::Checksum(u.hash, hash, hashin));
    }
    Ok(list)
}
