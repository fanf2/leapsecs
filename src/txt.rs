//! Compact text format for the leap second list
//! ============================================
//!
//! This module implements a number of standard traits for the
//! [`LeapSecs`][] type:
//!
//!   * [`std::str::FromStr`][] parses a leap second list in compact
//!     text format, returning
//!     `Result<`[`LeapSecs`][crate::LeapSecs]`, `[`Error`][enum@Error]`>`.
//!
//!   * [`std::fmt::Display`][] prints a leap second list in compact
//!     text format.
//!
//!   * [`std::fmt::LowerHex`][] and [`std::fmt::UpperHex`][] print a
//!     hexdump of a leap second list in compact binary format. There
//!     is no parser for the opposite conversion.

use crate::*;

impl std::str::FromStr for LeapSecs {
    type Err = Error;

    fn from_str(s: &str) -> Result<LeapSecs> {
        let mut list = LeapSecs::builder();
        let mut digits = 0;
        let mut gap = 0;
        for c in s.chars() {
            enum What {
                Zero,
                Digit(i32),
                Sign(Leap),
                Other,
            }
            use What::*;

            let what = match c {
                '0' => Zero,
                '1'..='9' => Digit(c as i32 - '0' as i32),
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
                (1..=3, Sign(sign)) => {
                    list.push_gap(gap, sign)?;
                    digits = 0;
                    gap = 0;
                }
                (0, _) => return Err(Error::FromStr("[1-9]", c)),
                (1..=2, _) => return Err(Error::FromStr("[0-9?+-]", c)),
                (3, _) => return Err(Error::FromStr("[?+-]", c)),
                _ => panic!("screwed up counting digits"),
            };
        }

        if digits != 0 {
            Err(Error::Truncated)
        } else {
            list.finish()
        }
    }
}

impl std::fmt::Display for LeapSecs {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        for leap in self {
            match leap.sign() {
                Leap::Zero => (),
                Leap::Neg => write!(f, "{}-", leap.gap())?,
                Leap::Pos => write!(f, "{}+", leap.gap())?,
                Leap::Exp => write!(f, "{}?", leap.gap())?,
            }
        }
        Ok(())
    }
}

impl std::fmt::LowerHex for LeapSecs {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        for byte in self.iter_bytes() {
            write!(f, "{:02x}", byte)?
        }
        Ok(())
    }
}

impl std::fmt::UpperHex for LeapSecs {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        for byte in self.iter_bytes() {
            write!(f, "{:02X}", byte)?
        }
        Ok(())
    }
}

#[cfg(test)]
mod test {
    use crate::*;
    use std::str::FromStr;

    #[test]
    fn test() {
        let text = "6+6+12+12+12+12+12+12+12+18+12+12+24+30+24+\
                    12+18+12+12+18+18+18+84+36+42+36+18+59?";
        let parsed = LeapSecs::from_str(text).unwrap();
        let output = format!("{}", parsed);
        assert_eq!(text, output);
        let input = "9+9-99+99-999+999?";
        let parsed = LeapSecs::from_str(&input).unwrap();
        let output = format!("{}", parsed);
        assert_eq!(input, output);
    }
}
