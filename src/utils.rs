use miniz_oxide::inflate::decompress_to_vec_zlib;
use std::collections::HashMap;

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
        //Can only pack 32 bits into a u32.
        if num > 32 {
            return None;
        }

        if self.len() < num {
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
            value = (value << 1) + bit;
        }

        self.position += num;
        Some(value)
    }

    pub fn read_bits_reversed(&mut self, num: u32) -> Option<u32> {
        match self.read_bits(num) {
            Some(value) => Some(value.reverse_bits() >> (32 - num)),
            None => None,
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

pub struct CRC32{

}

impl CRC32 {
    pub fn png_crc(bytes: Vec<u8>) -> Result<[u8; 4], &'static str> {
        const POLY: u32 = 0x04C11DB7;
        const XOROUT: u32 = 0xFFFFFFFF;
        const INIT: u32 = 0xFFFFFFFF;
        const REFIN: bool = true;
        const REFOUT: bool = true;

        let mut padding = vec![0u8; 4];
        let mut padded_bytes = bytes;
        padded_bytes.append(&mut padding);
        let mut padded_bits = Bits::new(padded_bytes, REFIN);

        let mut register = match padded_bits.read_bits(32) {
            Some(x) => x,
            None => return Err("Error while reading initial 32 bits into register. It's likely that the padding failed."),
        };

        //INIT value - see https://stackoverflow.com/questions/43823923/implementation-of-crc-8-what-does-the-init-parameter-do:
        register ^= INIT;

        while let Some(next_bit) = padded_bits.read_bits(1) {
            let popped_bit = register >> 31;
            register = (register << 1) + next_bit;

            match popped_bit {
                0 => (),
                _ => register ^= POLY,
            };
        }
        if REFOUT {
            register = register.reverse_bits();
        }

        register ^= XOROUT;

        Ok(register.to_be_bytes())
    }
}

pub struct Deflate {

}

impl Deflate {
    pub fn decompress(deflate_stream: Vec<u8>) -> Result<Vec<u8>, &'static str> {
        //implementation of zlib deflate algorithm
        let test : Vec<u8> = decompress_to_vec_zlib(&deflate_stream).expect("t");
        
        for i in 0..4 {
            println!("literal {i} in real output : {}",test[i]);
        }
        println!("number of decompressed literals {}", test.len());

        let first_byte = match deflate_stream.get(0) {
            Some(x) => x,
            None => return Err("Deflate stream was empty!"),
        };
        
        let cmf = first_byte & 0x0fu8;
        let window_size = 2_u32.pow(((first_byte >> 4) + 8) as u32); 
        
        println!("Compression method (cmf): {cmf}");
        println!("Compression window: {window_size}");

        let (comp, crc) = deflate_stream.split_at(deflate_stream.len() - 4);
        
        println!("CRC of uncompressed bytes {}", crc.iter().fold("".to_string(), |st, x| st + &format!("{} ", x) ));
        let flag_byte = comp.get(1).expect("Deflate stream does not contain flags byte!");

        //TODO: FDICT FUNCTIONALITY
        let _fdict = 0b1 & (flag_byte >> 5);

        let mut comp = Bits::new(comp[2..].to_vec(), true);
        let mut out : Vec<u8> = vec![]; 
        loop {    
            let bfinal = comp.read_bits(1).expect("Deflate stream header broken - couldn't read block final value!");
            let btype = comp.read_bits_reversed(2).expect("Deflate stream header broken - couldn't read block type!");
            
            println!("bfinal {bfinal} btype {btype}");

            out = match btype {
                0b0 => Self::decode_block_none(&mut comp, out)?,
                0b01 => Self::decode_block_fixed(&mut comp, out)?,
                0b10 => Self::decode_block_dynamic(&mut comp, out)?,
                _ => return Err("Deflate stream is broken - read reserved btype!"),
            };

            if bfinal == 1 { 
                break;
            }
        }

        if out == test {
            println!("Successfully decompressed!");
        }
        else {
            println!("Unsuccessfully decompressed");
            
            let dub = out.iter().zip(test.iter());
            dub.filter(|(x,y)| x != y).for_each(|(x,y)| println!("Differs: {x} != {y}"));

            panic!();
        }

        Ok(out)
    }

    fn decode_block_dynamic(comp: &mut Bits, out : Vec<u8>) -> Result<Vec<u8>, &'static str> {
        println!("Attempting to decode type 10 block!");
        let mut out = out;
        let literal_length_code_count = 257 + comp.read_bits_reversed(5).expect("Deflate stream is broken - couldn't read HLIT from stream!");
        let distance_code_count = 1 + comp.read_bits_reversed(5).expect("Deflate stream is broken - couldn't read HDIST from stream!");
        let code_length_code_length_count = 4 + comp.read_bits_reversed(4).expect("Deflate stream is broken couldn't read HCLEN from stream!");

        println!("There are {} encoded literals/lengths, {} encoded distances, and {} encoded CL codes", literal_length_code_count , distance_code_count , code_length_code_length_count );

        let code_length_huff : Huffman = Self::generate_code_length_huff(comp, code_length_code_length_count);
        let ll_huff : Huffman = Self::generate_dyn_huff(comp, code_length_huff.clone(), literal_length_code_count);
        let dist_huff : Huffman = Self::generate_dyn_huff(comp, code_length_huff, distance_code_count);

        let length_table : HashMap<u32, (u32,u32)> = Self::generate_length_table();
        let dist_table : HashMap<u32, (u32, u32)> = Self::generate_dist_table();

        loop {
            let code : u32 = ll_huff.read_one_code(comp).expect("Couldn't read next length or literal code!");
            println!("Found literal or length code {}", code);

            match code {
                x if x < 256 => {
                    out.push(x as u8);
                    continue;
                    },
                256 => break,
                _ => (),
            };
            
            //If the loop is still ongoing - decoded is a length - read distance and the read those
            //literals into out

            let (extra_len, length) = *length_table.get(&code).expect("Invalid fixed length huffman code was read for length!");

            println!("Decoded length code {} ", length);
            //Extra length bits are read MSB first instead of the usual LSB that all the other bytes
            //are read...
            let mut extra = 0u32;
            if extra_len != 0 {
                extra = comp.read_bits_reversed(extra_len).expect("Deflate stream is broken - couldn't read extra bits");
            }
            let length = length + extra;
            println!("Added extra bits to length {}", length); 
            //Distance code is huffman coded.
            //let dist_code = comp.read_bits().expect("Deflate stream is broken - couldn't read initial distance bits!"); 
            let decoded_dist : u32 = dist_huff.read_one_code(comp).expect("Couldn't read next distance code");
            
            println!("Read distance code from stream {}", decoded_dist);

            //Now that we have the distance code, use the hash table to read code and extra bits to
            //find real distance value.
        
            let (extra_len, dist) = *dist_table.get(&decoded_dist).expect(&format!("Invalid dynamic huffman code was read for distance ({})", decoded_dist));

            let mut extra = 0u32;

            if extra_len != 0 {
                extra = comp.read_bits_reversed(extra_len).expect("Deflate stream is broken - couldn't read extra distance bits!");
            }
            println!("Found distance from table {}", dist);
            let dist = dist + extra;
            println!("Added extra bits to distance {}", dist);
            
            println!("Pushing {} literals starting {} backwards onto output buffer of length {}", length, dist, out.len());
            //Push <length> literals starting from <dist> bytes before.
            out = Self::zlss(out, length, dist);
            println!("Output buffer size {}", out.len());
        }
        Ok(out)
    }

    fn generate_code_length_huff(comp: &mut Bits, code_count: u32) -> Huffman {
        let order : Vec<u32> = vec![16, 17, 18, 0, 8, 7, 9, 6, 10, 5, 11, 4, 12, 3, 13, 2, 14, 1, 15];
        let mut lengths : Vec<u32> = vec![];
        
        for _i in 0..code_count {
            //Read one 3 bit CL code length - which is reversed because it is an integer - from the stream
            let code_length = comp.read_bits_reversed(3).expect("Deflate stream is broken - couldn't read code length code lengths!");
            lengths.push(code_length);
        }

        let mut lengths_with_symbols : Vec<Vec<u32>> = vec![vec![]; 8];
        
        //Read input bits and find code length for each symbol in order
        for i in 0..code_count {
            let code_len = lengths[i as usize];
            lengths_with_symbols[code_len as usize].push(order[i as usize]);
        }    
       
        let code_length_huff : Huffman = match Huffman::generate_from_length_symbols(lengths_with_symbols) {
            Ok(x) => x,
            Err(e) => panic!("{}", e),
        };

        code_length_huff
    }

    fn decode_code_length_code(code: u32, last_pushed : u32, stream: &mut Bits) -> Vec<u32> {
        let mut codes : Vec<u32> = vec![];

        match code {
            0..=15 => codes.push(code),
            16 => {
                let push_count = 3 + stream.read_bits_reversed(2).expect("err!");
                for _i in 0..push_count {
                    codes.push(last_pushed);
                }
            },
            17 => {
                let zero_count = 3 + stream.read_bits_reversed(3).expect("err!");
                for _i in 0..zero_count {
                    codes.push(0);
                }
            },
            18 =>{
                let zero_count = 11 + stream.read_bits_reversed(7).expect("err!");               
                for _i in 0..zero_count {
                    codes.push(0);
                }
            }
            _ => (),
        }
        codes
    }

    fn generate_dyn_huff(comp: &mut Bits, cl_huff: Huffman, symbol_count: u32) -> Huffman {
        let mut lengths_with_symbols : Vec<Vec<u32>> = vec![vec![]; 16];
        let comp = comp;
        let mut code_lengths_pushed = 0u32;
        //Read ll_count ll code lengths using cl_huff into ll_lengths
        let mut last_pushed_length = 0u32;

        loop {
            //println!("lls : {} / {}", ll_lens_pushed, ll_count);
            if code_lengths_pushed == symbol_count {
                break;
            }

            let decoded : u32 = cl_huff.read_one_code(comp).expect("Couldn't decode LL huff code length using CL huff");
            //Decode one ll length from stream using cl_huff  
            
            let code_lengths = Self::decode_code_length_code(decoded, last_pushed_length, comp);
            
            for code_length in code_lengths {
                lengths_with_symbols[code_length as usize].push(code_lengths_pushed);
                code_lengths_pushed += 1;
                last_pushed_length = code_length;
            }
        }

        let huff = match Huffman::generate_from_length_symbols(lengths_with_symbols) {
            Ok(x) => x,
            Err(e) => panic!("{}", e),
        };

        huff 
    }

    fn decode_block_fixed(comp: &mut Bits, out : Vec<u8>) -> Result<Vec<u8>, &'static str> {
        println!("Attempting to decode type 01 block!");
        let mut out = out;
       
        let huff : Huffman = Self::generate_fixed_huffman(); 
        let length_table : HashMap<u32, (u32, u32)> = Self::generate_length_table();
        let dist_table : HashMap<u32, (u32, u32)> = Self::generate_dist_table();

        loop {
            let code = huff.read_one_code(comp)?;
            println!("Found literal or length code {}", code);

            match code {
                x if x < 256 => {
                    out.push(x as u8);
                    continue;
                    },
                256 => break,
                _ => (),
            };
            
            //If the loop is still ongoing - decoded is a length - read distance and the read those
            //literals into out
            let (extra_len, length) = *length_table.get(&code).expect("Invalid fixed length huffman code was read for length!");
            let mut extra = 0u32;

            //Extra length bits are read MSB first instead of the usual LSB that all the other bytes
            //are read...
            if extra_len != 0 {
                extra = comp.read_bits_reversed(extra_len).expect("Deflate stream is broken - couldn't read extra bits");
            }
            println!("Decoded length code {} ", length);
            let length = length + extra;
            println!("Added extra bits to length {}", length); 
            //Distance code is not huffman coded. Just a 5 bit code.
            let dist_value = comp.read_bits(5).expect("Deflate stream is broken - couldn't read initial distance bits!"); 
            println!("Read distance code from stream {}", dist_value);
            //Now that we have the distance code, use the hash table to read code and extra bits to
            //find real distance value.
        
            let (extra_len, dist) = *dist_table.get(&dist_value).expect(&format!("Invalid fixed length huffman code was read for distance ({})", dist_value));

            let mut extra = 0u32;

            if extra_len != 0 {
                extra = comp.read_bits_reversed(extra_len).expect("Deflate stream is broken - couldn't read extra distance bits!");
            }
            println!("Found distance from table {}", dist);
            let dist = dist + extra;
            println!("Added extra bits to distance {}", dist);
            
            println!("Pushing {} literals starting {} backwards onto output buffer of length {}", length, dist, out.len());
            //Push <length> literals starting from <dist> bytes before.
            out = Self::zlss(out, length, dist);
            println!("Output buffer size {}", out.len());
        }

        Ok(out)
    }

    fn decode_block_none(comp: &mut Bits, out : Vec<u8>) -> Result<Vec<u8>, &'static str> {
        println!("Attempting to decode type 00 block");
        let mut out = out;
        //comp.skip_byte();
        comp.read_bits(5);
        let block_len = comp.read_bits_reversed(16).expect("Couldn't read block length from stream"); // two bytes in reversed endianness
        let block_len_compl = comp.read_bits_reversed(16).expect("Couldn't read block lenght complement from stream");
        
        println!("Bytes in block {block_len}");

        if !block_len_compl << 16 != block_len << 16{
            println!("Complement is invalid!");
        } 
        
        for _i in 0..block_len {
            let next_byte = comp.read_bits(8).expect(&format!("Expected {block_len} bytes in stream - but reached end before!"));
            out.push((next_byte as u8).reverse_bits());
        }
        
        Ok(out)
    }

    fn generate_fixed_huffman() -> Huffman {
        let mut bitmap : HashMap<String, u32> = HashMap::new();

        //code length 7
        let mut code = 0u32;

        for value in 256..=279u32 { 
            let code_string = format!("{:#01$b}", code, 2 + 7);     
            bitmap.insert(code_string, value);
            code += 1;
        }

        code = code << 1;
        //code length 8
        for value in 0..=143u32 {
            let code_string = format!("{:#01$b}", code, 2 + 8);
            bitmap.insert(code_string, value);
            code += 1;
        }

        for value in 280..=287u32 {
            let code_string = format!("{:#01$b}", code, 2 + 8);
            bitmap.insert(code_string, value);
            code += 1;
        }

        code = code << 1;
        
        //code length 9
        for value in 144..=255u32 { 
            let code_string = format!("{:#01$b}", code, 2 + 9);
            bitmap.insert(code_string, value);
            code += 1;
        } 
        
        Huffman {bitmap}
    }

    //Push <length> literals starting from <dist> bytes before.
    fn zlss(out : Vec<u8>, length : u32, distance: u32) -> Vec<u8> {
        let mut out = out;

        for i in (out.len())..(out.len() + length as usize) {
            let found_literal = *out.get(i - distance as usize).expect("output buffer was not long enough - attempting to read literals that don't exist");
            out.push(found_literal);
        }

        out
    }


    fn generate_length_table() -> HashMap<u32, (u32,u32)> {
        let length_table : HashMap<u32, (u32, u32)> = HashMap::from([
                                                                    (257, (0,3)),
                                                                    (258, (0,4)),
                                                                    (259, (0,5)),
                                                                    (260, (0,6)),
                                                                    (261, (0,7)),
                                                                    (262, (0,8)),
                                                                    (263, (0,10)),
                                                                    (264, (0,10)),
                                                                    (265, (1,11)),
                                                                    (266, (1,13)),
                                                                    (267, (1,15)),
                                                                    (268, (1,17)),
                                                                    (269, (2,19)),
                                                                    (270, (2,23)),
                                                                    (271, (2,27)),
                                                                    (272, (2,31)),
                                                                    (273, (3,35)),
                                                                    (274, (3,43)),
                                                                    (275, (3,51)),
                                                                    (276, (3,59)),
                                                                    (277, (4,67)),
                                                                    (278, (4,83)),
                                                                    (279, (4,99)),
                                                                    (280, (4,115)),
                                                                    (281, (5,131)),
                                                                    (282, (5,163)),
                                                                    (283, (5,195)),
                                                                    (284, (5,225)),
                                                                    (285, (0,258))
        ]);

        length_table
    }

    fn generate_dist_table() -> HashMap<u32, (u32,u32)> {
        let dist_table : HashMap<u32, (u32,u32)> = HashMap::from([
                                                                 (0, (0,1)),
                                                                 (1, (0,2)),
                                                                 (2, (0,3)),
                                                                 (3, (0,3)),
                                                                 (4, (1,5)),
                                                                 (5, (1,7)),
                                                                 (6, (2,9)),
                                                                 (7, (3,13)),
                                                                 (8, (3,17)),
                                                                 (9, (3,25)),
                                                                 (10, (4,33)),
                                                                 (11, (4,49)),
                                                                 (12, (5,65)),
                                                                 (13, (5,97)),
                                                                 (14, (6, 129)),
                                                                 (15, (6, 193)),
                                                                 (16, (7,257)),
                                                                 (17, (7,385)),
                                                                 (18, (8,513)),
                                                                 (19, (8,769)),
                                                                 (20, (9,1025)),
                                                                 (21, (9,1537)),
                                                                 (22, (10,2049)),
                                                                 (23, (10,3073)),
                                                                 (24, (11,4097)),
                                                                 (25, (11,6145)),
                                                                 (26, (12,8193)),
                                                                 (27, (12,12289)),
                                                                 (28, (13,16385)),
                                                                 (29, (13,24577))
        ]);

        dist_table
    }

    pub fn defilter(decoded_stream: Vec<u8>, height: u32, width : u32, bpp: usize) -> Result<Vec<Vec<u8>>, &'static str> {
        println!("{} in {} scanlines" , decoded_stream.len(), height);
        let line_size = decoded_stream.len() / height as usize; 
        let scanlines : Vec<Vec<u8>> = decoded_stream
                                                    .chunks(line_size)
                                                    .map(|x| x.to_vec())
                                                    .collect();
        let mut filtered : Vec<Vec<u8>> = scanlines.clone();

        for (index, line) in scanlines.into_iter().enumerate() {
            filtered[index] = Filters::filter(index, &filtered, bpp);
        }

        Ok(filtered)
    }
}

