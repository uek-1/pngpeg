//! Utility algorthims and functions used while encoding or decoding

use std::{collections::HashMap, ops::{Mul, Add}, intrinsics::discriminant_value};
use crate::pixel::{Pixel, Pixels, ColorType};

/// Bits 
///
/// This struct is used to create bitstreams from a vector of bytes.
pub struct Bits {
    
    /// The bytes this stream is reading from
    bytes: Vec<u8>,

    /// Inner value used to record the next bit that will be read out. Note this value starts at 0.
    position: u32,

    /// Boolean value that represents the order which the bits are read - true for
    /// least-significant-bit (lsb) and false for most-significant-bit (msb).
    ///
    /// Consider a byte, 0b11110000. 
    /// If this byte is read LSB first, it would be read 00001111. If it were read MSB first, it
    /// would be read 11110000.
    lsb: bool,
}

impl Bits {
    
    /// Static method to initialize a new bitstream from "bytes" and read least-significant-bit
    /// first if lsb is set to true. 
    pub fn new(bytes: Vec<u8>, lsb: bool) -> Bits {
        Bits { bytes, position: 0 , lsb}
    }
    
    /// Method to read the next num bits in the stream - returns None if num is greater than the
    /// remaining length.
    ///
    /// Bits are returned as a u32, so there will be extra 0s if less than 32 bits are read. The
    /// output u32 has the last bit read on the rightmost position and the first bit read on the
    /// leftmost position. 
    ///
    /// Example: reading 3 bits from 01101111 MSB first would produce the u32 - 0b11. 
    pub fn read_bits(&mut self, num: u32) -> Option<u32> {
        // Can only pack 32 bits into a u32.
        if num > 32 {
            return None;
        }

        if self.len() < num {
            return None;
        }

        let mut value = 0u32;
        for i in self.position..(self.position + num) {
            let byte_index = (i / 8) as usize;
            
            // When read LSB first, the rightmost bit is the first (highest value) bit - so the
            // byte is essentially read in reverse.
            let byte = match self.lsb {
                false => self.bytes[byte_index],
                true => self.bytes[byte_index].reverse_bits(),
            };

            // The nth bit is the (n % 8)th bit of the (n div 8)th byte. To read it, we shift it to
            // the rightmost bit position and bitwise AND it with 00000....1. 
            let shift = 7 - (i % 8);
            let bit = (byte >> shift) as u32 & 1;
            value = (value << 1) + bit;
        }

        self.position += num;
        Some(value)
    }

    /// This method reads n bits just like read_bits() but reverses the output u32 after being
    /// completely calculated. 
    ///
    /// This method is used in cases where some data elements in the stream are packed in different
    /// endianness (MSB or LSB). 
    ///
    /// Example, reading 3 bits MSB first from 01101111 MSB first would produce the u32 - 0b110
    pub fn read_bits_reversed(&mut self, num: u32) -> Option<u32> {
        match self.read_bits(num) {
            Some(value) => Some(value.reverse_bits() >> (32 - num)),
            None => None,
        }
    }

    /// Method used to debug functionality - simply prints the entire byte which the next bit will
    /// be read from.
    pub fn print_current_byte(&mut self) {
        let byte_index = (self.position / 8) as usize;
        println!("pos {} current byte {byte_index} in stream {:#010b}", self.position, self.bytes[byte_index]);
    }
    
    /// Returns the remaining bits in the stream.
    pub fn len(&self) -> u32 {
        // Subtract read bits from total bits.
        let len_i32: i32 = 8 * self.bytes.len() as i32 - self.position as i32;

        match len_i32 < 0 {
            true => 0,
            false => len_i32 as u32,
        }
    }
}

