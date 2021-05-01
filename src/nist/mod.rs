// fetch and parse the NIST leap-seconds.list

pub mod parse;

use anyhow::{Context, Result};
use std::io::Read;

const NIST_FILE: &str = "leap-seconds.list";
const NIST_URL: &str = "ftp://ftp.nist.gov/pub/time/leap-seconds.list";

pub fn read() -> Result<Vec<u8>> {
    if let Ok(nist_text) = read_file(NIST_FILE) {
        Ok(nist_text)
    } else {
        let nist_text = read_url(NIST_URL)?;
        std::fs::write(NIST_FILE, &nist_text)?;
        Ok(nist_text)
    }
}

pub fn read_file(name: &str) -> Result<Vec<u8>> {
    let ctx = || format!("failed to read {}", name);
    let mut fh = std::fs::File::open(name).with_context(ctx)?;
    let mut buffer = Vec::new();
    fh.read_to_end(&mut buffer).with_context(ctx)?;
    Ok(buffer)
}

pub fn read_url(url: &str) -> Result<Vec<u8>> {
    let mut buffer = Vec::new();
    curl_get(&url, &mut buffer)
        .with_context(|| format!("failed to fetch {}", &url))?;
    Ok(buffer)
}

fn curl_get(url: &str, buffer: &mut Vec<u8>) -> Result<(), curl::Error> {
    let mut ua = curl::easy::Easy::new();
    ua.useragent("fanf/1.0")?;
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