#[derive(Clone)]
pub struct Huffman {
    bitmap : HashMap<String, u32>
}

impl Huffman {
    pub fn generate_from_length_symbols(lengths_with_symbols: Vec<Vec<u32>>) -> Result<Huffman, &'static str> {
        if lengths_with_symbols.len() == 0 {
            return Err("Cannot generate huffman from empty vector!");
        }

        let mut bitmap : HashMap<String, u32> = HashMap::new();
        let mut code = 0u32;    

        for (code_length, symbols) in lengths_with_symbols.into_iter().enumerate().skip(1) { 
            //We skip code length 0 when creating the table
            let mut sorted = symbols;
            sorted.sort();
            
            for symbol in sorted {
                let code_string = format!("{:#01$b}", code, code_length + 2); 
                bitmap.insert(code_string, symbol);
                code += 1;
            }
            code = code << 1;
        }
        let huffman = Huffman{bitmap};
        Ok(huffman)
    }

    pub fn read_one_code(&self, stream: &mut Bits) -> Result<u32, & 'static str>{
        let mut bits = 0u32;
        let mut num_bits = 0usize;
        loop{
            let next_bit = stream.read_bits(1).expect("Deflate stream is broken - trying to read out of bounds!");
            bits = (bits << 1) + next_bit;
            num_bits += 1;
            let bit_string = format!("{:#01$b}", bits, 2 + num_bits);     
            match self.bitmap.get(&bit_string) {
                Some(&x) => {return Ok(x)},
                None => continue,
            };
        }
    }

    pub fn get(&self,  key : &String) -> Option<&u32>{
        self.bitmap.get(key)
    }


}


