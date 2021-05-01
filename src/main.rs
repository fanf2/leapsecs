mod date;
mod nist;

use anyhow::*;

fn doit() -> Result<()> {
    let text = String::from_utf8(nist::read()?)?;
    let (_, unchecked) =
        nist::parse::parse(&text).map_err(|e| anyhow!("{}", e))?;
    dbg!(nist::check::check(unchecked)?);
    Ok(())
}

fn main() -> Result<()> {
    doit()
}
