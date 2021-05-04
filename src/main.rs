#![allow(dead_code)]
use anyhow::*;

mod bin;
mod date;
mod from;
mod gaps;
mod leapsecs;
mod nist;
mod txt;

fn main() -> Result<()> {
    let list = nist::read()?;
    print!("{}", nist::format(&list, date::today())?);
    print!("{}", &list);
    Ok(())
}