pub struct Filters {

}

impl Filters {
    pub fn filter(line_num: usize, scanlines: &Vec<Vec<u8>>, bpp: usize) -> Vec<u8>{
        let line = scanlines[line_num].clone();
        let (filter, line) = line.split_at(1);
        match filter[0] {
            0 => line.to_vec(),
            1 => Self::sub(line_num, scanlines, bpp),
            2 => Self::up(line_num, scanlines),
            3 => Self::ave(line_num, scanlines, bpp),
            4 => Self::paeth(line_num, scanlines, bpp),
            _ => panic!("INVALID FILTER TYPE!")
        }
    }
    
    fn get_filterless_line(line_num : usize, scanlines : &Vec<Vec<u8>>) -> Vec<u8> {
        let mut line = scanlines[line_num].clone();
        line.into_iter().skip(1).collect()
    }
    
    //NOTE: scanlines is always accessed with index + 1 because get_filterless_line has 1 less elem
    //per line than scanlines.

    fn sub(line_num : usize, scanlines : &Vec<Vec<u8>>, bpp : usize) -> Vec<u8> {
        println!("sub");
        let mut out = vec![];
        let mut line = Self::get_filterless_line(line_num, scanlines);
        let mut channel = 0;

        for (index, filtered_byte) in line.iter().enumerate() {
            let defiltered_byte = (*filtered_byte) as u32 + match index {
                _ if index < 3 + channel => 0,
                _ => {
                    //println!("s {} r {}", index, index - (3 * bpp));
                    out[index-(3*bpp)]},
            } as u32;
            //println!("filt : {filtered_byte}"); 
            let defiltered_byte = (defiltered_byte % 256) as u8;
            //println!("defit: {defiltered_byte}");
            out.push(defiltered_byte);
            channel = (channel + 1) % 3
            //out.push(0u8);
        }

        out
    }

