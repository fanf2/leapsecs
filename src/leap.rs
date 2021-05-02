// https://www.ucolick.org/~sla/leapsecs/dutc.html
//
// Before the year 4000 we expect there will be more than one leap
// second each month, at which point UTC as currently defined will no
// longer work. At that time DTAI is expected to be less than 4 hours,
// i.e. 14,400 seconds, which is less than 2^15.

pub type LeapSecs = Vec<LeapSecond>;

#[derive(Copy, Clone, Debug, Eq, PartialEq)]
pub enum LeapSecond {
    Zero { mjd: i32, dtai: i16 },
    Neg { mjd: i32, dtai: i16 },
    Pos { mjd: i32, dtai: i16 },
    Exp { mjd: i32 },
}

impl LeapSecond {
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
}
