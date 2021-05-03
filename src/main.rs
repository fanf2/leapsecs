#![allow(dead_code)]
use anyhow::*;

mod date;
mod leapsecs;
mod nist;

fn main() -> Result<()> {
    print!("{}", nist::format(&nist::read()?, date::today())?);
    Ok(())
}
