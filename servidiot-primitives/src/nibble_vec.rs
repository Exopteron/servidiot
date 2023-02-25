use serde::{Deserialize, Serialize};

/// A vec of nibbles.
#[derive(Debug, Serialize, Deserialize)]
#[serde(transparent)]
pub struct NibbleVec {
    #[serde(skip)]
    flag: bool,
    #[serde(serialize_with = "nbt::i8_array")]
    backing: Vec<i8>,
}
impl Default for NibbleVec {
    fn default() -> Self {
        Self::new()
    }
}
impl NibbleVec {
    pub fn new_from(v: Vec<u8>) -> Self {
        Self {
            flag: false,
            backing: unsafe {
                std::mem::transmute(v)
            }
        }
    }

    pub fn new() -> Self {
        Self { flag: false, backing: Vec::new() }
    }
    pub fn get(&self, idx: usize) -> u8 {
        let byte_index = idx / 2;
        let (a, b) = decompress_nibble(self.backing[byte_index] as u8);
        if idx % 2 == 0 {
            a
        } else {
            b
        }
    }

    /// Returns `None` if the value is greater than 15.
    pub fn set(&mut self, idx: usize, value: u8) -> Option<()> {
        let byte_index = idx / 2;
        let (mut a, mut b) = decompress_nibble(self.backing[byte_index] as u8);
        if idx % 2 == 0 {
            a = value;
        } else {
            b = value;
        }
        self.backing[byte_index] = make_nibble_byte(a, b)? as i8;
        Some(())
    }
    pub fn len(&self) -> usize {
        let len = self.backing.len() / 2;
        if self.flag {
            len - 1
        } else {
            len
        }
    }

    pub fn is_empty(&self) -> bool {
        self.backing.is_empty()
    }
    pub fn push(&mut self, v: u8) {
        if self.flag {
            let v2 = self.backing.pop().unwrap() as u8;
            self.backing.push(make_nibble_byte(v2, v).unwrap() as i8);
        } else {
            self.backing.push(v as i8);
        }
        self.flag ^= true;
    }
    pub fn get_backing(&self) -> &[u8] {
        unsafe {
            std::mem::transmute(self.backing.as_slice())
        }
    }

    pub fn backing_mut(&mut self) -> &mut Vec<u8> {
        unsafe {
            std::mem::transmute(&mut self.backing)
        }
    }
}
#[inline(always)]
fn make_nibble_byte(mut a: u8, mut b: u8) -> Option<u8> {
    if a > 15 || b > 15 {
        return None;
    }
    b <<= 4;
    b &= 0b11110000;
    a &= 0b00001111;
    Some(a | b)
}
#[inline(always)]
fn decompress_nibble(input: u8) -> (u8, u8) {
    let b = input & 0b11110000;
    let b = b >> 4;
    let a = input & 0b00001111;
    (a, b)
}

#[cfg(test)]
mod tests {
    use super::NibbleVec;

    #[test]
    fn nibble_test() {
        let mut array = NibbleVec::new();
        const SLICE: [u8; 5] = [4,2,7,3,5];

        for v in SLICE {
            array.push(v);
        }
        let mut out = vec![];
        for i in 0..array.len() {
            out.push(array.get(i));
        }
        for (idx, v) in out.iter().enumerate() {
            if SLICE[idx] != *v {
                panic!("fail")
            }
        }
    }
}