/// Implementation of the CRC-32 algorithm used in PNG files. The specification is  
/// POLY: 0x04C11DB7, XOROUT: 0xFFFFFFFF, INIT: 0xFFFFFFFF, REFIN: true, REFOUT: true. 
pub fn png_crc(bytes: Vec<u8>) -> Result<[u8; 4], &'static str> {
    const POLY: u32 = 0x04C11DB7;
    const XOROUT: u32 = 0xFFFFFFFF;
    const INIT: u32 = 0xFFFFFFFF;
    const REFIN: bool = true;
    const REFOUT: bool = true;
    
    // A 32 bit CRC is padded with 32 bits on the right.
    let mut padding = vec![0u8; 4];
    let mut padded_bytes = bytes;
    padded_bytes.append(&mut padding);
    let mut padded_bits = Bits::new(padded_bytes, REFIN);
    
    // Read 32 bits into the register to begin, because our polynomial has 32 explicit bits
    let mut register = match padded_bits.read_bits(32) {
        Some(x) => x,
        None => return Err("Error while reading initial 32 bits into register. It's likely that the padding failed."),
    };

    //INIT value - see https://stackoverflow.com/questions/43823923/implementation-of-crc-8-what-does-the-init-parameter-do:
    register ^= INIT;

    while let Some(next_bit) = padded_bits.read_bits(1) {
        // Pop one bit from the left, if it is a one XOR the register with poly. 
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

/// Takes in a compressed stream of bytes and returns a decompressed stream of bytes.
///
/// This method will error if the input stream does not meet the specification outlined in RFC
/// 1950 and 1951. Note that this method requires the input stream to be encoded in ZLIB
/// format.
pub fn decompress(deflate_stream: Vec<u8>) -> Result<Vec<u8>, &'static str> {
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
    
    // comp is the compressed bitstream and out is the vector in which decompressed bytes will
    // be stored.
    let mut comp = Bits::new(comp[2..].to_vec(), true);
    let mut out : Vec<u8> = vec![]; 
    loop {    
        // DEFLATE streams consist of multiple blocks of compressed bits each using one of
        // three outlined compression methods. For each block, three bits are read which indicate 
        // the compression type and whether the block is the final one in the stream. Then the
        // block is decompressed using one of three methods and output into out
        let bfinal = comp.read_bits(1).expect("Deflate stream header broken - couldn't read block final value!");
        let btype = comp.read_bits_reversed(2).expect("Deflate stream header broken - couldn't read block type!");
        
        println!("bfinal {bfinal} btype {btype}");

        out = match btype {
            0b0 => decode_block_none(&mut comp, out)?,
            0b01 => decode_block_fixed(&mut comp, out)?,
            0b10 => decode_block_dynamic(&mut comp, out)?,
            _ => return Err("Deflate stream is broken - read reserved btype!"),
        };

        if bfinal == 1 { 
            break;
        }
    }
       
    Ok(out)
}

