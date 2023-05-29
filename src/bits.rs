pub struct Bits {
    bytes: Vec<u8>,
    position: u32,
    lsb: bool,
}

impl Bits {
    pub fn new(bytes: Vec<u8>, lsb: bool) -> Bits {
        Bits { bytes, position: 0 , lsb}
    }

    pub fn read_bits(&mut self, num: u32) -> Option<u32> {
        if self.len() as i32 - (num as i32) < 0 {
            return None;
        }

        let mut value = 0u32;
        for i in self.position..(self.position + num) {
            let byte_index = (i / 8) as usize;
            let byte = match self.lsb {
                false => self.bytes[byte_index],
                true => self.bytes[byte_index].reverse_bits(),
            };

            let shift = 7 - (i % 8);
            let bit = (byte >> shift) as u32 & 1;
            value = (value << 1) | bit;
        }

        self.position += num;
        Some(value)
    }

    pub fn len(&self) -> u32 {
        let len_i32: i32 = 8 * self.bytes.len() as i32 - self.position as i32;

        match len_i32 < 0 {
            true => 0,
            false => len_i32 as u32,
        }
    }
}
