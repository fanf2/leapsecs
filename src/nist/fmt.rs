use std::fmt::Write;

use super::Error;
use crate::date::*;
use crate::leap::*;

pub fn format(list: &LeapSecs, up_mjd: i32) -> Result<String, Error> {
    let mut out = String::new();
    let updated = mjd2ntp(up_mjd);
    let expires = mjd2ntp(list.last().unwrap().mjd());
    write!(out, "#$\t{}\n", updated)?;
    write!(out, "#@\t{}\n", expires)?;
    for leap in list.iter().take(list.len() - 1) {
        write!(out, "{}\t{}\t", mjd2ntp(leap.mjd()), leap.dtai())?;
        let date = Gregorian::from(leap.mjd());
        let month = [
            "Jan", "Feb", "Mar", "Apr", "May", "Jun", "Jul", "Aug", "Sep",
            "Oct", "Nov", "Dec",
        ][(date.month() - 1) as usize];
        write!(out, "# {} {} {}\n", date.day(), month, date.year())?;
    }
    let hash = super::hash::hash(&list, up_mjd)?;
    write!(
        out,
        "#h\t{:08x} {:08x} {:08x} {:08x} {:08x}\n",
        hash[0], hash[1], hash[2], hash[3], hash[4],
    )?;
    Ok(out)
}