fn decode_block_dynamic(comp: &mut Bits, out : Vec<u8>)  -> Result<Vec<u8>, &'static str> {
    //println!("Attempting to decode type 10 block!");
    let mut out = out;
    let literal_length_code_count = 257 + comp.read_bits_reversed(5).expect("Deflate stream is broken - couldn't read HLIT from stream!");
    let distance_code_count = 1 + comp.read_bits_reversed(5).expect("Deflate stream is broken - couldn't read HDIST from stream!");
    let code_length_code_length_count = 4 + comp.read_bits_reversed(4).expect("Deflate stream is broken couldn't read HCLEN from stream!");

    //println!("There are {} encoded literals/lengths, {} encoded distances, and {} encoded CL codes", literal_length_code_count , distance_code_count , code_length_code_length_count );

    let code_length_huff : Huffman = generate_code_length_huff(comp, code_length_code_length_count);
    let ll_huff : Huffman = generate_dyn_huff(comp, code_length_huff.clone(), literal_length_code_count);
    let dist_huff : Huffman = generate_dyn_huff(comp, code_length_huff, distance_code_count);

    let length_table : HashMap<u32, (u32,u32)> = generate_length_table();
    let dist_table : HashMap<u32, (u32, u32)> = generate_dist_table();
    
    loop {
        let code : u32 = ll_huff.read_one_code(comp).expect("Couldn't read next length or literal code!");
        //println!("Found literal or length code {}", code);

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

        let (extra_length_bits, length) = *length_table.get(&code).expect("Invalid fixed length huffman code was read for length!");

        //println!("Decoded length code {} ", length);
        //Extra length bits are read MSB first instead of the usual LSB that all the other bytes
        //are read...
        let mut length_extra = 0u32;
        if extra_length_bits != 0 {
            length_extra = comp.read_bits_reversed(extra_length_bits).expect("Deflate stream is broken - couldn't read extra bits");
        }
        let length = length + length_extra;
        //println!("Added extra bits to length {}", length); 
        //Distance code is huffman coded.
        //let dist_code = comp.read_bits().expect("Deflate stream is broken - couldn't read initial distance bits!"); 
        let decoded_dist : u32 = dist_huff.read_one_code(comp).expect("Couldn't read next distance code");
        
        //println!("Read distance code from stream {}", decoded_dist);

        //Now that we have the distance code, use the hash table to read code and extra bits to
        //find real distance value.
    
        let (extra_dist_bits, dist) = *dist_table.get(&decoded_dist).expect(&format!("Invalid dynamic huffman code was read for distance ({})", decoded_dist));

        let mut dist_extra = 0u32;

        if extra_dist_bits != 0 {
            dist_extra = comp.read_bits_reversed(extra_dist_bits).expect("Deflate stream is broken - couldn't read extra distance bits!");
        }
        //println!("Found distance from table {}", dist);
        let dist = dist + dist_extra;
        //println!("Added extra bits to distance {}", dist);
        
        //println!("Pushing {} literals starting {} backwards onto output buffer of length {}", length, dist, out.len());
        //Push <length> literals starting from <dist> bytes before.
        out = zlss(out, length, dist);
        //println!("Output buffer size {}", out.len());
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
        0..=15 => {
            codes.push(code);
            println!("Pushed {code} x1");
        },
        16 => {
            let push_count = 3 + stream.read_bits_reversed(2).expect("err!");
            for _i in 0..push_count {
                codes.push(last_pushed);
            }
            println!("Pushed {last_pushed} x{push_count}");
        },
        17 => {
            let zero_count = 3 + stream.read_bits_reversed(3).expect("err!");
            for _i in 0..zero_count {
                codes.push(0);
            }
            println!("Pushed 0 x{zero_count} ");
        },
        18 =>{
            let zero_count = 11 + stream.read_bits_reversed(7).expect("err!");               
            for _i in 0..zero_count {
                codes.push(0);
            }
            println!("Pushed 0 x{zero_count} ");
        }
        _ => (),
    }
    codes
}

fn generate_dyn_huff(comp: &mut Bits, cl_huff: Huffman, symbol_count: u32) -> Huffman {
    let mut lengths_with_symbols : Vec<Vec<u32>> = vec![vec![]; 16];
    let comp = comp;
    let mut code_lengths_pushed = 0u32;
    let mut last_pushed_length = 0u32;

    loop {
        if code_lengths_pushed == symbol_count {
            break;
        }

        let decoded : u32 = cl_huff.read_one_code(comp).expect("Couldn't decode LL huff code length using CL huff");
        
        let code_lengths = decode_code_length_code(decoded, last_pushed_length, comp);
        
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
    let mut out = out;
   
    let huff : Huffman = generate_fixed_huffman(); 
    let length_table : HashMap<u32, (u32, u32)> = generate_length_table();
    let dist_table : HashMap<u32, (u32, u32)> = generate_dist_table();

    loop {
        let code = huff.read_one_code(comp)?;

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
        let length = length + extra;
        //Distance code is not huffman coded. Just a 5 bit code.
        let dist_value = comp.read_bits(5).expect("Deflate stream is broken - couldn't read initial distance bits!"); 
        //Now that we have the distance code, use the hash table to read code and extra bits to
        //find real distance value.
    
        let (extra_len, dist) = *dist_table.get(&dist_value).expect(&format!("Invalid fixed length huffman code was read for distance ({})", dist_value));

        let mut extra = 0u32;

        if extra_len != 0 {
            extra = comp.read_bits_reversed(extra_len).expect("Deflate stream is broken - couldn't read extra distance bits!");
        }

        let dist = dist + extra;
        
        //Push <length> literals starting from <dist> bytes before.
        out = zlss(out, length, dist);
    }

    Ok(out)
}

fn decode_block_none(comp: &mut Bits, out : Vec<u8>) -> Result<Vec<u8>, &'static str> {
    let mut out = out;

    comp.read_bits(5);
    let block_len = comp.read_bits_reversed(16).expect("Couldn't read block length from stream"); // two bytes in reversed endianness
    let block_len_compl = comp.read_bits_reversed(16).expect("Couldn't read block lenght complement from stream");
    

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
        (263, (0,9)),
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
        (284, (5,227)),
        (285, (0,258))
    ]);

    length_table
}

