use crate::*;
use std::result::Result;

const WIDE: u8 = 0x80;
const MONTH: u8 = 0x40;
const NEG: u8 = 0x20;
const POS: u8 = 0x10;
const FLAGS: u8 = 0xF0;
const LOW: u8 = 0x0F;

fn wide(nibble: u8) -> bool {
    nibble << 4 & WIDE != 0
}

// iterate over bytes one nibble at a time

struct Nibbles<'a> {
    inner: std::slice::Iter<'a, u8>,
    byte: Option<u8>,
}

impl<'a> Iterator for Nibbles<'a> {
    type Item = u8;
    fn next(&mut self) -> Option<u8> {
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

struct Expand<'a>(Nibbles<'a>);

impl<'a> Iterator for Expand<'a> {
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
    fn try_from(slice: &[u8]) -> Result<LeapSecs, Error> {
        let mut list = LeapSecs::builder();
        let bytes = slice.iter();
        let nibbles = Nibbles { inner: bytes, byte: None };
        for code in Expand(nibbles) {
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

impl LeapSecs {
    fn widecodes<F, E>(&self, mut emit: F) -> Result<(), E>
    where
        F: FnMut(u8) -> Result<(), E>,
    {
        for leap in self {
            let flags = match leap.sign() {
                Leap::Zero => continue,
                Leap::Neg => WIDE | NEG,
                Leap::Pos => WIDE | POS,
                Leap::Exp => WIDE | NEG | POS,
            };
            let mut gap = leap.gap();
            if gap % 6 == 0 {
                while gap >= 16 * 6 {
                    emit(WIDE | 15)?;
                    gap -= 16 * 6;
                }
                let gap = gap as u8 / 6 - 1;
                emit(flags | gap)?;
            } else if gap <= 16 {
                let gap = gap as u8 - 1;
                emit(flags | MONTH | gap)?;
            } else {
                while gap >= 16 * 6 {
                    emit(WIDE | 15)?;
                    gap -= 16 * 6;
                }
                let years = gap / 12;
                let months = gap % 12;
                if years > 0 {
                    let gap = years as u8 * 2 - 1;
                    emit(WIDE | gap)?;
                }
                let gap = months as u8 - 1;
                emit(flags | MONTH | gap)?;
            }
        }
        Ok(())
    }

    pub fn for_each_byte<F, E>(&self, mut emit: F) -> Result<usize, E>
    where
        F: FnMut(u8) -> Result<(), E>,
    {
        let mut last_nibble = 0;
        let mut nibble_count = 0;
        let mut expire_five = false;

        // first pass: calculate length and work out
        // how we will round to a whole number of bytes

        self.widecodes(|code| {
            if code == FLAGS | 4 {
                expire_five = true;
                nibble_count += 2;
            } else if code & FLAGS != WIDE | POS || wide(code & LOW) {
                nibble_count += 2;
            } else {
                nibble_count += 1;
                last_nibble = nibble_count;
            }
            Ok(())
        })?;

        if nibble_count % 2 == 0 {
            last_nibble = 0;
            expire_five = false;
        } else if expire_five {
            last_nibble = 0;
            nibble_count -= 1;
        } else {
            nibble_count += 1;
        }

        // second pass: actually write the output

        let expected_count = nibble_count;
        nibble_count = 0;
        let mut prev = None;

        self.widecodes(|code| {
            let (this, next) = if code & FLAGS != WIDE | POS
                || wide(code & LOW)
                || last_nibble == nibble_count + 1
            {
                nibble_count += 2;
                (code & FLAGS, Some(code & LOW))
            } else {
                nibble_count += 1;
                (code << 4, None)
            };
            if let Some(low) = prev {
                emit(low << 4 | this >> 4)?;
                prev = next;
            } else if let Some(low) = next {
                emit(this | low)?;
            } else {
                prev = Some(this >> 4);
            }
            Ok(())
        })?;

        nibble_count -= expire_five as usize;
        assert_eq!(expire_five, prev == Some(4));
        assert_eq!(!expire_five, prev == None);
        assert_eq!(expected_count, nibble_count);
        Ok(nibble_count / 2)
    }

    pub fn write_bytes<W>(&self, out: &mut W) -> std::io::Result<usize>
    where
        W: std::io::Write,
    {
        self.for_each_byte(|byte| out.write_all(&[byte]))
    }
}

impl From<&LeapSecs> for Vec<u8> {
    fn from(list: &LeapSecs) -> Vec<u8> {
        let mut bytes = Vec::new();
        list.write_bytes(&mut bytes).unwrap();
        bytes
    }
}

impl From<LeapSecs> for Vec<u8> {
    fn from(list: LeapSecs) -> Vec<u8> {
        Vec::<u8>::from(&list)
    }
}

#[cfg(test)]
mod test {
    use crate::*;
    use std::convert::TryFrom;

    #[test]
    fn test() {
        let binary: &[u8] = b"\x00\x11\x11\x11\x12\x11\x34\x31\
                              \x21\x12\x22\x9D\x56\x52\x7F";
        let parsed = LeapSecs::try_from(binary).unwrap();
        let written: Vec<u8> = parsed.into();
        assert_eq!(binary, written);
    }
}
