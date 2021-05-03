use std::convert::TryFrom;
use std::str::FromStr;

use crate::date::*;
use crate::leapsecs::*;

impl FromStr for LeapSecs {
    type Err = Error;
    fn from_str(s: &str) -> Result<LeapSecs> {
        let mut list = Vec::new();
        let mut month = mjd2month(i32::from(Gregorian(1972, 1, 1)))?;
        let mut dtai = 10;
        list.push(LeapSec::Zero { mjd: month2mjd(month), dtai });
        let (mut digits, mut gap) = (0, 0);
        for c in s.chars() {
            match (digits, c) {
                (0, '1'..='9') => {
                    digits = 1;
                    gap = c.to_digit(10).unwrap() as i32;
                }
                (1..=3, '0'..='9') => {
                    digits += 1;
                    gap = gap * 10 + c.to_digit(10).unwrap() as i32;
                }
                (1..=4, '-') => {
                    month += gap;
                    dtai -= 1;
                    list.push(LeapSec::Neg { mjd: month2mjd(month), dtai });
                    digits = 0;
                    gap = 0;
                }
                (1..=4, '+') => {
                    month += gap;
                    dtai += 1;
                    list.push(LeapSec::Pos { mjd: month2mjd(month), dtai });
                    digits = 0;
                    gap = 0;
                }
                (1..=4, '?') => {
                    month += gap;
                    list.push(LeapSec::Exp { mjd: month2mjd(month) });
                    digits = 0;
                    gap = 0;
                }
                (0, _) => return Err(Error::FromStr("[1-9]", c)),
                (1..=3, _) => return Err(Error::FromStr("[0-9?+-]", c)),
                (4, _) => return Err(Error::FromStr("[?+-]", c)),
                _ => panic!(),
            }
        }
        LeapSecs::try_from(list)
    }
}
