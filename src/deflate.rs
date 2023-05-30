use crate::bits::Bits;
use std::collections::HashMap;




pub fn decompress(deflate_stream: Vec<u8>) -> Result<Vec<u8>, &'static str> {
    //implementation of zlib deflate algorithm
    let mut decoded_stream: Vec<u8> = vec![];
     
    let first_byte = deflate_stream.get(0).expect("Deflate stream is empty!");

    let cmf = first_byte & 0x0fu8;
    let window_size = 2_u32.pow(((first_byte >> 4) + 8) as u32); 
    
    println!("Compression type: {cmf}");
    println!("Compression window: {window_size}");

    let (comp, crc) = deflate_stream.split_at(deflate_stream.len() - 4);
    
    println!("CRC of uncompressed bytes {}", crc.iter().fold("".to_string(), |st, x| st + &format!("{} ", x) ));

    let mut comp = Bits::new(comp[2..].to_vec(), true);
    let mut out : Vec<u8> = vec![]; 

    loop {    
        let bfinal = comp.read_bits(1).expect("Deflate stream header broken - couldn't read block final value!");
         
        
        let btype = comp.read_bits(2).expect("Deflate stream header broken - couldn't read block type!");
        
        out = match btype {
            0b0 => decode_block_none(&mut comp),
            0b01 => decode_block_fixed(&mut comp, out),
            0b10 => decode_block_dynamic(&mut comp),
            _ => return Err("Deflate stream is broken!"),
        };


        if bfinal == 1 { 
            break;
        }
    }

    Ok(decoded_stream)
}

fn decode_block_dynamic(comp: &mut Bits) -> Vec<u8> {
    println!("Attempting to decode type 10 block!");
    vec![]
}

fn decode_block_fixed(comp: &mut Bits, out : Vec<u8>) -> Vec<u8> {
    println!("Attempting to decode type 01 block!");
    let mut out = out;
    let mut current_bits = comp.read_bits(6).expect("Deflate stream is broken  couldn't read first 7 bits!");
   
    let huff : HashMap<u32, u32> = generate_fixed_huffman(); 
    let length_table : HashMap<u32, (u32, u32)> = generate_length_table();
    let dist_table : HashMap<u32, (u32, u32)> = generate_dist_table();

    loop {
        let next_bit = comp.read_bits(1).expect("Deflate stream is broken - trying to read out of bounds!");

        current_bits = (current_bits << 1) + next_bit;

        let code = match huff.get(&current_bits) {
            Some(&x) => x,
            None => continue,
        };
        
        println!("Found literal or length code {}", code);

        match code {
            x if x < 256 => {
                out.push(x as u8);
                current_bits = comp.read_bits(6).expect("Deflate stream is broken - couldn't read bit after pushing literal!");
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
            extra = comp.read_bits(extra_len).expect("Deflate stream is broken - couldn't read extra bits");
            extra = extra.reverse_bits() >> (32 - extra_len);
        }
        println!("Decoded length code {} ", length);
        let length = length + extra;
        println!("Added extra bits to length {}", length); 
        //Distance code is not huffman coded. Just a 5 bit code.
        let mut dist_value = comp.read_bits(5).expect("Deflate stream is broken - couldn't read initial distance bits!"); 
        println!("Read distance code from stream {}", dist_value);
        //Now that we have the distance code, use the hash table to read code and extra bits to
        //find real distance value.
    
        let (extra_len, dist) = *dist_table.get(&dist_value).expect(&format!("Invalid fixed length huffman code was read for distance ({})", dist_value));

        let mut extra = 0u32;

        if extra_len != 0 {
            extra = comp.read_bits(extra_len).expect("Deflate stream is broken - couldn't read extra distance bits!");
            extra = extra.reverse_bits() >> (32 - extra_len); 
        }
        println!("Found distance from table {}", dist);
        let dist = dist + extra;
        println!("Added extra bits to distance {}", dist);
        
        println!("Pushing {} literals starting {} backwards onto output buffer of length {}", length, dist, out.len());
        //Push <length> literals starting from <dist> bytes before.
        for i in (out.len())..(out.len() + length as usize) {
            let found_literal = *out.get(i - dist as usize).expect("output buffer was not long enough - attempting to read literals that don't exist");
            out.push(found_literal);
        }
        println!("Output buffer size {}", out.len());
    }

    out
}

fn decode_block_none(comp: &mut Bits) -> Vec<u8> {
    println!("Attempting to decode type 00 block");
    vec![]
}

fn generate_fixed_huffman() -> HashMap<u32, u32> {
    let mut huff : HashMap<u32, u32> = HashMap::new();

    //code length 7
    let mut code = 0b0000000u32;

    for value in 256..=279u32 { 
        huff.insert(code, value);
        code += 1;
    }

    code = code << 1;

    //code length 8
    for value in 0..=143u32 {
        huff.insert(code, value);
        code += 1;
    }

    for value in 280..=287u32 {
        huff.insert(code, value);
        code += 1;
    }

    code = code << 1;
    
    //code length 9
    for value in 144..=255u32 { 
        huff.insert(code, value);
        code += 1;
    } 

    huff
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
                                                                (283, (5,227)),
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

pub fn defilter(decoded_stream: Vec<u8>) -> Result<Vec<u8>, &'static str> {
    Ok(decoded_stream)
}