fn generate_dist_table() -> HashMap<u32, (u32,u32)> {
    let dist_table : HashMap<u32, (u32,u32)> = HashMap::from([
         (0, (0,1)),
         (1, (0,2)),
         (2, (0,3)),
         (3, (0,4)),
         (4, (1,5)),
         (5, (1,7)),
         (6, (2,9)),
         (7, (2,13)),
         (8, (3,17)),
         (9, (3,25)),
         (10, (4,33)),
         (11, (4,49)),
         (12, (5,65)),
         (13, (5,97)),
         (14, (6,129)),
         (15, (6,193)),
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

pub fn decompressed_to_scanlines(decoded_stream: Vec<u8>, image_height: u32) -> Vec<Vec<u8>> {
    let line_size = decoded_stream.len() / image_height as usize; 
    decoded_stream
        .chunks(line_size)
        .map(|x| x.to_vec())
        .collect()
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
}


pub struct Defilter {
    channels : usize,
    bit_depth: u32,
    scanlines : Vec<Vec<u8>>,
    defiltered : Vec<Vec<u8>>,
}

impl Defilter {
    pub fn new(channels : usize, bit_depth : u32, scanlines: Vec<Vec<u8>>) -> Defilter{
        let scanline_count = scanlines.len();

        Defilter {
            channels : channels,
            bit_depth : bit_depth,
            scanlines : scanlines,
            defiltered : vec![vec![]; scanline_count]
        }
    } 

    pub fn defilter(&mut self) -> Result<Vec<Vec<u8>>, &'static str> {
        for scanline_num in 0..self.scanlines.len() {
            self.defilter_line(scanline_num);
        } 
        Ok(self.defiltered.clone())
    }

    fn defilter_line(&mut self, line_num: usize) {
        let line = self.scanlines[line_num].clone();
        let filter = line.get(0).unwrap_or(&5);
        match filter {
            0 => self.defilter_line_by_none(line_num),
            1 => self.defilter_line_by_sub(line_num),
            2 => self.defilter_line_by_up(line_num),
            3 => self.defilter_line_by_ave(line_num),
            4 => self.defilter_line_by_paeth(line_num),
            _ => panic!("INVALID FILTER TYPE!")
        }
    }
    
    fn get_filterless_line(&self, line_num : usize) -> Vec<u8> {
        self.scanlines[line_num].clone()
            .into_iter()
            .skip(1)
            .collect()
    }

    fn get_bytes_per_sample(&self) -> usize {
        match self.bit_depth / 8 {
            0 => 1,
            _ => (self.bit_depth / 8) as usize,
        }
    }

    fn get_bytes_per_pixel(&self) -> usize {
        self.channels * self.get_bytes_per_sample()
    }

    fn get_left_pixel_bytes(&self, origin_row : usize, origin_column: usize) -> Vec<u8> {
        let bytes_per_pixel = self.get_bytes_per_pixel();
        let mut out = vec![];

        for i in origin_column..(origin_column + bytes_per_pixel) {
            match i < bytes_per_pixel {
                true => out.push(0),
                false => out.push(self.defiltered[origin_row][i - bytes_per_pixel])   
            }
        }

        out
    }
    
    fn get_up_pixel_bytes(&self, origin_row : usize, origin_column: usize) -> Vec<u8> {
        let bytes_per_pixel = self.get_bytes_per_pixel();
        let mut out = vec![];
        let row_above = match origin_row < 1 {
            true => vec![0; bytes_per_pixel],
            false => self.defiltered[origin_row - 1].clone()
        };

        for i in origin_column..(origin_column + bytes_per_pixel) {
            out.push(row_above[i]);
        }    

        out
   }

    fn get_upper_left_pixel_bytes(&self, origin_row : usize, origin_column: usize) -> Vec<u8> {
        let bytes_per_pixel = self.get_bytes_per_pixel();
        let mut out = vec![];
        let row_above = match origin_row < 1 {
            true => vec![0; bytes_per_pixel],
            false => self.defiltered[origin_row - 1].clone()
        };

        for i in origin_column..(origin_column + bytes_per_pixel) {
            match i < bytes_per_pixel {
                true => out.push(0),
                false => out.push(row_above[i - bytes_per_pixel])   
            }
        }

        out
    }

    fn defilter_line_by_none(&mut self, line_num : usize) {
        let mut line = self.get_filterless_line(line_num);
        self.defiltered[line_num].append(&mut line);
    }

    fn defilter_line_by_sub(&mut self, line_num : usize) {
        let line = self.get_filterless_line(line_num);
        let bytes_per_pixel = self.get_bytes_per_pixel();

        for (index, filtered_pixel_bytes) in line.chunks(bytes_per_pixel).enumerate() {
            let left_pixel_bytes = self.get_left_pixel_bytes(line_num, index * bytes_per_pixel);
            let mut defiltered_pixel_bytes : Vec<u8> = left_pixel_bytes
                .into_iter()
                .zip(filtered_pixel_bytes.into_iter())
                .map(|(left, filtered)| ((*filtered as u32 + left as u32) % 256) as u8)
                .collect();
            
            self.defiltered[line_num].append(&mut defiltered_pixel_bytes);
        }
    }

    fn defilter_line_by_up(&mut self, line_num : usize) {
        let line = self.get_filterless_line(line_num);
        let bytes_per_pixel = self.get_bytes_per_pixel();

        for (index, filtered_pixel_bytes) in line.chunks(bytes_per_pixel).enumerate() { 
            let up_pixel_bytes = self.get_up_pixel_bytes(line_num, index * bytes_per_pixel);
            let mut defiltered_pixel_bytes : Vec<u8> = up_pixel_bytes
                .into_iter()
                .zip(filtered_pixel_bytes.into_iter())
                .map(|(up, filtered)| ((*filtered as u32 + up as u32) % 256) as u8)
                .collect();
            
            self.defiltered[line_num].append(&mut defiltered_pixel_bytes); 
        }
    }

    fn defilter_line_by_ave(&mut self, line_num : usize) {
        let line = self.get_filterless_line(line_num);
        let bytes_per_pixel = self.get_bytes_per_pixel();

        for (index, filtered_pixel_bytes) in line.chunks(bytes_per_pixel).enumerate() {
            let left_pixel_bytes = self.get_left_pixel_bytes(line_num, index * bytes_per_pixel);
            let up_pixel_bytes = self.get_up_pixel_bytes(line_num, index * bytes_per_pixel);
            let mut defiltered_pixel_bytes : Vec<u8> = up_pixel_bytes
                .into_iter()
                .zip(left_pixel_bytes.into_iter())
                .map(|(up, left)| ((up as f64 + left as f64) / 2.0).floor())
                .zip(filtered_pixel_bytes.into_iter())
                .map(|(ave, filtered)|  ((*filtered as u32 + ave as u32) % 256) as u8)
                .collect();

            self.defiltered[line_num].append(&mut defiltered_pixel_bytes);
        }
    }

    fn defilter_line_by_paeth(&mut self, line_num : usize) {
        let line = self.get_filterless_line(line_num);
        let bytes_per_pixel = self.get_bytes_per_pixel();

        for (index, filtered_pixel_bytes) in line.chunks(bytes_per_pixel).enumerate() { 
            let left_pixel_bytes = self.get_left_pixel_bytes(line_num, index * bytes_per_pixel);
            let up_pixel_bytes = self.get_up_pixel_bytes(line_num, index * bytes_per_pixel);
            let upper_left_pixel_bytes = self.get_upper_left_pixel_bytes(line_num, index * bytes_per_pixel);
            let mut defiltered_pixel_bytes : Vec<u8> = left_pixel_bytes
                .into_iter()
                .zip(up_pixel_bytes.into_iter())
                .zip(upper_left_pixel_bytes.into_iter())
                .map(|((left, up), up_left)| Self::get_paeth_predictor(left as u32, up as u32, up_left as u32))
                .zip(filtered_pixel_bytes.into_iter())
                .map(|(paeth, filtered)|  ((*filtered as u32 + paeth as u32) % 256) as u8)
                .collect();

            self.defiltered[line_num].append(&mut defiltered_pixel_bytes);
        }
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

pub fn defiltered_to_pixels(defiltered_scanlines : Vec<Vec<u8>>, png_color_type : usize) -> Pixels {
    let pixel_color_type = ColorType::from_png_color_type(png_color_type);
    let mut pixels = Pixels::new();
    
    for (idx, scanline) in defiltered_scanlines.into_iter().enumerate() {
        pixels.push(vec![]);
        for pixel_data in scanline.chunks(pixel_color_type.to_channels())  {
            pixels[idx].push(Pixel::new(pixel_color_type, pixel_data.to_vec()));
        }
    }

    pixels
}


//TODO: using i32s for the DCT-2 Matrix could cause large errors because sin is in [0,1].
fn generate_dct_matrix() -> Vec<Vec<f64>> {
    let mut dct_matrix : Vec<Vec<f64>> = vec![vec![0.0; 8]; 8];
    
    for i in 0..8 {
        for j in 0..8 {
            let coeff : f64= (i as f64) * 3.14 * (2.0 * (j as f64) + 1.0) / 16.0;
            dct_matrix[i][j] = coeff.cos();
        }
    }

    dct_matrix
}

pub fn dct(block: Vec<Vec<u8>>) -> Vec<Vec<f64>> {
    let zeroed_block : Vec<Vec<f64>> = block.subtract_amount(128);
    let dct_matrix : Vec<Vec<f64>> = generate_dct_matrix();
    let mut horizontal_block : Vec<Vec<f64>> = vec![];
    //Horizontal:
    for row in zeroed_block.iter() {
        horizontal_block.push(dct_matrix.matrix_multiply(row));
    }

    let mut out_block = vec![];
    
    for row in horizontal_block.transpose().iter() {
        out_block.push(dct_matrix.matrix_multiply(row))
    }
    
    out_block.transpose()
}

impl SubtractAmount<f64> for Vec<Vec<u8>> {
    fn subtract_amount(&self, amt: u8) -> Vec<Vec<f64>> {
        let mut new_matrix = vec![];

        for (row_num, row) in self.iter().enumerate() {
            new_matrix.push(vec![]);
            for elem in row {
                new_matrix[row_num].push(*elem as f64 - amt as f64);
            }
        }

        new_matrix
    }
}

impl Transpose<f64> for Vec<Vec<f64>> { 
    fn transpose(&self) -> Vec<Vec<f64>> {
        let mut transposed = vec![vec![0.0; self[0].len()]; self.len()];
        
        for (row_num, row) in self.iter().enumerate() {
            for (col_num, elem) in row.iter().enumerate() {
                transposed[row_num][col_num] = *elem;
            }
        }

        transposed
    }
}

impl MatrixMultiply<f64> for Vec<Vec<f64>> {
    fn matrix_multiply(&self, other: &Vec<f64>) -> Vec<f64> {
        self.iter()
            .map(|x| Self::dot_product(x, other))
            .collect()
    }
}

trait SubtractAmount<T> {
    fn subtract_amount(&self, amt: u8) -> Vec<Vec<T>>; 
}

trait MatrixMultiply<T : Mul<Output = T> + Add<Output = T> +  From<u8> + Copy> {
    fn dot_product(left: &Vec<T>, right: &Vec<T>) -> T{
        left.into_iter()
            .zip(right.into_iter())
            .fold(T::from(0), |dot_product, (x,y)| dot_product + (*x) * (*y))
    }
    fn matrix_multiply(&self, other: &Vec<T>) -> Vec<T>;
}

trait Transpose<T> {
    fn transpose(&self) -> Vec<Vec<T>>;
}
