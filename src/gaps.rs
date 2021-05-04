use crate::date::*;
use crate::leapsecs::*;

pub(crate) struct Gap(pub i32, pub Leap);

impl std::convert::TryFrom<Vec<Gap>> for LeapSecs {
    type Error = Error;
    fn try_from(gaps: Vec<Gap>) -> Result<LeapSecs> {
        let mut list = vec![LeapSec::zero()];
        let mut month = mjd2month(list[0].mjd())?;
        let mut dtai = list[0].dtai();
        for Gap(gap, leap) in gaps {
            month += gap;
            let mjd = month2mjd(month);
            match leap {
                Leap::Zero => {
                    list.push(LeapSec::Zero { mjd, dtai });
                }
                Leap::Neg => {
                    dtai -= 1;
                    list.push(LeapSec::Neg { mjd, dtai });
                }
                Leap::Pos => {
                    dtai += 1;
                    list.push(LeapSec::Pos { mjd, dtai });
                }
                Leap::Exp => {
                    // NIST expiry date is 28th of the month
                    list.push(LeapSec::Exp { mjd: mjd + 27 });
                }
            }
        }
        LeapSecs::try_from(list)
    }
}
