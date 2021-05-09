use ring::digest::*;
use std::convert::{TryFrom, TryInto};
use std::fmt::Write;

use super::Hash;
use crate::*;

impl std::fmt::Display for Hash {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let [h0, h1, h2, h3, h4] = self.0;
        write!(f, "{:08x} {:08x} {:08x} {:08x} {:08x}", h0, h1, h2, h3, h4)
    }
}

const NTP_EPOCH: MJD = Gregorian(1900, 1, 1).mjd();

fn ntp_from(mjd: MJD) -> i64 {
    (mjd - NTP_EPOCH) as i64 * 86400
}

fn mjd_from(ntp: i64) -> Result<MJD> {
    let days = i32::try_from(ntp.div_euclid(86400))?;
    let secs = i32::try_from(ntp.rem_euclid(86400))?;
    let mjd = NTP_EPOCH + days;
    if secs != 0 {
        Err(Error::Midnight(ntp, mjd, secs))
    } else {
        Ok(mjd)
    }
}

pub fn format(list: &LeapSecs, updated_mjd: MJD) -> Result<String> {
    let mut out = String::new();
    let expires_mjd = list.expires();
    let updated_date = Gregorian::from(updated_mjd);
    let expires_date = Gregorian::from(expires_mjd);
    let updated_ntp = ntp_from(updated_mjd);
    let expires_ntp = ntp_from(expires_mjd);
    write!(out, "#\tupdated {}\n#$\t{}\n#\n", updated_date, updated_ntp)?;
    write!(out, "#\texpires {}\n#@\t{}\n#\n", expires_date, expires_ntp)?;
    for &leap in list.iter().take(list.len() - 1) {
        let date = Gregorian::from(leap.mjd());
        let month = [
            "Jan", "Feb", "Mar", "Apr", "May", "Jun", "Jul", "Aug", "Sep",
            "Oct", "Nov", "Dec",
        ][(date.month() - 1) as usize];
        writeln!(
            out,
            "{}\t{}\t# {} {} {}",
            ntp_from(leap.mjd()),
            leap.dtai().unwrap(),
            date.day(),
            month,
            date.year()
        )?;
    }
    let hash = sha1(&hashin(list, updated_ntp)?);
    write!(out, "#\n#h\t{}\n", hash)?;
    Ok(out)
}

impl TryFrom<super::UncheckedList> for LeapSecs {
    type Error = Error;
    fn try_from(u: super::UncheckedList) -> Result<LeapSecs> {
        let mut list = LeapSecs::builder();
        for (ntp, dtai, date) in u.leapsecs {
            let mjd = mjd_from(ntp)?;
            if mjd != MJD::from(date) {
                return Err(Error::TimeDate(ntp, mjd, date));
            } else {
                list.push_date(date, dtai)?
            }
        }
        let _check = mjd_from(u.updated)?;
        let expires = mjd_from(u.expires)?;
        list.push_exp(Gregorian::from(expires))?;
        let list = list.finish()?;
        let hashin = hashin(&list, u.updated)?;
        let calculated = sha1(&hashin);
        if u.hash != calculated {
            Err(Error::Checksum(u.hash, calculated, hashin))
        } else {
            Ok(list)
        }
    }
}

fn hashin(list: &LeapSecs, updated: i64) -> Result<String> {
    let expires = ntp_from(list.expires());
    let mut hashin = String::new();
    write!(hashin, "{}{}", updated, expires)?;
    for leap in list.iter().take(list.len() - 1) {
        write!(hashin, "{}{}", ntp_from(leap.mjd()), leap.dtai().unwrap())?;
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
