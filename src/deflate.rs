use crate::bits::Bits;
use std::collections::HashMap;

pub fn decompress(deflate_stream: Vec<u8>) -> Result<Vec<u8>, &'static str> {
    //implementation of zlib deflate algorithm
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
            0b0 => decode_block_none(&mut comp, out),
            0b01 => decode_block_fixed(&mut comp, out),
            0b10 => decode_block_dynamic(&mut comp, out),
            _ => return Err("Deflate stream is broken!"),
        };


        if bfinal == 1 { 
            break;
        }
    }

    Ok(out)
}

fn decode_block_dynamic(comp: &mut Bits, out : Vec<u8>) -> Vec<u8> {
    println!("Attempting to decode type 10 block!");
    let mut out = out;
    let HLIT = 257 + comp.read_bits(5).expect("Deflate stream is broken - couldn't read HLIT from stream!");
    let HDIST = 1 + comp.read_bits(5).expect("Deflate stream is broken - couldn't read HDIST from stream!");
    let HCLEN = 4 + comp.read_bits(4).expect("Deflate stream is broken couldn't read HCLEN from stream!");
    println!("There are {} encoded literals/lengths, {} encoded distances, and {} encoded CL codes", HLIT , HDIST , HCLEN );

    let code_length_huff : HashMap<String, u32> = generate_code_length_huff(comp, HCLEN);
    let ll_huff : HashMap<u32, u32> = generate_dyn_ll_huff(comp, code_length_huff.clone(), HLIT);
    let dist_huff : HashMap<u32, u32> = generate_dyn_dist_huff(comp, code_length_huff, HDIST);

    //call func to decode HLIT literals/lengthss into huffman table.
    //call func to decode HDIST distances
    //Use huffman tables to decode compressed data.

    out
}

fn generate_code_length_huff(comp: &mut Bits, code_count: u32) -> HashMap<String, u32> {
    //code lengths are numbers so read them in backwards;
    let order : Vec<u32> = vec![16, 17, 18, 0, 8, 7, 9, 6, 10, 5, 11, 4, 12, 3, 13, 2, 14, 1, 15];
    let mut lengths : Vec<u32> = vec![];
    
    for _i in 0..code_count {
        let code_length = comp.read_bits(3).expect("Deflate stream is broken - couldn't read code length code lengths!");
        lengths.push(code_length);
    }

    let mut lengths_with_symbols : Vec<Vec<u32>> = vec![vec![]; 8];
    
    //Read input bits and find code length for each symbol in order
    for i in 0..code_count {
        let code_len = lengths[i as usize];
        lengths_with_symbols[code_len as usize].push(order[i as usize]);
    }    
   
    let mut code_length_huff : HashMap<String, u32> = HashMap::new();
    //Construct hashmap!
    let mut code = 0u32;

    for (code_length, symbols) in lengths_with_symbols.into_iter().enumerate().skip(1) { 
        //We skip code length 0 when creating the table
        let mut sorted = symbols;
        sorted.sort();
        
        //println!("{}", code_length);
        for symbol in sorted {
            let code_string = format!("{:#01$b}", code, code_length + 2); 
            //println!("{}", &code_string);
            code_length_huff.insert(code_string, symbol);
            code += 1;
        }
        code = code << 1;
    }
    
    for (code, symbol) in code_length_huff.clone() {
        println!("{} -> {} ", code, symbol);
    }
    println!("");
    

    code_length_huff
}

fn generate_dyn_ll_huff(comp: &mut Bits, cl_huff: HashMap<String,u32>, ll_count: u32) -> HashMap<u32, u32> {
    let mut lengths_with_symbols : Vec<Vec<u32>> = vec![vec![]; 16];
    let comp = comp;
    let mut ll_lens_pushed = 0u32;
    //Read ll_count ll code lengths using cl_huff into ll_lengths
    loop {
        println!("ll_lens_pushed {}", ll_lens_pushed);
        if ll_lens_pushed == ll_count {
            break;
        }

        let mut decoded = 0u32;
        //Decode one ll length from stream using cl_huff - may cause error due to premature
        //matching caused by u32! 
        let mut current_bits = comp.read_bits(1).expect("Deflate stream broken - couldn't decode ll code length!"); 
        let mut current_length = 1;
        loop {
            let current_bit_string = format!("{:#01$b}", current_bits, 2 + current_length);
            println!("{}", &current_bit_string);
            match cl_huff.get(&current_bit_string) {
                Some(&x) => {decoded = x; break},
                None => (),
            }
            current_bits = (current_bits << 1) + comp.read_bits(1).expect("Deflate stream broken - couldn't decode ll code length!");
            current_length += 1;
        }
        
        let mut last_pushed_len = 0u32;

        println!("{decoded}"); //ERROR HERE!

        match decoded {
            0..=15 => {
                let ll_len = decoded;
                lengths_with_symbols[ll_len as usize].push(ll_lens_pushed);
                ll_lens_pushed += 1;
                last_pushed_len = decoded;
            },
            16 => {
                let ll_len = last_pushed_len;
                let push_count = 3 + (comp.read_bits(2).expect("err!").reverse_bits() >> 30);
                for i in 0..push_count {
                    lengths_with_symbols[ll_len as usize].push(ll_lens_pushed);
                    ll_lens_pushed += 1;
                }
            },
            17 => {
                let zero_count = 3 + (comp.read_bits(3).expect("err!").reverse_bits() >> 29);
                for i in 0..zero_count {
                    lengths_with_symbols[0].push(ll_lens_pushed);
                    ll_lens_pushed += 1;
                }
                last_pushed_len = 0;
            },
            18 =>{
                let zero_count = 11 + (comp.read_bits(7).expect("err!").reverse_bits() >> 25);
                for i in 0..zero_count {
                    lengths_with_symbols[0].push(ll_lens_pushed);
                    ll_lens_pushed += 1;
                }
                last_pushed_len = 0;
            }
            _ => (),
        };
    }

    let mut ll_huff : HashMap<u32, u32> = HashMap::new();
    let mut code = 0u32;    

    for ll_length in lengths_with_symbols {
        let mut sorted = ll_length;
        sorted.sort();

        for symbol in sorted {
            ll_huff.insert(code, symbol);
            code += 1;
        }
        code = code << 1;
    }

    ll_huff
}

