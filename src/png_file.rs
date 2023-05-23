use crate::png_chunk::*;

pub struct Png {
    chunks : Vec<PngChunk>
}

impl Png {
    pub fn new() -> Png{
        Png{
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

    pub fn from_bytes(buffer: Vec<u8>) -> Result<Png, &'static str> {
        let mut out_png = Png::new();
        //Verify that the first 8 bytes specify a png file!
        let png_header = [137u8, 80u8, 78u8, 71u8, 13u8, 10u8, 26u8, 10u8];
        
        if buffer.iter().take(8).eq(&png_header) {
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
            
            //ALL SLICES ARE ABLE TO BE UNRWRAPPED INTO ARRAYS BECAUSE THERE ARE AT LEAST 12 BYTES
            //IN THE BUFFER.

            //Get length of chunk data, which is stored in the first 4 bytes of the chunk
            let chunk_length_bytes = &buffer_mut[..4];
            let chunk_length: usize = u32::from_be_bytes(chunk_length_bytes.try_into().unwrap()) as usize;  
            
            //Get type of chunk, which is stored in the 4th-8th bytes of the chunk
            let chunk_type_bytes = &buffer_mut[4..8];
            let chunk_type_string = String::from_utf8_lossy(chunk_type_bytes);
            let chunk_type = ChunkType::type_from_bytes(chunk_type_bytes.try_into().unwrap());
            //Get chunk data
            let chunk_data = &buffer_mut[8..8+chunk_length];
            
            //Get chunk data's CRC code, stored in the last four bytes of the chunk
            let chunk_crc_bytes = &buffer_mut[8+chunk_length..12+chunk_length];
            
            //Create new chunk:
            let png_chunk = PngChunk::new(
                chunk_length,
                chunk_type,
                chunk_data.to_vec(),
                chunk_crc_bytes.try_into().unwrap(),
            );

            //Verify CRC
            match png_chunk.verify_crc() {
                true => out_png.add_chunk(png_chunk),
                false => return Err("Invalid CRC!")
            };
            
                        
            //println!("TYPE {}, LENGTH {}", chunk_type_string, chunk_length);
            //Update buffer
            buffer_mut = &buffer_mut[12 + chunk_length..];
        }
        Ok(out_png)
    }
}
