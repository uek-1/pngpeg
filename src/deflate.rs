pub fn decompress(deflate_stream: Vec<u8>) -> Result<Vec<u8>, &'static str> {
    //implementation of zlib deflate algorithm
    let mut decoded_stream: Vec<u8> = vec![];
    
    Ok(decoded_stream)
}

pub fn defilter(decoded_stream: Vec<u8>) -> Result<Vec<u8>, &'static str> {
    Ok(decoded_stream)
}
