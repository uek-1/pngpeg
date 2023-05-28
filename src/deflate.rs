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

    let comp = &comp[2..];
    
    //Deflate step

    Ok(decoded_stream)
}

pub fn defilter(decoded_stream: Vec<u8>) -> Result<Vec<u8>, &'static str> {
    Ok(decoded_stream)
}
