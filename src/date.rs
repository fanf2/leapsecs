#[derive(Copy, Clone, Debug, Eq, Ord, PartialEq, PartialOrd)]
pub struct Gregorian(pub i32, pub i32, pub i32);

impl std::fmt::Display for Gregorian {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{:04}-{:02}-{:02}", self.year(), self.month(), self.day())
    }
}

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
    pub const fn mjd(self) -> MJD {
        let Gregorian(y, m, d) = self;
        let (y, m) = if m > 2 { (y, m + 1) } else { (y - 1, m + 13) };
        MJD(days_in_years(y) + muldiv(m, 153, 5) + d - 679004)
    }
}

impl From<MJD> for Gregorian {
    fn from(mjd: MJD) -> Gregorian {
        let mut d = mjd.0 + 678881;
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

impl From<Gregorian> for MJD {
    fn from(date: Gregorian) -> MJD {
        date.mjd()
    }
}

const fn days_in_years(y: i32) -> i32 {
    muldiv(y, 1461, 4) - muldiv(y, 1, 100) + muldiv(y, 1, 400)
}

const fn muldiv(var: i32, mul: i32, div: i32) -> i32 {
    (var * mul).div_euclid(div)
}

#[allow(clippy::upper_case_acronyms)]
#[derive(Copy, Clone, Eq, Ord, PartialEq, PartialOrd)]
pub struct MJD(i32);

impl std::fmt::Display for MJD {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{} mjd {}", Gregorian::from(*self), self.0)
    }
}

impl std::fmt::Debug for MJD {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "MJD({})", self)
    }
}

impl From<i32> for MJD {
    fn from(mjd: i32) -> MJD {
        MJD(mjd)
    }
}

impl std::ops::Add<i32> for MJD {
    type Output = MJD;
    fn add(self, days: i32) -> MJD {
        MJD(self.0 + days)
    }
}

impl std::ops::Sub<MJD> for MJD {
    type Output = i32;
    fn sub(self, other: MJD) -> i32 {
        self.0 - other.0
    }
}

pub fn today() -> MJD {
    use std::convert::TryFrom;
    use std::time::SystemTime;
    let now = SystemTime::now().duration_since(SystemTime::UNIX_EPOCH);
    // panic if we are in a tardis
    let days = now.unwrap().as_secs().div_euclid(86400);
    MJD::from(Gregorian(1970, 1, 1)) + i32::try_from(days).unwrap()
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
            let mjd = MJD::from(mjd);
            assert_eq!(date, Gregorian::from(mjd));
            assert_eq!(mjd, MJD::from(date));
        }
        assert_eq!(146097, days_in_years(400));
    }
}
