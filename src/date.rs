use crate::leapsecs::{Error, Result};

#[derive(Copy, Clone, Debug, Eq, Ord, PartialEq, PartialOrd)]
pub struct Gregorian(pub i32, pub i32, pub i32);

impl Gregorian {
    pub fn year(self) -> i32 {
        self.0
    }
    pub fn month(self) -> i32 {
        self.1
    }
    pub fn day(self) -> i32 {
        self.2
    }
}

impl std::fmt::Display for Gregorian {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{:04}-{:02}-{:02}", self.year(), self.month(), self.day(),)
    }
}

// for error reporting
#[allow(clippy::upper_case_acronyms)]
#[derive(Debug, Eq, PartialEq)]
pub struct MJD(pub i32);

impl std::fmt::Display for MJD {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{} mjd {}", Gregorian::from(self.0), self.0)
    }
}

// for error reporting
#[allow(clippy::upper_case_acronyms)]
#[derive(Debug, Eq, PartialEq)]
pub struct NTP(pub i64);

impl std::fmt::Display for NTP {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        if let Ok(mjd) = ntp2mjd(self.0) {
            write!(f, "{} ntp {}", MJD(mjd), self.0)
        } else {
            write!(f, "{}", self.0)
        }
    }
}

fn muldiv(var: i32, mul: i32, div: i32) -> i32 {
    use std::ops::Mul;
    var.mul(mul).div_euclid(div)
}

fn days_in_years(y: i32) -> i32 {
    muldiv(y, 1461, 4) - muldiv(y, 1, 100) + muldiv(y, 1, 400)
}

impl From<Gregorian> for i32 {
    fn from(Gregorian(y, m, d): Gregorian) -> i32 {
        let (y, m) = if m > 2 { (y, m + 1) } else { (y - 1, m + 13) };
        days_in_years(y) + muldiv(m, 153, 5) + d - 679004
    }
}

impl From<i32> for Gregorian {
    fn from(mjd: i32) -> Gregorian {
        let mut d = mjd + 678881;
        let mut y = muldiv(d, 400, 146097) + 1;
        y -= (days_in_years(y) > d) as i32;
        d -= days_in_years(y) - 31;
        let m = muldiv(d, 17, 520);
        d -= muldiv(m, 520, 17);
        if m > 10 {
            Gregorian(y + 1, m - 10, d)
        } else {
            Gregorian(y, m + 2, d)
        }
    }
}

pub fn today() -> i32 {
    use std::time::SystemTime;
    let now = SystemTime::now().duration_since(SystemTime::UNIX_EPOCH);
    // panic if we are in a tardis
    let unix_date = now.unwrap().as_secs().div_euclid(86400);
    i32::from(Gregorian(1970, 1, 1)) + unix_date as i32
}

pub fn mjd2ntp(mjd: i32) -> i64 {
    86400 * (mjd - i32::from(Gregorian(1900, 1, 1))) as i64
}

pub fn month2mjd(month: i32) -> i32 {
    let year = month.div_euclid(12);
    let month = month.rem_euclid(12);
    i32::from(Gregorian(year, month + 1, 1))
}

pub fn ntp2mjd(ntp: i64) -> Result<i32> {
    let days = ntp.div_euclid(86400) as i32;
    let secs = ntp.rem_euclid(86400) as i32;
    let mjd = i32::from(Gregorian(1900, 1, 1)) + days;
    if secs == 0 {
        Ok(mjd)
    } else {
        Err(Error::Midnight(NTP(ntp)))
    }
}

pub fn mjd2month(mjd: i32) -> Result<i32> {
    let date = Gregorian::from(mjd);
    if date.day() == 1 {
        Ok(date.year() * 12 + date.month() - 1)
    } else {
        Err(Error::MonthFirst(MJD(mjd)))
    }
}

pub fn months_between(prev: i32, next: i32) -> i32 {
    let prev = Gregorian::from(prev);
    let next = Gregorian::from(next);
    (next.year() * 12 + next.month() - 1)
        - (prev.year() * 12 + prev.month() - 1)
}

#[cfg(test)]
mod test {
    use super::*;

    #[test]
    fn test() {
        for &(date, mjd) in &[
            (Gregorian(-1, 12, 31), -678942),
            (Gregorian(0, 1, 1), -678941),
            (Gregorian(0, 12, 31), -678576),
            (Gregorian(1, 1, 1), -678575),
            (Gregorian(1858, 11, 16), -1),
            (Gregorian(1858, 11, 17), 0),
            (Gregorian(1900, 1, 1), 15020),
            (Gregorian(1970, 1, 1), 40587),
            (Gregorian(2001, 1, 1), 5 * 146097 - 678575),
            (Gregorian(2020, 2, 2), 58881),
        ] {
            assert_eq!(date, Gregorian::from(mjd));
            assert_eq!(mjd, i32::from(date));
        }
        assert_eq!(146097, days_in_years(400));
    }
}
