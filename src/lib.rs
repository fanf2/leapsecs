use std::convert::TryFrom;
use std::ops::Index;
use thiserror::Error;

mod bin;
mod txt;

pub mod date;
pub mod nist;

use crate::nist::Hash;
use date::*;

//  ___             _ _       ___
// | _ \___ ____  _| | |_    | __|_ _ _ _ ___ _ _
// |   / -_|_-< || | |  _|_  | _|| '_| '_/ _ \ '_|
// |_|_\___/__/\_,_|_|\__( ) |___|_| |_| \___/_|
//                       |/

pub type Result<T> = std::result::Result<T, Error>;

#[derive(Error, Debug)]
pub enum Error {
    #[error("checksum failed {0} <> {1} data {2}")]
    Checksum(Hash, Hash, String),
    #[error("leap seconds list is empty")]
    Empty,
    #[error("leap seconds list has expired ({0})")]
    Expired(Gregorian),
    #[error("incorrect starting point {0}")]
    FalseStart(Gregorian, i16),
    #[error("format error {0}")]
    Format(#[from] std::fmt::Error),
    #[error("overflow in date arithmetic")]
    FromInt(#[from] std::num::TryFromIntError),
    #[error("expected {0}, found {1}")]
    FromStr(&'static str, char),
    #[error("gap must be between 1 and 999 months")]
    Gap(Gregorian, i32, Gregorian),
    #[error("can't add more leap seconds after expiry time ({0})")]
    LeapAfterExp(Gregorian, Gregorian),
    #[error("time is not midnight (NTP {0} is {1} + {2})")]
    Midnight(i64, MJD, i32),
    #[error("date {0} is not {1} of month")]
    MonthDay(Gregorian, i32),
    #[error("parse error {0}")]
    Nom(String),
    #[error("timestamp and date do not match (NTP {0} is {1} <> {2})")]
    TimeDate(i64, MJD, Gregorian),
    #[error("missing expiry date at end of list")]
    Truncated,
    #[error("{0}")]
    Unicode(#[from] std::str::Utf8Error),
    #[error("leap is not +1 or -1 ({0} -> {1})")]
    WrongLeap(Gregorian, i16, Gregorian, i16),
}

//  _
// | |   ___ __ _ _ __
// | |__/ -_) _` | '_ \
// |____\___\__,_| .__/
//               |_|

#[derive(Copy, Clone, Debug, Eq, PartialEq)]
pub enum Leap {
    Zero,
    Neg,
    Pos,
    Exp,
}

use Leap::*;

//  _                  ___
// | |   ___ __ _ _ __/ __| ___ __
// | |__/ -_) _` | '_ \__ \/ -_) _|
// |____\___\__,_| .__/___/\___\__|
//               |_|

// https://www.ucolick.org/~sla/leapsecs/dutc.html
//
// Before the year 4000 we expect there will be more than one leap
// second each month, at which point UTC as currently defined will no
// longer work. At that time DTAI is expected to be less than 4 hours,
// i.e. 14,400 seconds, which is less than 2^15.

#[derive(Copy, Clone, Debug, Eq, PartialEq)]
pub struct LeapSec {
    gap: u16,
    sign: Leap,
    month: u16,
    dtai: Option<i16>,
}

fn date_of(month: i32, day: i32) -> Gregorian {
    let year = month.div_euclid(12);
    let month = month.rem_euclid(12);
    Gregorian(1972 + year, month + 1, day)
}

fn month_of(date: Gregorian, day: i32) -> Result<i32> {
    if date.day() == day {
        Ok((date.year() - 1972) * 12 + (date.month() - 1))
    } else {
        Err(Error::MonthDay(date, day))
    }
}

// NIST and IERS leap second tables expire on the 28th of the month
const EXPIRES_DATE: i32 = 28;

impl LeapSec {
    pub fn date(self) -> Gregorian {
        let mut date = date_of(self.month as i32, 1);
        if self.sign == Exp {
            date.2 = EXPIRES_DATE;
        }
        date
    }
    pub fn dtai(self) -> Result<i16> {
        self.dtai.ok_or_else(|| Error::Expired(self.date()))
    }
    pub fn gap(self) -> u16 {
        self.gap
    }
    pub fn sign(self) -> Leap {
        self.sign
    }
    pub fn mjd(self) -> MJD {
        MJD::from(self.date())
    }
    fn start() -> LeapSec {
        LeapSec { gap: 0, sign: Zero, month: 0, dtai: Some(10) }
    }
}

impl std::fmt::Display for LeapSec {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let leap = match self.sign {
            Zero => "  ",
            Neg => "-1",
            Pos => "+1",
            Exp => return write!(f, "{} ??", self.date()),
        };
        write!(f, "{} {} DTAI {}", self.date(), leap, self.dtai().unwrap())
    }
}

//  _                  ___
// | |   ___ __ _ _ __/ __| ___ ___ ___
// | |__/ -_) _` | '_ \__ \/ -_) __(_-<
// |____\___\__,_| .__/___/\___\___/__/
//               |_|

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct LeapSecs(Vec<LeapSec>);

impl LeapSecs {
    pub fn builder() -> LeapSecBuilder {
        Default::default()
    }
    pub fn expires(&self) -> MJD {
        self.0.last().unwrap().mjd()
    }
    pub fn is_empty(&self) -> bool {
        self.0.is_empty()
    }
    pub fn iter(&self) -> std::slice::Iter<'_, LeapSec> {
        self.into_iter()
    }
    pub fn len(&self) -> usize {
        self.0.len()
    }
}

impl Index<usize> for LeapSecs {
    type Output = LeapSec;

    fn index(&self, i: usize) -> &LeapSec {
        &self.0[i]
    }
}

impl<'a> IntoIterator for &'a LeapSecs {
    type Item = &'a LeapSec;
    type IntoIter = std::slice::Iter<'a, LeapSec>;

    fn into_iter(self) -> Self::IntoIter {
        self.0.iter()
    }
}

//  _                  ___          ___      _ _    _
// | |   ___ __ _ _ __/ __| ___ ___| _ )_  _(_) |__| |___ _ _
// | |__/ -_) _` | '_ \__ \/ -_) __| _ \ || | | / _` / -_) '_|
// |____\___\__,_| .__/___/\___\___|___/\_,_|_|_\__,_\___|_|
//               |_|

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct LeapSecBuilder(Vec<LeapSec>);

impl Default for LeapSecBuilder {
    fn default() -> LeapSecBuilder {
        LeapSecBuilder::new()
    }
}

impl LeapSecBuilder {
    pub fn new() -> LeapSecBuilder {
        LeapSecBuilder(Vec::new())
    }

    pub fn finish(mut self) -> Result<LeapSecs> {
        let last = self.last()?;
        if last.sign != Exp {
            Err(Error::Truncated)
        } else if last.mjd() < date::today() {
            Err(Error::Expired(last.date()))
        } else {
            self.0.shrink_to_fit();
            Ok(LeapSecs(self.0))
        }
    }

    fn last(&self) -> Result<LeapSec> {
        match self.0.last() {
            Some(&last) => Ok(last),
            None => Err(Error::Empty),
        }
    }
    fn push_start(&mut self) {
        self.0.push(LeapSec::start());
    }

    fn push_leap_sec(
        &mut self,
        last: LeapSec,
        mut gap: i32,
        sign: Leap,
        mut month: i32,
        dtai: Option<i16>,
    ) -> Result<()> {
        if last.sign == Exp {
            return Err(Error::LeapAfterExp(last.date(), date_of(month, 1)));
        }
        if last.sign == Zero && last.month != 0 {
            gap += last.gap as i32;
            month += last.gap as i32;
            self.0.pop();
        }
        let gap = match gap {
            1..=999 => gap as u16,
            _ => return Err(Error::Gap(last.date(), gap, date_of(month, 1))),
        };
        let month = u16::try_from(month)?;
        assert_eq!(last.month + gap, month);
        assert_eq!(sign == Exp, dtai == None);
        self.0.push(LeapSec { gap, sign, month, dtai });
        Ok(())
    }

    pub fn push_gap(&mut self, gap: i32, sign: Leap) -> Result<()> {
        if self.0.is_empty() {
            self.push_start();
        }
        let last = self.last()?;
        let month = last.month as i32 + gap;
        let ldtai = last.dtai()?;
        let dtai = match sign {
            Zero => Some(ldtai),
            Neg => Some(ldtai - 1),
            Pos => Some(ldtai + 1),
            Exp => None,
        };
        self.push_leap_sec(last, gap, sign, month, dtai)
    }

    pub fn push_exp(&mut self, date: Gregorian) -> Result<()> {
        let month = month_of(date, EXPIRES_DATE)?;
        let last = self.last()?;
        let gap = month - last.month as i32;
        self.push_leap_sec(last, gap, Exp, month, None)
    }

    pub fn push_date(&mut self, date: Gregorian, dtai: i16) -> Result<()> {
        let month = month_of(date, 1)?;
        let last = if let Ok(last) = self.last() {
            last
        } else if month == 0 && dtai == 10 {
            self.push_start();
            return Ok(());
        } else {
            return Err(Error::FalseStart(date, dtai));
        };

        let gap = month - last.month as i32;
        let sign = match dtai - last.dtai()? {
            -1 => Neg,
            1 => Pos,
            _ => {
                return Err(Error::WrongLeap(
                    last.date(),
                    last.dtai()?,
                    date,
                    dtai,
                ))
            }
        };
        self.push_leap_sec(last, gap, sign, month, Some(dtai))
    }
}