    fn up(line_num : usize , scanlines : &Vec<Vec<u8>>) -> Vec<u8> {
        println!("up");
        let mut out = vec![];
        let mut line = Self::get_filterless_line(line_num, scanlines);

        for (index, byte) in line.iter().enumerate() { 
            let defiltered_byte = *byte as u32 + match line_num { 
                0 => 0,
                _ => scanlines[line_num - 1][index], 
            } as u32;
            
            let defiltered_byte = (defiltered_byte % 256) as u8;

            out.push(defiltered_byte);
        }

        out
    }

    fn ave(line_num : usize, scanlines : &Vec<Vec<u8>>, bpp : usize) -> Vec<u8> {
        println!("ave");
        let mut out = vec![];
        let mut line = Self::get_filterless_line(line_num, scanlines);
        let mut channel = 0;

        for (index, byte) in line.iter().enumerate() {
            let left : u32 = match index {
                _ if index < 3 + channel => 0,
                _ => out[index-(3 * bpp)],
            } as u32;
            let up : u32 = match line_num {
                0 => 0,
                _ => scanlines[line_num - 1][index], 
            } as u32;
            //MAY CAUSE ERRORS DUE TO ROUNDING - CONV TO F64:
            let av = ((left as f64 + up as f64 ) / 2.0).floor(); 
            let defiltered_byte = ((*byte as u32  + av as u32)%256) as u8;
            out.push(defiltered_byte);
            channel = (channel + 1) % 3
        }

        out
    }

