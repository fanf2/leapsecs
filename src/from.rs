use std::convert::TryFrom;

use crate::date::*;
use crate::leapsecs::*;

#[derive(Clone, Debug, Default, Eq, PartialEq)]
pub struct LeapSecs(Vec<LeapSec>);

impl<'a> From<&'a LeapSecs> for &'a [LeapSec] {
    fn from(s: &'a LeapSecs) -> &'a [LeapSec] {
        &s.0
    }
}

impl TryFrom<Vec<LeapSec>> for LeapSecs {
    type Error = Error;

    fn try_from(list: Vec<LeapSec>) -> Result<LeapSecs> {
        if list.len() < 2 {
            return Err(Error::Empty);
        }
        let last = list.len() - 1;
        let mut prev = LeapSec::zero();
        for (i, &next) in list.iter().enumerate() {
            let inorder = |always| {
                if always || next.mjd() <= prev.mjd() {
                    Err(Error::OutOfOrder(prev, next))
                } else {
                    Ok(())
                }
            };
            let leap = |err: fn(LeapSec, LeapSec) -> Error, sign| {
                if next.dtai() != prev.dtai() + sign {
                    Err(err(prev, next))
                } else {
                    Ok(())
                }
            };
            match next {
                LeapSec::Zero { .. } => {
                    let _ = mjd2month(next.mjd())?;
                    if i != 0 {
                        return Err(Error::OutOfOrder(prev, next));
                    }
                    if prev != next {
                        return Err(Error::FalseStart(next));
                    }
                }
                LeapSec::Neg { .. } => {
                    let _ = mjd2month(next.mjd())?;
                    inorder(i == 0 || i == last)?;
                    leap(Error::WrongNeg, -1)?;
                    prev = next;
                }
                LeapSec::Pos { .. } => {
                    let _ = mjd2month(next.mjd())?;
                    inorder(i == 0 || i == last)?;
                    leap(Error::WrongPos, 1)?;
                    prev = next;
                }
                LeapSec::Exp { .. } => {
                    inorder(i != last)?;
                    if next.mjd() <= today() {
                        return Err(Error::Expired(next));
                    }
                }
            }
        }
        Ok(LeapSecs(list))
    }
}
