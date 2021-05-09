//! Compact formats for the leap second list
//! ========================================
//!
//! The `leapsecs` crate can read and write the list of leap seconds
//! in a number of formats.
//!
//!   * A compact text format, implemented by the [`txt`][] module.
//!     For example, the current list is:
//!
//! ```text
//! 6+6+12+12+12+12+12+12+12+18+12+12+24+30+24+12+18+12+12+18+18+18+84+36+42+36+18+59?
//! ```
//!
//!   * A compact binary format, implemented by the [`bin`][] module.
//!     For example, the current list is:
//!
//! ```text
//! 00111111 12113431 2112229D 565287FA
//! ```
//!
//!   * The NIST `leap-seconds.list` format, implemented by the [`nist`][] module.
//!
//! The main interface is through the [`LeapSecs`][] type and the standard
//! conversion traits that it implements. These are documented in the
//! [`txt`][] and [`bin`][] modules.
//!
//! [`LeapSecs`][] contains a list of [`LeapSec`][] objects that mostly
//! represent individual leap seconds. The modules in [`leapsecs`][self] use
//! a [`LeapSecBuilder`][] to construct a [`LeapSecs`][] list.
//!
//! The [`enum@Error`][] type collects together the possible kinds of
//! conversion failures.

#![doc(
    html_logo_url = "https://raw.githubusercontent.com/fanf2/leapsecs/main/doc/logo.png"
)]

use std::convert::TryFrom;
use std::ops::Index;
use thiserror::Error;

pub mod bin;
pub mod date;
pub mod nist;
pub mod txt;

use crate::nist::Hash;
pub use date::*;

//  ___             _ _       ___
// | _ \___ ____  _| | |_    | __|_ _ _ _ ___ _ _
// |   / -_|_-< || | |  _|_  | _|| '_| '_/ _ \ '_|
// |_|_\___/__/\_,_|_|\__( ) |___|_| |_| \___/_|
//                       |/

/// A specialized [`Result`][] type to avoid writing out
/// [`leapsecs::Error`][enum@Error].
///
pub type Result<T> = std::result::Result<T, Error>;

