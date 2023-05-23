pub fn png_crc(bytes: Vec<u8>) -> [u8; 4]{
    //reverse(inverted(CRC POLYNOMIAL)) is the same as reverse(inverted(CRC(bits)))  
    let poly = 0b11101101101110001000001100100000u32; 
    
    //Record number of bits in input bytes:
    let in_bits = 8 * bytes.len();

    //Add 32 bits (4 bytes) of 0s at the end of the bytes vector
    let mut padded_bytes = bytes;
    padded_bytes.append(&mut vec![255u8;4]);

    //Create bitstream
    let mut bits = Bits::new(padded_bytes);

    let mut out_bits = [0u8; 4];
    //println!("BITS BEFORE TEMP {}", bits.len()); 
    //Since we padded bytes with 0u8 * 4 we will always have at least 32 bits - this is a safe
    //unwrap.
    let mut temp_bits = bits.read_bits(32).unwrap();
    let mut bits_processed = 0usize;
    //println!("BITS AFTER TEMP {}", bits.len());
    loop{
        println!("BITS PROCESSED {} BITS INPUT {} BITS REMAINING {}", bits_processed, in_bits, bits.len());
        println!("temp: {:#034b}", temp_bits);
        if bits_processed == in_bits {
            break;
        }

        match (temp_bits >> 31).count_ones() {
            0 => {
                //if the first bit is a zero, shift temp by one and read one from stream.
                temp_bits = temp_bits << 1;
                temp_bits += match bits.read_bits(1) {
                    Some(x) => x,
                    None => panic!("Error reading bits from stream!")
                };
                bits_processed += 1;
            },
            _ => {
                //XOR temp_bits with poly
                //println!("poly: {:#034b}", poly);
                temp_bits = temp_bits ^ poly;
            },
        };
    }
    temp_bits = !temp_bits;
    for i in temp_bits.to_be_bytes() {
        println!("{i}");
    } 
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

        let mut value = 0x0000u32;
        //TODO: TEST WITH LSB (THIS IS MSB!) READING, WITH AND WIHTOT INVERSION
        for i in self.position..(self.position+num) {
            let byte_index = (i / 8) as usize;
            let byte = self.bytes[byte_index];
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

