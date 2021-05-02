// fetch and parse the NIST leap-seconds.list

use anyhow::*;
use std::io::Read;
use thiserror::Error;

use crate::date::*;
use crate::leap::*;

mod check;
mod parse;

const NIST_FILE: &str = "leap-seconds.list";
const NIST_URL: &str = "ftp://ftp.nist.gov/pub/time/leap-seconds.list";

#[derive(Error, Debug)]
pub enum Error {
    #[error("checksum failed {0:?} <> {1:?} data {2}")]
    Checksum([u8; 20], [u8; 20], String),
    #[error("leap seconds list is empty (published {0:?}")]
    Empty(TimeStamp),
    #[error("leap seconds file has expired ({0:?})")]
    Expired(TimeStamp),
    #[error("starts with {1} seconds at {0:?}")]
    FalseStart(TimeStamp, u16),
    #[error("format error {0}")]
    Format(#[from] std::fmt::Error),
    #[error("timestamp is not midnight ({0:?})")]
    Fractional(TimeStamp),
    #[error("leap more than one second ({1} -> {2} at {0:?})")]
    LargeLeap(TimeStamp, u16, u16),
    #[error("timestamp and date do not match ({0:?} <> {1:?})")]
    Mismatch(TimeStamp, Gregorian),
    #[error("lack of leap ({1} at {0:?})")]
    NoLeap(TimeStamp, u16),
    #[error("leap seconds are disordered ({0:?} > {1:?})")]
    OutOfOrder(TimeStamp, TimeStamp),
    #[error("DTAI is too large ({0:?})")]
    Spinny(TimeStamp, u64),
    #[error("leap second is after expiry time ({0:?})")]
    TooLate(TimeStamp),
    #[error("timestamp is before 1972 ({0:?})")]
    TooSoon(TimeStamp),
}

// just for error reporting
#[derive(Debug, Eq, PartialEq)]
pub struct TimeStamp {
    ntp: u64,
    mjd: i32,
    date: Gregorian,
}

pub fn read() -> Result<LeapSecs> {
    read_bytes(&load_file(NIST_FILE).or(save_url())?)
}

pub fn read_bytes(data: &[u8]) -> Result<LeapSecs> {
    read_str(std::str::from_utf8(data)?)
}

pub fn read_file(name: &str) -> Result<LeapSecs> {
    read_bytes(&load_file(name)?)
}

pub fn read_str(text: &str) -> Result<LeapSecs> {
    let (_, unchecked) = parse::parse(&text).map_err(|e| anyhow!("{}", e))?;
    Ok(check::check(unchecked)?)
}

pub fn read_url(url: &str) -> Result<LeapSecs> {
    read_bytes(&load_url(url)?)
}

////////////////////////////////////////////////////////////////////////

// timestamp, DTAI, date
type UncheckedLeap = (u64, u64, Gregorian);

#[derive(Clone, Debug, Default)]
struct UncheckedNIST {
    pub updated: u64,
    pub expires: u64,
    pub leapsecs: Vec<UncheckedLeap>,
    pub hash: [u8; 20],
}

fn save_url() -> Result<Vec<u8>> {
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