fn generate_dyn_dist_huff(comp: &mut Bits, cl_huff: HashMap<String ,u32>, dist_count: u32) -> HashMap<u32, u32> {
    HashMap::new()
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
        
        //Causes errors -> premature matching.
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

fn decode_block_none(comp: &mut Bits, out : Vec<u8>) -> Vec<u8> {
    println!("Attempting to decode type 00 block");
    let mut out = out;
    //comp.skip_byte();
    comp.read_bits(5);
    let block_len = comp.read_bits(16).expect("Couldn't read block length from stream").reverse_bits() >> 16; // two bytes in reversed endianness
    let block_len_compl = comp.read_bits(16).expect("Couldn't read block lenght complement from stream").reverse_bits() >> 16;
    
    println!("Bytes in block {block_len}");

    if block_len_compl != !block_len {
        println!("Complement is invalid!");
    } 
    
    for _i in 0..block_len {
        let next_byte = comp.read_bits(8).expect(&format!("Expected {block_len} bytes in stream - but reached end before!"));
        out.push(next_byte as u8);
    }
    
    /*
    for elem in out.iter() {
        println!("{}", elem);
    }
    */
    out

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

struct Filters {

}

impl Filters {
    pub fn filter(line_num: usize, scanlines: &Vec<Vec<u8>>, bpp: usize) -> Vec<u8>{
        let line = scanlines[line_num].clone();
        let (filter, line) = line.split_at(1);

        match filter[0].reverse_bits() {
            0 => line.to_vec(),
            1 => Self::sub(line_num, scanlines, bpp),
            2 => Self::up(line_num, scanlines),
            3 => Self::ave(line_num, scanlines, bpp),
            4 => Self::paeth(line_num, scanlines, bpp),
            _ => vec![]
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
                    println!("s {} r {}", index, index - (3 * bpp));
                    out[index-(3*bpp)]},
            } as u32;
            println!("filt : {filtered_byte}"); 
            let defiltered_byte = (defiltered_byte % 256) as u8;
            println!("defit: {defiltered_byte}");
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

        for (index, byte) in line.iter().enumerate() {
            let left : u8 = match index {
                0 => 0,
                _ => scanlines[line_num][index-bpp],
            };
            let up : u8 = match line_num {
                0 => 0,
                _ => scanlines[line_num - 1][index], 
            };
            //MAY CAUSE ERRORS DUE TO ROUNDING - CONV TO F64:
            let av : u8 = (left + up ) / 2; 
            let defiltered_byte = ((*byte as u32  + av as u32)%256) as u8;
            out.push(defiltered_byte);
        }

        out
    }

    fn paeth(line_num : usize, scanlines : &Vec<Vec<u8>>, bpp: usize) -> Vec<u8> {
        //println!("paeth");
        let mut out = vec![]; 
        let mut line = Self::get_filterless_line(line_num, scanlines);
        let mut channel = 0;

        for (index, byte) in line.iter().enumerate() { 
            let left : u8 = match index {
                _ if index < 3 + channel => 0,
                _ => out[index-(3*bpp)],
            };

            let up : u8 = match line_num {
                _ if index < 3 + channel => 0,
                _ => scanlines[line_num-1][index],
            };

            let up_left : u8 = match (line_num, index) {
                (0,_) => 0,
                _ if index < 3 + channel => 0,
                _ => scanlines[line_num-1][index-(3*bpp)],
            };

            let paeth_predictor = |l, u, ul| { 
                println!("luul {l} {u} {ul}");
                let p : i32 = l as i32 + u as i32 - ul as i32;
                let pa = (p - l as i32).abs();
                let pb = (p - u as i32).abs();
                let pc = (p - ul as i32).abs();
                println!("{} {} {}", pa, pb , pc);
                if pa <= pb && pa <= pc {
                    return pa as u32;
                }
                else if pb <= pc {
                    return pb as u32;
                }
                pc as u32
            }; 
            let defiltered_byte = *byte as u32 + (paeth_predictor(left as u32, up as u32, up_left as u32));
            out.push((defiltered_byte % 256) as u8);
            //out.push(0u8);
            channel = (channel + 1) % 3;
        }

        out
    }
}

