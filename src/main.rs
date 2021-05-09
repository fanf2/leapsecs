use leapsecs::*;

fn main() -> anyhow::Result<()> {
    let list = nist::read()?;
    println!("{}", nist::format(&list, date::today())?);
    println!("{}", &list);
    println!("{:X}", &list);
    Ok(())
}
