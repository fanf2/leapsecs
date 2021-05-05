use leapsecs::*;

fn main() -> anyhow::Result<()> {
    let list = nist::read()?;
    print!("{}", nist::format(&list, date::today())?);
    print!("{}", &list);
    Ok(())
}
