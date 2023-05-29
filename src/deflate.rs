use crate::bits::Bits;
use std::collections::HashMap;

const VAL : u8 = 1;

pub fn decompress(deflate_stream: Vec<u8>) -> Result<Vec<u8>, &'static str> {
    //implementation of zlib deflate algorithm
    let mut decoded_stream: Vec<u8> = vec![];
     
    let first_byte = match deflate_stream.get(0) {
        Some(x) => x,
        None => return Err("Deflate stream is empty!"),
    };

    let cmf = first_byte & 0x0fu8;
    let window_size = 2_u32.pow(((first_byte >> 4) + 8) as u32); 
    
    println!("Compression type: {cmf}");
    println!("Compression window: {window_size}");

    let (comp, crc) = deflate_stream.split_at(deflate_stream.len() - 4);
    
    println!("CRC of uncompressed bytes {}", crc.iter().fold("".to_string(), |st, x| st + &format!("{} ", x) ));

    let mut comp = Bits::new(comp[2..].to_vec(), true);
    let mut out : Vec<u8> = vec![]; 

    loop {    
        let bfinal = match comp.read_bits(1) {
            Some(x) => x,
            None => return Err("Deflate stream header broken - couldn't read block final value!"),
        };
        
        if bfinal == 1 { 
            break;
        }

        let btype = match comp.read_bits(2) {
            Some(x) => x,
            None => return Err("Deflate stream header broken - couldn't read block type!"),
        };
    
        let mut decoded_block = match btype {
            0b0 => decode_block_none(&mut comp),
            0b01 => decode_block_fixed(&mut comp),
            0b10 => decode_block_dynamic(&mut comp),
            _ => return Err("Deflate stream is broken!"),
        };

        out.append(&mut decoded_block);
    }

    Ok(decoded_stream)
}

fn decode_block_dynamic(comp: &mut Bits) -> Vec<u8> {
    vec![]
}

fn decode_block_fixed(comp: &mut Bits) -> Vec<u8> {
    let mut out = vec![];
    let mut current_bits = 0u32;
    
    let ll_tree : HashMap<u32, u32> = generate_fixed_ll_tree();
    let dist_tree : HashMap<u32, u32> = generate_fixed_dist_tree(); 

    loop {
        let next_bit = match comp.read_bits(1) {
            Some(x) => x,
            None => panic!("Deflate stream is broken - trying to read out of bounds!"),
        };

        current_bits = (current_bits << 1) + next_bit;

        let decoded = match ll_tree.get(&current_bits) {
            Some(&x) => x,
            None => continue,
        };

        match decoded {
            x if x < 256 => {out.push(x as u8); break},
            256 => break,
            _ => (),
        };
        
        //If the loop is still ongoing - decoded is a length - read distance and the read those
        //literals into out
        let length = decoded;
        let mut dist_code = 0u32; 
        let mut dist = 0u32;
        
        loop {
            let next_bit = match comp.read_bits(1) {
                Some(x) => x,
                None => panic!("Deflate stream is broken - trying to read out of bounds!"),
            };

            dist_code = (dist_code << 1) + next_bit;
            
            match dist_tree.get(&dist_code) {
                Some(&x) => {dist = x; break},
                None => continue
            };
        }
       
        //Push <length> literals starting from <dist> bytes before.
        for i in 0..length {
            out.push(out[(i - dist) as usize]);
        }
    }

    out
}

fn decode_block_none(comp: &mut Bits) -> Vec<u8> {
    vec![]
}

fn generate_fixed_ll_tree() -> HashMap<u32, u32> {
    HashMap::new()
}

fn generate_fixed_dist_tree() -> HashMap<u32, u32> {
    HashMap::new()
}

pub fn defilter(decoded_stream: Vec<u8>) -> Result<Vec<u8>, &'static str> {
    Ok(decoded_stream)
}
