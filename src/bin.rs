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
                None => Some(hi << 4 | 4), // add trailing nibble
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

// squash bytecodes to nibbles where possible

struct Bytecodes<'a> {
    inner: Widecodes<'a>,
    prev: Option<u8>,
    pos: usize,
    widen: usize,
}

impl<'a> Iterator for Bytecodes<'a> {
    type Item = u8;
    fn next(&mut self) -> Option<u8> {
        let code = match self.inner.next() {
            Some(code) => code,
            None => return None,
        };
        let (this, next) = if code & FLAGS != WIDE | POS
            || wide(code & LOW)
            || self.widen == self.pos + 1
        {
            self.pos += 2;
            (code & FLAGS, Some(code & LOW))
        } else {
            self.pos += 1;
            (code << 4, None)
        };
        if let Some(low) = self.prev {
            self.prev = next;
            Some(low << 4 | this >> 4)
        } else if let Some(low) = next {
            Some(this | low)
        } else {
            self.prev = Some(this >> 4);
            self.next()
        }
    }
}

impl LeapSecs {
    fn widecodes(&self) -> Widecodes<'_> {
        Widecodes { inner: self.iter(), flags: 0, gap: 0 }
    }

    fn scan_bytes(&self) -> (usize, usize) {
        let mut len = 0;
        let mut widen = 0;

        for code in self.widecodes() {
            if code == FLAGS | 4 {
                // omit trailing nibble
                len += 1;
            } else if code & FLAGS != WIDE | POS || wide(code & LOW) {
                len += 2;
            } else {
                len += 1;
                widen = len;
            }
        }

        if len % 2 == 0 {
            (len / 2, 0)
        } else {
            (len / 2 + 1, widen)
        }
    }

    pub fn len_bytes(&self) -> usize {
        self.scan_bytes().0
    }

    pub fn iter_bytes(&self) -> impl Iterator<Item = u8> + '_ {
        let widen = self.scan_bytes().1;
        Bytecodes { inner: self.widecodes(), prev: None, pos: 0, widen }
    }

    pub fn write_bytes<W>(&self, out: &mut W) -> std::io::Result<()>
    where
        W: std::io::Write,
    {
        for byte in self.iter_bytes() {
            out.write_all(&[byte])?;
        }
        Ok(())
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
