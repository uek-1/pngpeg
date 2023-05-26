use crate::png_chunk::*;
use crate::png_stream::*;

pub struct EncPng {  
    chunks : Vec<PngChunk>
}

impl EncPng {
    pub fn new() -> EncPng{
        EncPng{
            chunks : vec![],
        }
    }

    pub fn add_chunk(&mut self, chunk: PngChunk) {
        self.chunks.push(chunk);
    }
    
    pub fn print_chunks(&self) {
        for (num, chunk) in self.chunks.iter().enumerate() {
            println!("Chunk {}, {}", num, chunk);
        }
    }

    pub fn from_bytes(buffer: Vec<u8>) -> Result<EncPng, &'static str> {
        let mut out_png = EncPng::new();
    
        const PNG_HEADER : [u8; 8] = [137u8, 80u8, 78u8, 71u8, 13u8, 10u8, 26u8, 10u8];

        if buffer.iter().take(8).eq(&PNG_HEADER) {
            println!("Valid PNG header.");
        }
        else {
            return Err("PNG header is invalid!");
        }
        //Print out all the chunks with lengths
        let mut buffer_mut = &buffer[8..];

        loop {
            //Check if there are at least 12 bytes remaining - the minimum in a chunk
            if buffer_mut.len() < 12 {
                break;
            }
            
            //ALL SLICES ARE ABLE TO BE UNRWRAPPED INTO ARRAYS BECAUSE THERE ARE AT LEAST 12 BYTES.

            //Length of chunk data - not including the type and crc - is stored in the first 4 bytes of the chunk
            let chunk_length_bytes = &buffer_mut[..4];
            let chunk_length: usize = u32::from_be_bytes(chunk_length_bytes.try_into().unwrap()) as usize;  
            
            //Type of chunk is stored in the 4th-8th bytes of the chunk
            let chunk_type_bytes = &buffer_mut[4..8];
            let chunk_type_string = String::from_utf8_lossy(chunk_type_bytes);
            let chunk_type = ChunkType::type_from_bytes(chunk_type_bytes.try_into().unwrap());
            
            //Every byte between type and CRC is chunk data
            let chunk_data = &buffer_mut[8..8+chunk_length];
            
            //CRC code is stored in the last four bytes of the chunk
            let chunk_crc_bytes = &buffer_mut[8+chunk_length..12+chunk_length];
            
            let mut png_chunk = PngChunk::new(
                chunk_length,
                chunk_type,
                chunk_data.to_vec(),
                chunk_crc_bytes.try_into().unwrap(),
            );
            
            match png_chunk.verify_crc() {
                Ok(false) => return Err("Invalid CRC!"),
                Ok(_) => (),
                Err(x) => return Err(x),
            };
            
            out_png.add_chunk(png_chunk);

            buffer_mut = &buffer_mut[12 + chunk_length..];
        }
        Ok(out_png)
    }

    pub fn decompress(&self) -> DecPng {
        

        DecPng {

        }
    }

    fn get_deflate_stream(&self) -> Vec<u8> {
        let mut deflate_stream : Vec<u8> = vec![];

        for chunk in self.chunks.iter() {
            match chunk.get_type() {
                ChunkType::IDAT => deflate_stream.append(&mut chunk.get_data().clone()),
                _ => continue,
            }; 
        }

        deflate_stream
    } 
}

