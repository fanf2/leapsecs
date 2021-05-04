use thiserror::Error;

use crate::date::*;
use crate::nist::Hash;

pub enum Leap {
    Zero,
    Neg,
    Pos,
    Exp,
}

pub use crate::from::LeapSecs;

// https://www.ucolick.org/~sla/leapsecs/dutc.html
//
// Before the year 4000 we expect there will be more than one leap
// second each month, at which point UTC as currently defined will no
// longer work. At that time DTAI is expected to be less than 4 hours,
// i.e. 14,400 seconds, which is less than 2^15.

#[derive(Copy, Clone, Debug, Eq, PartialEq)]
pub enum LeapSec {
    Zero { mjd: i32, dtai: i16 },
    Neg { mjd: i32, dtai: i16 },
    Pos { mjd: i32, dtai: i16 },
    Exp { mjd: i32 },
}

impl LeapSec {
    pub fn mjd(self) -> i32 {
        match self {
            Self::Zero { mjd, .. } => mjd,
            Self::Neg { mjd, .. } => mjd,
            Self::Pos { mjd, .. } => mjd,
            Self::Exp { mjd } => mjd,
        }
    }
    pub fn dtai(self) -> i16 {
        match self {
            Self::Zero { dtai, .. } => dtai,
            Self::Neg { dtai, .. } => dtai,
            Self::Pos { dtai, .. } => dtai,
            Self::Exp { .. } => panic!(),
        }
    }
    pub fn zero() -> Self {
        Self::Zero { mjd: i32::from(Gregorian(1972, 1, 1)), dtai: 10 }
    }
    pub(crate) fn month_zero(month: i32, dtai: i16) -> Self {
        LeapSec::Zero { mjd: month2mjd(month), dtai }
    }
    pub(crate) fn month_neg(month: i32, dtai: i16) -> Self {
        LeapSec::Neg { mjd: month2mjd(month), dtai }
    }
    pub(crate) fn month_pos(month: i32, dtai: i16) -> Self {
        LeapSec::Pos { mjd: month2mjd(month), dtai }
    }
    pub(crate) fn month_exp(month: i32) -> Self {
        // NIST expiry date is 28th of the month
        LeapSec::Exp { mjd: month2mjd(month) + 27 }
    }
}

impl std::fmt::Display for LeapSec {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match *self {
            LeapSec::Zero { mjd, dtai } => {
                write!(f, "{}    DTAI {}", MJD(mjd), dtai)
            }
            LeapSec::Neg { mjd, dtai } => {
                write!(f, "{} -1 DTAI {}", MJD(mjd), dtai)
            }
            LeapSec::Pos { mjd, dtai } => {
                write!(f, "{} +1 DTAI {}", MJD(mjd), dtai)
            }
            LeapSec::Exp { mjd } => write!(f, "{} ??", MJD(mjd)),
        }
    }
}

pub type Result<T> = std::result::Result<T, Error>;

#[derive(Error, Debug)]
pub enum Error {
    #[error("checksum failed {0} <> {1} data {2}")]
    Checksum(Hash, Hash, String),
    #[error("leap seconds list is empty")]
    Empty,
    #[error("leap seconds list has expired ({0})")]
    Expired(LeapSec),
    #[error("incorrect starting point {0}")]
    FalseStart(LeapSec),
    #[error("format error {0}")]
    Format(#[from] std::fmt::Error),
    #[error("expected {0}, found {1}")]
    FromStr(&'static str, char),
    #[error("time is not midnight ({0})")]
    Midnight(NTP),
    #[error("date is not first of month ({0})")]
    MonthFirst(MJD),
    #[error("parse error {0}")]
    Nom(String),
    #[error("leap seconds are disordered ({0} > {1})")]
    OutOfOrder(LeapSec, LeapSec),
    #[error("timestamp and date do not match ({0} <> {1})")]
    TimeDate(NTP, Gregorian),
    #[error("{0}")]
    Unicode(#[from] std::str::Utf8Error),
    #[error("leap is not -1 ({0} -> {1})")]
    WrongNeg(LeapSec, LeapSec),
    #[error("leap is not +1 ({0} -> {1})")]
    WrongPos(LeapSec, LeapSec),
}
