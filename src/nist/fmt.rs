use ring::digest::*;
use std::convert::TryInto;
use std::fmt::Write;

use super::Hash;
use crate::date::*;
use crate::leapsecs::*;

impl std::fmt::Display for Hash {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let [h0, h1, h2, h3, h4] = self.0;
        write!(f, "{:08x} {:08x} {:08x} {:08x} {:08x}", h0, h1, h2, h3, h4)
    }
}

pub fn format(list: &LeapSecs, updated_mjd: i32) -> Result<String> {
    let list: &[LeapSec] = list.into();
    let mut out = String::new();
    let expires_mjd = list.last().unwrap().mjd();
    let updated_date = Gregorian::from(updated_mjd);
    let expires_date = Gregorian::from(expires_mjd);
    let updated_ntp = mjd2ntp(updated_mjd);
    let expires_ntp = mjd2ntp(expires_mjd);
    write!(out, "#\tupdated {}\n#$\t{}\n#\n", updated_date, updated_ntp)?;
    write!(out, "#\texpires {}\n#@\t{}\n#\n", expires_date, expires_ntp)?;
    for leap in list.iter().take(list.len() - 1) {
        let date = Gregorian::from(leap.mjd());
        let month = [
            "Jan", "Feb", "Mar", "Apr", "May", "Jun", "Jul", "Aug", "Sep",
            "Oct", "Nov", "Dec",
        ][(date.month() - 1) as usize];
        writeln!(
            out,
            "{}\t{}\t# {} {} {}",
            mjd2ntp(leap.mjd()),
            leap.dtai(),
            date.day(),
            month,
            date.year()
        )?;
    }
    let hash = sha1(&hashin(list, updated_ntp)?);
    write!(out, "#\n#h\t{}\n", hash)?;
    Ok(out)
}

pub(super) fn check(u: super::UncheckedList) -> Result<LeapSecs> {
    let mut prev = LeapSec::zero().dtai();
    let mut list = Vec::new();
    for &(ntp, dtai, date) in u.leapsecs.iter() {
        let mjd = ntp2mjd(ntp)?;
        if mjd != i32::from(date) {
            return Err(Error::TimeDate(NTP(ntp), date));
        } else if dtai == prev {
            list.push(LeapSec::Zero { mjd, dtai });
        } else if dtai < prev {
            list.push(LeapSec::Neg { mjd, dtai });
        } else if dtai > prev {
            list.push(LeapSec::Pos { mjd, dtai });
        }
        prev = dtai;
    }
    let _check = ntp2mjd(u.updated)?;
    let expires = ntp2mjd(u.expires)?;
    list.push(LeapSec::Exp { mjd: expires });
    let hashin = hashin(&list, u.updated)?;
    let calculated = sha1(&hashin);
    if u.hash != calculated {
        Err(Error::Checksum(u.hash, calculated, hashin))
    } else {
        list.try_into()
    }
}

fn hashin(list: &[LeapSec], updated: i64) -> Result<String> {
    let expires = mjd2ntp(list.last().unwrap().mjd());
    let mut hashin = String::new();
    write!(hashin, "{}{}", updated, expires)?;
    for leap in list.iter().take(list.len() - 1) {
        write!(hashin, "{}{}", mjd2ntp(leap.mjd()), leap.dtai())?;
    }
    Ok(hashin)
}

fn sha1(input: &str) -> Hash {
    let hash = digest(&SHA1_FOR_LEGACY_USE_ONLY, input.as_bytes());
    // panic if sha1 is not the standard size
    let hash8: [u8; 20] = hash.as_ref().try_into().unwrap();
    let mut hash32: Hash = Default::default();
    for i in 0..5 {
        let word: [u8; 4] = hash8[i * 4..i * 4 + 4].try_into().unwrap();
        hash32.0[i] = u32::from_be_bytes(word);
    }
    hash32
}
