#![no_main]
use leapsecs::*;
use libfuzzer_sys::fuzz_target;
use std::convert::TryFrom;
use std::fmt::Write;
use std::str::FromStr;

fn fuzz_bin(data: &[u8]) {
    let parse1 = match LeapSecs::try_from(data) {
        Ok(parsed) => parsed,
        Err(Error::Empty) => return,
        Err(Error::Expired(_)) => return,
        Err(Error::FromInt(_)) => return,
        Err(Error::Gap(..)) => return,
        Err(Error::Truncated) => return,
        Err(err) => panic!("\ninput {:?}\nerror {}\n", data, err),
    };
    // the data is not going to be in canonical form, so we can't just
    // output the list in binary format and expect it to match, so
    // let's check a round-trip via text format
    let out1: &[u8] = &Vec::<u8>::from(&parse1);
    let text = format!("{}", parse1);
    let parse2 = LeapSecs::from_str(&text).unwrap();
    let out2: &[u8] = &Vec::<u8>::from(&parse2);
    assert_eq!(out1, out2);
}

fn fuzz_txt(data: &[u8]) {
    if data.len() < 1 {
        return;
    }
    let mut input = String::new();
    for &byte in &data[1..] {
        let sign = if byte < 128 { "-" } else { "+" };
        write!(input, "{}{}", byte % 128 + 1, sign).unwrap();
    }
    write!(input, "{}?", data[0] as u16 + 1).unwrap();
    let parsed = match LeapSecs::from_str(&input) {
        Ok(parsed) => parsed,
        Err(Error::Expired(_)) => return,
        Err(Error::FromInt(_)) => return,
        Err(e) => panic!("{}\n{}", input, e),
    };
    let output = format!("{}", parsed);
    assert_eq!(input, output);
}

fuzz_target!(|data: &[u8]| {
    if data.len() < 1 {
        return;
    }
    let rest = &data[1..];
    match data[0] {
        0 => fuzz_bin(rest),
        1 => fuzz_txt(rest),
        _ => (),
    }
});
