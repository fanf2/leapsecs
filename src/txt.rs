use std::convert::TryFrom;

use crate::date::*;
use crate::gaps::*;
use crate::leapsecs::*;

impl std::str::FromStr for LeapSecs {
    type Err = Error;
    fn from_str(s: &str) -> Result<LeapSecs> {
        enum What {
            Zero,
            Digit(i32),
            Sign(Leap),
            Other,
        }
        use What::*;
        let mut gaps = Vec::new();
        let mut digits = 0;
        let mut gap = 0;
        for c in s.chars() {
            let what = match c {
                '0' => Zero,
                '1'..='9' => Digit(c.to_digit(10).unwrap() as i32),
                '-' => Sign(Leap::Neg),
                '+' => Sign(Leap::Pos),
                '?' => Sign(Leap::Exp),
                _ => Other,
            };
            match (digits, what) {
                (0..=2, Digit(n)) => {
                    digits += 1;
                    gap = gap * 10 + n;
                }
                (1..=2, Zero) => {
                    digits += 1;
                    gap *= 10;
                }
                (1..=3, Sign(leap)) => {
                    gaps.push(Gap(gap, leap));
                    digits = 0;
                    gap = 0;
                }
                (0, _) => return Err(Error::FromStr("[1-9]", c)),
                (1..=2, _) => return Err(Error::FromStr("[0-9?+-]", c)),
                (3, _) => return Err(Error::FromStr("[?+-]", c)),
                _ => panic!(), // screwed up counting digits
            };
        }
        if digits != 0 {
            gaps.push(Gap(gap, Leap::Zero));
        }
        LeapSecs::try_from(gaps)
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
