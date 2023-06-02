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
        //Clearly I can only pack 32 bits into a u32.
        if num > 32 {
            return None;
        }

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

    pub fn skip_byte(&mut self)  {
        let curr_byte_index = (self.position / 8) as usize;
        loop {
            self.position += 1;
            if (self.position / 8 ) as usize == curr_byte_index {
                break;
            }
        }
    }

    pub fn print_current_byte(&mut self) {
        let byte_index = (self.position / 8) as usize;
        println!("pos {} current byte {byte_index} in stream {:#010b}", self.position, self.bytes[byte_index]);
    }


    pub fn len(&self) -> u32 {
        let len_i32: i32 = 8 * self.bytes.len() as i32 - self.position as i32;

        match len_i32 < 0 {
            true => 0,
            false => len_i32 as u32,
        }
    }
}
