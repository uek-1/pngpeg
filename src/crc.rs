pub fn png_crc(bytes: Vec<u8>) -> Result<[u8; 4], & 'static str> {
    const POLY : u32 = 0x04C11DB7;
    const XOROUT : u32 = 0xFFFFFFFF;
    const INIT : u32 = 0xFFFFFFFF;
    const REFIN : bool = true;
    const REFOUT : bool = true;

    let mut padding = vec![0u8; 4];
    let mut padded_bytes = bytes;
    padded_bytes.append(&mut padding);
    let mut padded_bits = Bits::new(padded_bytes);
    
    let mut register = match padded_bits.read_bits(32, REFIN) {
        Some(x) => x,
        None => return Err("Error while reading initial 32 bits into register. It's likely that the padding failed."),
    };
    
    //INIT value - see https://stackoverflow.com/questions/43823923/implementation-of-crc-8-what-does-the-init-parameter-do:
    register = register ^ INIT;

    loop {
        let next_bit : u32 = match padded_bits.read_bits(1, REFIN) {
            Some(x) => x,
            None => break,
        };

        let popped_bit = register >> 31;
        register = (register << 1) + next_bit;

        match popped_bit {
            0 => (),
            _ => register = register ^ POLY,
        };
    }
    if REFOUT {
        register = register.reverse_bits();
    }
    
    register = register ^ XOROUT;

    Ok(register.to_be_bytes())
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

    pub fn read_bits(&mut self, num : u32, lsb : bool) -> Option<u32> { 
        if self.len() as i32 - (num as i32) < 0 {
            return None;
        }

        let mut value = 0u32;
        for i in self.position..(self.position+num) {
            let byte_index = (i / 8) as usize;
            let byte = match lsb {
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
        let len_i32 : i32 = 8 * self.bytes.len() as i32 - self.position as i32;
        
        match len_i32 < 0 {
            true => 0,
            false => len_i32 as u32,
        }
    } 
}

