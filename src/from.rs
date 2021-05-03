use std::convert::TryFrom;

use crate::date::*;
use crate::leapsecs::*;

#[derive(Clone, Debug, Default, Eq, PartialEq)]
pub struct LeapSecs(Vec<LeapSec>);

impl From<LeapSecs> for Vec<LeapSec> {
    fn from(s: LeapSecs) -> Vec<LeapSec> {
        s.0
    }
}

impl<'a> From<&'a LeapSecs> for &'a [LeapSec] {
    fn from(s: &'a LeapSecs) -> &'a [LeapSec] {
        &s.0
    }
}

impl TryFrom<Vec<LeapSec>> for LeapSecs {
    type Error = Error;

    fn try_from(list: Vec<LeapSec>) -> Result<LeapSecs> {
        if list.len() < 2 {
            return Err(Error::Empty());
        }
        let last = list.len() - 1;

        let mut prev = LeapSec::zero();
        for (i, &this) in list.iter().enumerate() {
            match this {
                LeapSec::Zero { .. } => {
                    let _ = mjd2month(this.mjd())?;
                    if i != 0 {
                        return Err(Error::OutOfOrder(prev, this));
                    }
                    if prev != this {
                        return Err(Error::FalseStart(this));
                    }
                }

                LeapSec::Neg { .. } => {
                    let _ = mjd2month(this.mjd())?;
                    if i == 0 || i == last || this.mjd() <= prev.mjd() {
                        return Err(Error::OutOfOrder(prev, this));
                    }
                    if this.dtai() != prev.dtai() - 1 {
                        return Err(Error::WrongNeg(prev, this));
                    }
                    prev = this;
                }

                LeapSec::Pos { .. } => {
                    let _ = mjd2month(this.mjd())?;
                    if i == 0 || i == last || this.mjd() <= prev.mjd() {
                        return Err(Error::OutOfOrder(prev, this));
                    }
                    if this.dtai() != prev.dtai() + 1 {
                        return Err(Error::WrongPos(prev, this));
                    }
                    prev = this;
                }

                LeapSec::Exp { .. } => {
                    if i != last || this.mjd() <= prev.mjd() {
                        return Err(Error::OutOfOrder(prev, this));
                    }
                    if this.mjd() <= today() {
                        return Err(Error::Expired(this));
                    }
                }
            }
        }

        Ok(LeapSecs(list))
    }
}
