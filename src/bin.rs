use crate::*;

const WIDE: u8 = 0x80;
const MONTH: u8 = 0x40;
const NEG: u8 = 0x20;
const POS: u8 = 0x10;
const LOW: u8 = 0x0F;

fn wide(nibble: u8) -> bool {
    nibble << 4 & WIDE != 0
}

// iterate over bytes one nibble at a time

struct Nibble<'a, T> {
    inner: &'a mut T,
    byte: Option<u8>,
}

impl<'a, T> Iterator for Nibble<'a, T>
where
    T: Iterator<Item = &'a u8>,
{
    type Item = u8;
    fn next(&mut self) -> Option<Self::Item> {
        if let Some(byte) = self.byte {
            self.byte = None;
            Some(byte)
        } else if let Some(&byte) = self.inner.next() {
            // bigendian
            self.byte = Some(byte & LOW);
            Some(byte >> 4)
        } else {
            None
        }
    }
}

// convert nibbles to bytecodes

struct Expand<'a, T>(&'a mut T);

impl<'a, T> Iterator for Expand<'a, T>
where
    T: Iterator<Item = u8>,
{
    type Item = u8;
    fn next(&mut self) -> Option<Self::Item> {
        match self.0.next() {
            None => None,
            Some(lo) if !wide(lo) => Some(POS | lo),
            Some(hi) => match self.0.next() {
                None => Some(hi << 4 | 4),
                Some(lo) => Some(hi << 4 | lo),
            },
        }
    }
}

impl std::convert::TryFrom<&[u8]> for LeapSecs {
    type Error = Error;
    fn try_from(slice: &[u8]) -> Result<LeapSecs> {
        let mut list = LeapSecs::builder();
        let mut bytes = slice.iter();
        let mut nibbles = Nibble { inner: &mut bytes, byte: None };
        for code in Expand(&mut nibbles) {
            let mul = if code & MONTH != 0 { 1 } else { 6 };
            let gap = (((code & LOW) + 1) * mul) as i32;
            let sign = match code & (NEG | POS) {
                NEG => Leap::Neg,
                POS => Leap::Pos,
                0 => Leap::Zero,
                _ => Leap::Exp,
            };
            list.push_gap(gap, sign)?;
        }
        list.finish()
    }
}

#[cfg(test)]
mod test {
    use crate::*;
    use std::convert::TryFrom;

    #[test]
    fn test() {
        let text = "6+6+12+12+12+12+12+12+12+18+12+12+24+30+24+\
                    12+18+12+12+18+18+18+84+36+42+36+18+59?";
        let binary: &[u8] = b"\x00\x11\x11\x11\x12\x11\x34\x31\
                              \x21\x12\x22\x9D\x56\x52\x87\xFA";
        let parsed = LeapSecs::try_from(binary).unwrap();
        let printed = format!("{}", parsed);
        assert_eq!(text, printed);
    }
}
