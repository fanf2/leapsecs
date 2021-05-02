#![allow(dead_code)]
use anyhow::*;

mod date;
mod leap;
mod nist;

fn main() -> Result<()> {
    print!("{}", nist::format(&nist::read()?, date::today())?);
    Ok(())
}