/// The error type for leap seconds.
///
/// This covers things like parsing errors, consistency errors,
/// overflows - almost all errors in `leapsecs`.
///
/// The exceptions are IO-related errors from the `nist` module, which
/// use `anyhow::Result` because those functions are more
/// application-oriented.
///
#[derive(Error, Debug)]
pub enum Error {
    /// The NIST `leap-seconds.list` checksum did not match.
    #[error("checksum failed {0} <> {1} data {2}")]
    Checksum(Hash, Hash, String),
    /// Attempted to create an empty list
    #[error("leap seconds list is empty")]
    Empty,
    /// Attempted to use a list after its expiry date
    #[error("leap seconds list has expired ({0})")]
    Expired(Gregorian),
    /// Attempted to create a list that doesn't start at 1972-01-01 DTAI=10
    #[error("incorrect starting point {0}")]
    FalseStart(Gregorian, i16),
    /// An error occurred when converting `LeapSecs` to a string
    #[error("format error {0}")]
    Format(#[from] std::fmt::Error),
    /// We encountered a date in the distant past or future
    #[error("overflow in date arithmetic")]
    FromInt(#[from] std::num::TryFromIntError),
    /// Syntax error in the compact text format of the leap seconds list
    #[error("expected {0}, found {1}")]
    FromStr(&'static str, char),
    /// The leap seconds list is out of order or excessively spaced out
    #[error("gap must be between 1 and 999 months")]
    Gap(Gregorian, i32, Gregorian),
    /// There can't be any leap seconds after the list's expiry date
    #[error("can't add more leap seconds after expiry time ({0})")]
    LeapAfterExp(Gregorian, Gregorian),
    /// Timestamps in the NIST `leap-seconds.list` should be at midnight
    #[error("time is not midnight (NTP {0} is {1} + {2})")]
    Midnight(i64, MJD, i32),
    /// Leap seconds should occur just before the 1st of the month,
    /// and expiry dates should be the 28th of the month.
    #[error("date {0} is not {1} of month")]
    MonthDay(Gregorian, i32),
    /// Syntax error in the NIST `leap-seconds.list`
    #[error("parse error {0}")]
    Nom(String),
    /// Mismatched timestamp and date in the NIST `leap-seconds.list`
    #[error("timestamp and date do not match (NTP {0} is {1} <> {2})")]
    TimeDate(i64, MJD, Gregorian),
    /// The leap seconds list lacks an expiry date
    #[error("missing expiry date at end of list")]
    Truncated,
    /// The NIST `leap-seconds.list` is not valid UTF-8
    #[error("{0}")]
    Unicode(#[from] std::str::Utf8Error),
    /// A leap second is not exactly +1 or -1
    #[error("leap is not +1 or -1 ({0} -> {1})")]
    WrongLeap(Gregorian, i16, Gregorian, i16),
}

//  _
// | |   ___ __ _ _ __
// | |__/ -_) _` | '_ \
// |____\___\__,_| .__/
//               |_|

/// The possible kinds of [`LeapSec`][].
///
/// This type is also used with [`LeapSecBuilder`][] and elsewhere
/// inside [`leapsecs`][self].
///
#[derive(Copy, Clone, Debug, Eq, PartialEq)]
pub enum Leap {
    /// Used for the first entry in the list, to represent the
    /// starting point rather than a leap second.
    ///
    /// Also used by the [`bin`][] format to represent long gaps
    /// between leap seconds.
    ///
    Zero,
    /// A negative leap second.
    Neg,
    /// A positive leap second.
    Pos,
    /// Used for the last entry in the list, to represent when its
    /// validity period expires.
    Exp,
}

use Leap::*;

//  _                  ___
// | |   ___ __ _ _ __/ __| ___ __
// | |__/ -_) _` | '_ \__ \/ -_) _|
// |____\___\__,_| .__/___/\___\__|
//               |_|

/// An entry in a [`LeapSecs`][] list.
///
/// A [`LeapSec`][] is read-only and immutable. Every [`LeapSec`][] is
/// constructed by a [`LeapSecBuilder`][] as part of a [`LeapSecs`][] list.
///
/// The first entry in the list is [`Leap::Zero`][], representing the initial
/// value of DTAI before any leap seconds.
///
/// The last entry in the list is [`Leap::Exp`][], representing the list's
/// expiry date.
///
/// The entries in between are [`Leap::Pos`][] or maybe [`Leap::Neg`][] leap
/// seconds.
///
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

    /// Get the date immediately following the leap second. This is
    /// the date from which [`LeapSec::dtai()`][] is valid, or the list's
    /// expiry date if this [`LeapSec`][] is the last entry.
    ///
    pub fn date(self) -> Gregorian {
        let mut date = date_of(self.month as i32, 1);
        if self.sign == Exp {
            date.2 = EXPIRES_DATE;
        }
        date
    }

    /// Get the difference between UTC and TAI after this leap second.
    ///
    /// The last entry in the list represents its expiry date, after
    /// which DTAI is not valid, in which case this returns
    /// [`Error::Expired`][]
    ///
    /// Before the year 4000 we expect there will be more than one leap
    /// second each month, at which point UTC as currently defined will no
    /// longer work. At that time DTAI is expected to be less than 4 hours,
    /// i.e. 14,400 seconds, which fits in `i16`. For more details, see
    /// <https://www.ucolick.org/~sla/leapsecs/dutc.html>
    ///
    pub fn dtai(self) -> Result<i16> {
        self.dtai.ok_or_else(|| Error::Expired(self.date()))
    }

    /// Get the length of the gap between the previous leap second and this
    /// one, counted in months.
    ///
    /// The compact leap second formats limit this value to at most 999.
    ///
    pub fn gap(self) -> u16 {
        self.gap
    }

    /// Get the [`MJD`][] of the [`LeapSec::date()`][] immediately following
    /// this leap second.
    ///
    pub fn mjd(self) -> MJD {
        MJD::from(self.date())
    }

    /// What kind of leap second this is
    ///
    pub fn sign(self) -> Leap {
        self.sign
    }

    /// Get the value first entry in a [`LeapSecs`][] list
    ///
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

/// A list of [`LeapSec`][] leap second objects.
///
/// A [`LeapSecs`][] list is a read-only immutable object, constructed by a
/// [`LeapSecBuilder`][].
///
/// You can index a [`LeapSecs`][] list with `[]` and iterate over it in a
/// similar manner to a `Vec` or slice.
///
/// The conversion traits implemented for [`LeapSecs`][] are documented in the
/// [`txt`][] and [`bin`][] modules.
///
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct LeapSecs(Vec<LeapSec>);

impl LeapSecs {

    /// Convenience method for getting a [`LeapSecBuilder`][]
    pub fn builder() -> LeapSecBuilder {
        Default::default()
    }

    /// Get the expiry date of the list.
    pub fn expires(&self) -> MJD {
        self.0.last().unwrap().mjd()
    }

    /// Returns true if [`LeapSecs::len()`][] is zero
    pub fn is_empty(&self) -> bool {
        self.0.is_empty()
    }

    /// Get an iterator over the [`LeapSec`][] elements
    pub fn iter(&self) -> std::slice::Iter<'_, LeapSec> {
        self.into_iter()
    }

    /// Get the number of [`LeapSec`][] elements
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

/// Construct a [`LeapSecs`][] list, checking it for validity.
///
/// There are two primary ways to use a [`LeapSecBuilder`][], depending on
/// the format of the source data.
///
/// The parsers for the compact leap second list formats use
/// [`LeapSecBuilder::push_gap()`][].
///
/// The [`nist`][] parser uses [`LeapSecBuilder::push_date()`][] and
/// [`LeapSecBuilder::push_exp()`][].
///
/// It isn't an error to use both ways to construct a list, but why would
/// you?
///
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct LeapSecBuilder(Vec<LeapSec>);

impl Default for LeapSecBuilder {
    fn default() -> LeapSecBuilder {
        LeapSecBuilder::new()
    }
}

impl LeapSecBuilder {

    /// Get a new [`LeapSecBuilder`][]
    pub fn new() -> LeapSecBuilder {
        LeapSecBuilder(Vec::new())
    }

    /// Do the final consistency checks on the [`LeapSecBuilder`][] and
    /// if they pass, return the completed  [`LeapSecs`][] list.
    ///
    pub fn finish(mut self) -> Result<LeapSecs> {
        let last = self.last()?;
        if last.sign != Exp {
            Err(Error::Truncated)
        } else if last.mjd() < MJD::today() {
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

    /// Add an entry to the list
    ///
    /// `gap` is the time between the last entry and this one, measured in
    /// months
    ///
    /// `sign` is what kind of leap second this is.
    ///
    /// When `sign` is [`Leap::Zero`][], `push_gap()` builds a single entry
    /// in multiple steps.
    ///
    /// When `sign` is [`Leap::Exp`][], `push_gap()` adds the expiry date as
    /// the last entry.
    ///
    /// When using `push_gap()`, there is no need to explicitly add the
    /// first (non-leap-second) entry in the list.
    ///
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

    /// Add the expiry date to the list.
    ///
    /// The date must be the 28th of the month.
    ///
    /// This must be done last, before calling [`LeapSecBuilder::finish()`][]
    ///
    pub fn push_exp(&mut self, date: Gregorian) -> Result<()> {
        let month = month_of(date, EXPIRES_DATE)?;
        let last = self.last()?;
        let gap = month - last.month as i32;
        self.push_leap_sec(last, gap, Exp, month, None)
    }

    /// Add an entry to the list
    ///
    /// `date` must be the first of the month immediately after the leap
    /// second.
    ///
    /// `dtai` is the difference between UTC and TAI starting after the leap
    /// second at the given `date`. [`LeapSec::dtai()`][] discusses why DTAI
    /// values use `i16`.
    ///
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
