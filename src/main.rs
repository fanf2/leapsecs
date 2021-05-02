use anyhow::*;

mod date;
mod leap;
mod nist;

fn main() -> Result<()> {
    dbg!(nist::read()?);
    Ok(())
}
