#![allow(dead_code)]
use anyhow::*;

mod date;
mod from;
mod leapsecs;
mod nist;
mod txt;

fn main() -> Result<()> {
    let list = nist::read()?;
    print!("{}", nist::format(&list, date::today())?);
    print!("{}", &list);
    Ok(())
}
