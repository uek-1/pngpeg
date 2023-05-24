pub fn png_crc(bytes: Vec<u8>) -> [u8; 4]{
    let poly = 0xEDB88320u64; //reversed without x^32 term. 
    
    let mut crc = 0xffffffffu64;
    //Create bitstream
    for byte in bytes {
        let mut padded_byte = (crc << 8) + (byte as u64);
        
        for i in 0..8 {
            //println!("{:#b}", padded_byte);
            padded_byte = match padded_byte & 1 {
                0 => (padded_byte >> 1 ) & 0x7FFFFFFF,                     //skip if first (last in MSB) byte is 0
                _ => ((padded_byte >> 1) & 0x7FFFFFFF)^ poly,            //XOR bits with poly. But
                  //because poly does not encode the x^31 term - which always results in 0, we can
                  //implicitly calculate the XOR by right shifting.
            };
        }
        
        //XOR crc with next bytes crc.
        crc = (crc ^ padded_byte); 
    }
    
    crc = !crc;
    //temp_bits = !temp_bits;
    for i in crc.to_be_bytes().iter().skip(4) {
        print!("{} ", !i);
    } 
    println!();

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
        if self.len() as i32 - (num as i32) < 0 {
            return None;
        }

        let mut value = 0u32;
        //TODO: TEST WITH LSB (THIS IS MSB!) READING, WITH AND WIHTOT INVERSION
        for i in self.position..(self.position+num) {
            let byte_index = (i / 8) as usize;
            let byte = self.bytes[byte_index];
            let shift = 7 - (i % 8); // MSB : 7 - (i % 8);
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

