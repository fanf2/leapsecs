use std::convert::TryFrom;

use crate::date::*;
use crate::leapsecs::*;

impl std::str::FromStr for LeapSecs {
    type Err = Error;
    fn from_str(s: &str) -> Result<LeapSecs> {
        let mut list = vec![LeapSec::zero()];
        let mut month = mjd2month(list[0].mjd())?;
        let mut dtai = list[0].dtai();
        let (mut digits, mut gap) = (0, 0);
        for c in s.chars() {
            match (digits, c) {
                (0, '1'..='9') => {
                    digits = 1;
                    gap = c.to_digit(10).unwrap() as i32;
                }
                (1..=2, '0'..='9') => {
                    digits += 1;
                    gap = gap * 10 + c.to_digit(10).unwrap() as i32;
                }
                (1..=3, '-') => {
                    month += gap;
                    dtai -= 1;
                    list.push(LeapSec::month_neg(month, dtai));
                    digits = 0;
                    gap = 0;
                }
                (1..=3, '+') => {
                    month += gap;
                    dtai += 1;
                    list.push(LeapSec::month_pos(month, dtai));
                    digits = 0;
                    gap = 0;
                }
                (1..=3, '?') => {
                    month += gap;
                    list.push(LeapSec::month_exp(month));
                    digits = 0;
                    gap = 0;
                }
                (0, _) => return Err(Error::FromStr("[1-9]", c)),
                (1..=2, _) => return Err(Error::FromStr("[0-9?+-]", c)),
                (3, _) => return Err(Error::FromStr("[?+-]", c)),
                _ => panic!(),
            }
        }
        LeapSecs::try_from(list)
    }
}

impl std::fmt::Display for LeapSecs {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let list: &[LeapSec] = self.into();
        let mut prev = LeapSec::zero();
        for &next in list.iter() {
            let gap = months_between(prev.mjd(), next.mjd());
            match next {
                LeapSec::Zero { .. } => (),
                LeapSec::Neg { .. } => write!(f, "{}-", gap)?,
                LeapSec::Pos { .. } => write!(f, "{}+", gap)?,
                LeapSec::Exp { .. } => write!(f, "{}?", gap)?,
            }
            prev = next;
        }
        Ok(())
    }
}

#[cfg(test)]
mod test {
    use crate::leapsecs::*;
    use crate::nist;
    use std::str::FromStr;

    #[test]
    fn test() {
        let original = nist::read().expect("get leap-seconds.list");
        let output = format!("{}", original);
        let parsed = LeapSecs::from_str(&output).unwrap();
        assert_eq!(original, parsed);
        let input = "9+9-99+99-999+999?";
        let parsed = LeapSecs::from_str(&input).unwrap();
        let output = format!("{}", parsed);
        assert_eq!(input, output);
    }
}
