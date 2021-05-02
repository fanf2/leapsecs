use ring::digest::*;
use std::convert::TryInto;
use std::fmt::Write;

use super::{Error, TimeStamp};
use super::{UncheckedLeap, UncheckedNIST};
use crate::date::*;
use crate::leap::*;

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

fn sha1(input: &str) -> [u8; 20] {
    let hash = digest(&SHA1_FOR_LEGACY_USE_ONLY, input.as_bytes());
    // panic if sha1 is not the standard size
    hash.as_ref().try_into().unwrap()
}

fn timestamp(ntp: u64) -> Result<TimeStamp, Error> {
    let ts = TimeStamp::from(ntp);
    if ts.date.year() < 1972 {
        Err(Error::TooSoon(ts))
    } else if ts != TimeStamp::from(ts.date) {
        Err(Error::Fractional(ts))
    } else {
        Ok(ts)
    }
}

type CheckOne = Result<(i32, u16), Error>;

fn leapsecond(expires: i32, (ntp, dtai64, date): UncheckedLeap) -> CheckOne {
    let ts = timestamp(ntp)?;
    let dtai16 = dtai64 as u16;
    if dtai64 != dtai16 as u64 {
        Err(Error::Spinny(dtai64))
    } else if ts.date != date {
        Err(Error::Mismatch(ts, TimeStamp::from(date)))
    } else if ts.mjd >= expires {
        Err(Error::TooLate(ts))
    } else {
        Ok((ts.mjd, dtai16))
    }
}

fn sequence(
    acc: Result<LeapSecs, Error>,
    u: CheckOne,
) -> Result<LeapSecs, Error> {
    let mut list = acc?;
    let (mjd, dtai) = u?;
    let next = if let Some(last) = list.last() {
        if mjd <= last.mjd() {
            return Err(Error::OutOfOrder(
                TimeStamp::from(last.mjd()),
                TimeStamp::from(mjd),
            ));
        } else if dtai == last.dtai() {
            return Err(Error::NoLeap(TimeStamp::from(mjd), dtai));
        } else if dtai == last.dtai() - 1 {
            LeapSecond::Neg { mjd, dtai }
        } else if dtai == last.dtai() + 1 {
            LeapSecond::Pos { mjd, dtai }
        } else {
            return Err(Error::LargeLeap(
                TimeStamp::from(mjd),
                last.dtai(),
                dtai,
            ));
        }
    } else {
        if mjd != i32::from(Gregorian(1972, 1, 1)) || dtai != 10 {
            return Err(Error::FalseStart(TimeStamp::from(mjd), dtai));
        } else {
            LeapSecond::Zero { mjd, dtai }
        }
    };
    list.push(next);
    Ok(list)
}

pub(super) fn check(u: UncheckedNIST) -> Result<LeapSecs, Error> {
    let _updated = timestamp(u.updated)?.mjd;
    let expires_ts = timestamp(u.expires)?;
    let expires = expires_ts.mjd;
    if expires < today() {
        return Err(Error::Expired(expires_ts));
    }
    let mut hashin = String::new();
    write!(hashin, "{}{}", u.updated, u.expires)?;
    for leap in &u.leapsecs {
        write!(hashin, "{}{}", leap.0, leap.1)?;
    }
    let hash = sha1(&hashin);
    if u.hash != hash {
        return Err(Error::Checksum(u.hash, hash, hashin));
    }
    let mut leapsecs = u
        .leapsecs
        .iter()
        .map(|&uls| leapsecond(expires, uls))
        .fold(Ok(Vec::new()), sequence)?;
    leapsecs.push(LeapSecond::Exp { mjd: expires });
    Ok(leapsecs)
}
