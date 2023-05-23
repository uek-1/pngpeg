pub fn png_crc(bytes: Vec<u8>) -> [u8; 4]{
    let mut bits = Bits::new(bytes);
    let poly = 0xffffffffu32; 

    /*
    for i in start_position..end_position {
            let byte_index = (i / 8) as usize;
            let byte = self.bytes[byte_index];
            let shift = 7 - (i % 8);
            let bit = (byte >> shift) as u64 & 1;
            value = (value << 1) | bit;
        }
    */

    [0u8,0u8,0u8,0u8]
}

struct Bits {
    bytes: Vec<u8>,
    position: u32,
}

impl Bits {
    pub fn new(bytes: Vec<u8>) -> Bits {
        Bits {
            bytes :bytes,
            position: 0,
        }
    }

    pub fn read_bits(&mut self, num : u32) -> Option<u32> { 
        if self.position >= self.len() {
            return None;
        }

        let mut value = 0x0000u32; 
        for i in self.position..(self.position+num) {
            let byte_index = (i / 8) as usize;
            let byte = self.bytes[byte_index];
            let shift = 7 - (i % 8);
            let bit = (byte >> shift) as u32 & 1;
            value = (value << 1) | bit;
        }

        self.position += num;
        Some(0)
    }

    pub fn len(&self) -> u32 {
        let len_i32 : i32 = 8 * self.bytes.len() as i32 - self.position as i32 -1i32;
        
        match len_i32 < 0 {
            true => 0,
            false => len_i32 as u32,
        }
    } 
}

