// fetch and parse the NIST leap-seconds.list

use anyhow::Context;
use std::convert::TryInto;
use std::io::Read;

use crate::*;

mod fmt;
mod parse;

pub use fmt::format;

const NIST_FILE: &str = "leap-seconds.list";
const NIST_URL: &str = "ftp://ftp.nist.gov/pub/time/leap-seconds.list";

pub fn read() -> anyhow::Result<LeapSecs> {
    Ok(read_bytes(&load_file(NIST_FILE).or_else(save_url)?)?)
}

pub fn read_bytes(data: &[u8]) -> Result<LeapSecs> {
    read_str(std::str::from_utf8(data)?)
}

pub fn read_file(name: &str) -> anyhow::Result<LeapSecs> {
    Ok(read_bytes(&load_file(name)?)?)
}

pub fn read_str(text: &str) -> Result<LeapSecs> {
    match parse::parse(&text) {
        Ok((_, unchecked)) => unchecked.try_into(),
        Err(nom::Err::Error(err)) => {
            Err(Error::Nom(nom::error::convert_error(text, err)))
        }
        Err(nom::Err::Failure(err)) => {
            Err(Error::Nom(nom::error::convert_error(text, err)))
        }
        _ => panic!(),
    }
}

pub fn read_url(url: &str) -> anyhow::Result<LeapSecs> {
    Ok(read_bytes(&load_url(url)?)?)
}

////////////////////////////////////////////////////////////////////////

// public for error reporting
#[derive(Clone, Debug, Default, Eq, PartialEq)]
pub struct Hash([u32; 5]);

// timestamp, DTAI, date
type UncheckedLeap = (i64, i16, Gregorian);

#[derive(Clone, Debug, Default)]
struct UncheckedList {
    pub updated: i64,
    pub expires: i64,
    pub leapsecs: Vec<UncheckedLeap>,
    pub hash: Hash,
}

fn save_url(_: anyhow::Error) -> anyhow::Result<Vec<u8>> {
    eprintln!("fetching {}", NIST_URL);
    let data = load_url(NIST_URL)?;
    std::fs::write(NIST_FILE, &data)
        .with_context(|| format!("failed to write {}", NIST_FILE))?;
    Ok(data)
}

fn load_file(name: &str) -> anyhow::Result<Vec<u8>> {
    let ctx = || format!("failed to read {}", name);
    let mut fh = std::fs::File::open(name).with_context(ctx)?;
    let mut data = Vec::new();
    fh.read_to_end(&mut data).with_context(ctx)?;
    Ok(data)
}

fn load_url(url: &str) -> anyhow::Result<Vec<u8>> {
    let mut data = Vec::new();
    curl_get(&url, &mut data)
        .with_context(|| format!("failed to fetch {}", &url))?;
    Ok(data)
}

fn curl_get(url: &str, buffer: &mut Vec<u8>) -> anyhow::Result<()> {
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
mod test {
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
