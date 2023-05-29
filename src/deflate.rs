use crate::bits::Bits;
use std::collections::HashMap;

const VAL : u8 = 1;

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
        
        let mut decoded_block = match btype {
            0b0 => decode_block_none(&mut comp),
            0b01 => decode_block_fixed(&mut comp),
            0b10 => decode_block_dynamic(&mut comp),
            _ => return Err("Deflate stream is broken!"),
        };

        out.append(&mut decoded_block);

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

fn decode_block_fixed(comp: &mut Bits) -> Vec<u8> {
    println!("Attempting to decode type 01 block!");
    let mut out = vec![];
    let mut current_bits = comp.read_bits(6).expect("Deflate stream is broken  couldn't read first 7 bits!");
   
    let huff : HashMap<u32, u32> = generate_fixed_huffman(); 
    let length_table : HashMap<u32, (u32, u32)> = HashMap::new();
    let dist_table : HashMap<u32, (u32, u32)> = HashMap::new();

    loop {
        let next_bit = comp.read_bits(1).expect("Deflate stream is broken - trying to read out of bounds!");

        current_bits = (current_bits << 1) + next_bit;

        let code = match huff.get(&current_bits) {
            Some(&x) => x,
            None => continue,
        };
        
        println!("Found code {}", code);

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
        
        let extra = comp.read_bits(extra_len).expect("Deflate stream is broken - couldn't read extra bits!");
        
        let length = length + extra.reverse_bits() >> (32 - extra_len);

        let mut dist_code = comp.read_bits(6).expect("Deflate stream is broken - couldn't read initial distance bits!"); 
        let mut dist_value = 0u32;
        
        loop {
            let next_bit = comp.read_bits(1).expect("Deflate stream is broken - couldn't find distance code!");

            dist_code = (dist_code << 1) + next_bit;
            
            match huff.get(&dist_code) {
                Some(&x) => {dist_value = x; break},
                None => continue
            };
        }
        
        //Now that we have the distance code, use the hash table to read code and extra bits to
        //find real distance value.

        let (extra_len, dist) = *dist_table.get(&dist_value).expect("Invalid fixed length huffman code was read for distance");
        let extra = comp.read_bits(extra_len).expect("Deflate stream is broken - couldn't read extra distance bits!");

        let dist = dist + extra.reverse_bits() >> (32 - extra_len);
       
        //Push <length> literals starting from <dist> bytes before.
        for i in 0..length {
            out.push(out[(i - dist) as usize]);
        }
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

pub fn defilter(decoded_stream: Vec<u8>) -> Result<Vec<u8>, &'static str> {
    Ok(decoded_stream)
}
