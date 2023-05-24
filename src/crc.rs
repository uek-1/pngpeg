pub fn png_crc(bytes: Vec<u8>) -> [u8; 4]{
    let poly = 0x04C11DB7u32; 
    let mut poly_stack = 0u32;
    let mut padded_bytes = bytes;
    padded_bytes.append(&mut vec![0u8;4]);
    let mut register = 0u32;
    
    //Read 4 bytes into register. We just padded it with 4 so the unwrap is safe:
    let mut shifts = 3;
    for byte in padded_bytes.clone().into_iter().take(4) {
        register += (byte as u32) << 8 * shifts;
        shifts -= 1;
    }

    //Turn padded_bytes into an iter so we can read from the start:
    let mut padded_bytes = padded_bytes.into_iter();

    //CRC process :
    loop {
        //println!("REGISTER {:#b}", register);
        //Attempt to read a byte from padded_bytes
        let next_byte = match padded_bytes.next() {
            Some(x) => x,
            None => break,
        };

        //Read topmost byte:
        let mut top = register >> 24;
    
        //Pop topmost byte from register:
        register = register << 8;

        //Add next byte to register:
        register = register + next_byte as u32;

        //Caclulate poly_stack :
        for i in 0..8 {
            //println!("poly stack {:#b}", poly_stack);
            //Read first (MSB) bit of top:
            top = (top << 1) ^ match (top >> 7) & 1 {
                //If its a 0, there is no XOR operation
                0 => 0, 
                //If its a 1, add poly << 8 to poly_stack because the nth bit of the top byte will
                //affect 32 - 8 - n bits of the next 4 bytes. 
                _ => {
                    poly_stack = poly_stack ^ (poly << (8 - i));
                    poly
                },
            };
        }

        //XOR register with poly_stack:
        register = register ^ poly_stack;
    }
    println!("CRC: ");
    for byte in register.clone().to_be_bytes() {
        print!("{} ", byte);
    }
    println!("");
    //Since the loop broke, register contains only the last four bytes - the remainder
    register.to_be_bytes()
}
/*
    
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
*/

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

