// fetch and parse the NIST leap-seconds.list

use anyhow::*;
use std::io::Read;
use thiserror::Error;

use crate::date::*;
use crate::leap::*;

mod check;
mod fmt;
mod parse;

pub use fmt::format;

const NIST_FILE: &str = "leap-seconds.list";
const NIST_URL: &str = "ftp://ftp.nist.gov/pub/time/leap-seconds.list";

#[derive(Error, Debug)]
pub enum Error {
    #[error("checksum failed {0:?} <> {1:?} data {2}")]
    Checksum(Hash, Hash, String),
    #[error("leap seconds list is empty (published {0:?}")]
    Empty(TimeStamp),
    #[error("leap seconds file has expired ({0:?})")]
    Expired(TimeStamp),
    #[error("starts with {1} seconds at {0:?}")]
    FalseStart(TimeStamp, i16),
    #[error("format error {0}")]
    Format(#[from] std::fmt::Error),
    #[error("timestamp is not midnight ({0:?})")]
    Fractional(TimeStamp),
    #[error("leap more than one second ({1} -> {2} at {0:?})")]
    LargeLeap(TimeStamp, i16, i16),
    #[error("timestamp and date do not match ({0:?} <> {1:?})")]
    Mismatch(TimeStamp, Gregorian),
    #[error("lack of leap ({1} at {0:?})")]
    NoLeap(TimeStamp, i16),
    #[error("leap seconds are disordered ({0:?} > {1:?})")]
    OutOfOrder(TimeStamp, TimeStamp),
    #[error("DTAI is too large ({0:?})")]
    Spinny(TimeStamp, i64),
    #[error("leap second is after expiry time ({0:?})")]
    TooLate(TimeStamp),
    #[error("timestamp is before 1972 ({0:?})")]
    TooSoon(TimeStamp),
}

// just for error reporting
#[derive(Debug, Eq, PartialEq)]
pub struct TimeStamp {
    ntp: i64,
    mjd: i32,
    date: Gregorian,
}

pub type Hash = [u32; 5];

pub fn read() -> Result<Vec<LeapSec>> {
    read_bytes(&load_file(NIST_FILE).or_else(save_url)?)
}

pub fn read_bytes(data: &[u8]) -> Result<Vec<LeapSec>> {
    read_str(std::str::from_utf8(data)?)
}

pub fn read_file(name: &str) -> Result<Vec<LeapSec>> {
    read_bytes(&load_file(name)?)
}

pub fn read_str(text: &str) -> Result<Vec<LeapSec>> {
    let (_, unchecked) = parse::parse(&text).map_err(|e| anyhow!("{}", e))?;
    Ok(check::check(unchecked)?)
}

pub fn read_url(url: &str) -> Result<Vec<LeapSec>> {
    read_bytes(&load_url(url)?)
}

////////////////////////////////////////////////////////////////////////

// timestamp, DTAI, date
type UncheckedLeap = (i64, i64, Gregorian);

#[derive(Clone, Debug, Default)]
struct UncheckedList {
    pub updated: i64,
    pub expires: i64,
    pub leapsecs: Vec<UncheckedLeap>,
    pub hash: Hash,
}

fn save_url(_: anyhow::Error) -> Result<Vec<u8>> {
    eprintln!("fetching {}", NIST_URL);
    let data = load_url(NIST_URL)?;
    std::fs::write(NIST_FILE, &data)
        .with_context(|| format!("failed to write {}", NIST_FILE))?;
    Ok(data)
}

fn load_file(name: &str) -> Result<Vec<u8>> {
    let ctx = || format!("failed to read {}", name);
    let mut fh = std::fs::File::open(name).with_context(ctx)?;
    let mut data = Vec::new();
    fh.read_to_end(&mut data).with_context(ctx)?;
    Ok(data)
}

fn load_url(url: &str) -> Result<Vec<u8>> {
    let mut data = Vec::new();
    curl_get(&url, &mut data)
        .with_context(|| format!("failed to fetch {}", &url))?;
    Ok(data)
}

fn curl_get(url: &str, buffer: &mut Vec<u8>) -> Result<()> {
    let mut ua = curl::easy::Easy::new();
    ua.useragent(&format!(
        "leapsecs/0 curl/{}",
        curl::Version::get().version()
    ))?;
    ua.fail_on_error(true)?;
    ua.url(url)?;
    let mut xfer = ua.transfer();
    xfer.write_function(|chunk| {
        buffer.extend_from_slice(chunk);
        Ok(chunk.len())
    })?;
    xfer.perform()?;
    Ok(())
}

////////////////////////////////////////////////////////////////////////

#[cfg(test)]
mod tests {
    use crate::date;
    use crate::nist;

    #[test]
    fn test() {
        let original = nist::read().expect("get leap-seconds.list");
        let printed = nist::format(&original, date::today())
            .expect("formatting leap seconds");
        let parsed = nist::read_str(&printed).expect("re-parsing leap-seconds");
        assert_eq!(original, parsed);
    }
}
