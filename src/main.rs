#![allow(dead_code)]
use anyhow::*;

mod date;
mod from;
mod leapsecs;
mod nist;
mod txt;

fn main() -> Result<()> {
    print!("{}", nist::format(&nist::read()?, date::today())?);
    Ok(())
}
