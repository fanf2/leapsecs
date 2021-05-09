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

//   __                 _         _
//  / _|_ _ ___ _ __   | |__ _  _| |_ ___ ___
// |  _| '_/ _ \ '  \  | '_ \ || |  _/ -_|_-<
// |_| |_| \___/_|_|_| |_.__/\_, |\__\___/__/
//                           |__/

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
    fn next(&mut self) -> Option<u8> {
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

//  _     _         _         _
// (_)_ _| |_ ___  | |__ _  _| |_ ___ ___
// | | ' \  _/ _ \ | '_ \ || |  _/ -_|_-<
// |_|_||_\__\___/ |_.__/\_, |\__\___/__/
//                       |__/

// convert list of leap seconds to list of wide bytecodes

struct Widecodes<'a> {
    inner: std::slice::Iter<'a, LeapSec>,
    flags: u8,
    gap: u16,
}

impl<'a> Iterator for Widecodes<'a> {
    type Item = u8;
    fn next(&mut self) -> Option<u8> {
        if self.gap == 0 {
            if let Some(leap) = self.inner.next() {
                self.flags = match leap.sign() {
                    Leap::Zero => return self.next(),
                    Leap::Neg => WIDE | NEG,
                    Leap::Pos => WIDE | POS,
                    Leap::Exp => WIDE | NEG | POS,
                };
                self.gap = leap.gap();
            } else {
                return None;
            }
        }
        if self.gap >= 16 * 6 {
            self.gap -= 16 * 6;
            Some(WIDE | 15)
        } else if self.gap % 6 == 0 {
            let gap = self.gap as u8 / 6 - 1;
            self.gap = 0;
            Some(self.flags | gap)
        } else if self.gap <= 16 {
            let gap = self.gap as u8 - 1;
            self.gap = 0;
            Some(self.flags | MONTH | gap)
        } else {
            let years = self.gap / 12;
            let months = self.gap % 12;
            if years > 0 {
                let gap = years as u8 * 2 - 1;
                self.gap = months;
                Some(WIDE | gap)
            } else {
                let gap = months as u8 - 1;
                self.gap = 0;
                Some(self.flags | MONTH | gap)
            }
        }
    }
}

impl LeapSecs {
    fn widecodes(&self) -> Widecodes<'_> {
        Widecodes { inner: self.iter(), flags: 0, gap: 0 }
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

        for code in self.widecodes() {
            if code == FLAGS | 4 {
                expire_five = true;
                nibble_count += 2;
            } else if code & FLAGS != WIDE | POS || wide(code & LOW) {
                nibble_count += 2;
            } else {
                nibble_count += 1;
                last_nibble = nibble_count;
            }
        }

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

        for code in self.widecodes() {
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
        }

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