    fn paeth(line_num : usize, scanlines : &Vec<Vec<u8>>, bpp: usize) -> Vec<u8> {
        println!("paeth");
        let mut out = vec![]; 
        let line = Self::get_filterless_line(line_num, scanlines);
        let mut channel = 0;

        for (index, byte) in line.iter().enumerate() { 
            let left : u32 = match index {
                _ if index < 3 + channel => 0,
                _ => out[index-(3*bpp)],
            } as u32;

            
            let up : u32 = match line_num {
                0 => 0,
                _ => scanlines[line_num-1][index],
            } as u32;
           
            
            let up_left : u32 = match (line_num, index) {
                _ if line_num < 1 => 0,
                _ if index < 3 + channel => 0,
                _ => scanlines[line_num-1][index-(3*bpp)],
            } as u32;

            let defiltered_byte = (*byte) as u32 + (Self::get_paeth_predictor(left as u32, up as u32, up_left as u32));
            out.push((defiltered_byte % 256) as u8);
            //out.push(0u8);
            channel = (channel + 1) % 3;
        }

        out
    }

    fn get_paeth_predictor(left: u32, up: u32, upleft: u32) -> u32 {
        let inital : i32 = left as i32 + up as i32 - upleft as i32;
        let pred_left = (inital - left as i32).abs();
        let pred_up = (inital - up as i32).abs();
        let pred_upleft = (inital - upleft as i32).abs();
        if pred_left <= pred_up && pred_left <= pred_upleft {
            return left as u32;
        }
        else if pred_up <= pred_upleft {
            return up as u32;
        }
        upleft as u32
    }
}