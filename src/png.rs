use crate::pixel::Pixels;
use crate::utils::Deflate;
use std::fs;
use crate::utils::CRC32;



pub enum Png {
    Decoded(DecPng),
    Encoded(EncPng),
}

pub struct PngChunk {
    chunk_length: usize,
    chunk_type: ChunkType,
    chunk_data: Vec<u8>,
    chunk_crc: [u8; 4],
}

#[derive(PartialEq)]
pub enum ChunkType {
    IHDR,
    PLTE,
    IDAT,
    IEND,
    Unknown,
}

impl PngChunk {
    pub fn verify_crc(&self) -> Result<bool, &'static str> {
        let chunk_data = self.get_data();
        let mut crc_data = ChunkType::bytes_from_type(self.get_type())?.to_vec();
        crc_data.append(&mut chunk_data.clone());

        match CRC32::png_crc(crc_data) {
            Ok(x) if x == self.get_crc() => Ok(true),
            Ok(_) => Ok(false),
            Err(x) => Err(x),
        }
    }

    pub fn new(c_length: usize, c_type: ChunkType, c_data: Vec<u8>, c_crc: [u8; 4]) -> PngChunk {
        PngChunk {
            chunk_length: c_length,
            chunk_type: c_type,
            chunk_data: c_data,
            chunk_crc: c_crc,
        }
    }

    pub fn get_length(&self) -> usize {
        self.chunk_length
    }

    pub fn get_type(&self) -> &ChunkType {
        &self.chunk_type
    }

    pub fn get_data(&self) -> &Vec<u8> {
        &self.chunk_data
    }

    pub fn get_crc(&self) -> [u8; 4] {
        self.chunk_crc
    }

    pub fn decompress(&mut self) -> Result<(), &'static str> {
        Err("DECOMPRESSION FAILED!")
    }
}

impl ChunkType {
    pub fn type_from_bytes(bytes: [u8; 4]) -> ChunkType {
        match bytes {
            [73u8, 72u8, 68u8, 82u8] => ChunkType::IHDR,
            [80u8, 76u8, 84u8, 69u8] => ChunkType::PLTE,
            [73u8, 68u8, 65u8, 84u8] => ChunkType::IDAT,
            [73u8, 69u8, 78u8, 68u8] => ChunkType::IEND,
            _ => ChunkType::Unknown,
        }
    }
    pub fn bytes_from_type(chunktype: &ChunkType) -> Result<[u8; 4], & 'static str> {
        match chunktype {
            ChunkType::IHDR => Ok([73u8, 72u8, 68u8, 82u8]),
            ChunkType::PLTE => Ok([80u8, 76u8, 84u8, 69u8]),
            ChunkType::IDAT => Ok([73u8, 68u8, 65u8, 84u8]),
            ChunkType::IEND => Ok([73u8, 69u8, 78u8, 68u8]),
            ChunkType::Unknown => Err("ChunkType::Unknown has no defined bytes!"),
        }
    }
}

impl std::fmt::Display for ChunkType {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        match self {
            ChunkType::IHDR => write!(f, "IHDR"),
            ChunkType::IDAT => write!(f, "IDAT"),
            ChunkType::IEND => write!(f, "IEND"),
            ChunkType::PLTE => write!(f, "PLTE"),
            ChunkType::Unknown => write!(f, "Unknown"),
        }
    }
}

impl std::fmt::Display for PngChunk {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        let mut crc_str = "".to_string();
        for byte in self.get_crc() {
            crc_str += &format!("{} ", byte);
        }

        let display_str = format!(
            "TYPE : {}, LENGTH : {}, CRC : {}",
            self.get_type(),
            self.get_length(),
            crc_str
        );
        write!(f, "{}", display_str)
    }
}
pub struct EncPng {
    chunks: Vec<PngChunk>,
}

impl EncPng {
    pub fn new() -> EncPng {
        EncPng { chunks: vec![] }
    }

    pub fn add_chunk(&mut self, chunk: PngChunk) {
        self.chunks.push(chunk);
    }

    pub fn print_chunks(&self) {
        for (num, chunk) in self.chunks.iter().enumerate() {
            println!("Chunk {}, {}", num, chunk);
        }
    }

    pub fn decompress(self) -> Result<DecPng, &'static str> {
        DecPng::try_from(self)
    }

    pub fn get_deflate_stream(&self) -> Vec<u8> {
        let mut deflate_stream: Vec<u8> = vec![];

        for chunk in self.chunks.iter() {
            match chunk.get_type() {
                ChunkType::IDAT => deflate_stream.append(&mut chunk.get_data().clone()),
                _ => continue,
            };
        }
        println!("{}", deflate_stream.len());
        deflate_stream
    }

    pub fn get_width(&self) -> u32 { 
        self.chunks
            .iter()
            .find(|x| *x.get_type() == ChunkType::IHDR)
            .unwrap()
            .get_data()
            .clone()
            .into_iter()
            .take(4)
            .fold(0u32, |width, x| (width << 8) + x as u32)
    }

    pub fn get_height(&self) -> u32 {
        self.chunks
            .iter()
            .find(|x| *x.get_type() == ChunkType::IHDR)
            .unwrap()
            .get_data()
            .clone()
            .into_iter()
            .skip(4)
            .take(4)
            .fold(0u32, |height, x| (height << 8) + x as u32)
    }

    pub fn get_pixel_depth(&self) -> u32 { 
        self.chunks
            .iter()
            .find(|x| *x.get_type() == ChunkType::IHDR)
            .unwrap()
            .get_data()
            .clone()
            .into_iter()
            .skip(8)
            .take(1)
            .fold(0u32, |depth, x| (depth << 8) + x as u32)
    }

    pub fn get_color_type(&self) -> u32 {
        self.chunks
            .iter()
            .find(|x| *x.get_type() == ChunkType::IHDR)
            .unwrap()
            .get_data()
            .clone()
            .into_iter()
            .skip(9)
            .take(1)
            .fold(0u32, |depth, x| (depth << 8) + x as u32)
    }

    pub fn get_interlace_type(&self) ->u32 {
        self.chunks
            .iter()
            .find(|x| *x.get_type() == ChunkType::IHDR)
            .unwrap()
            .get_data()
            .clone()
            .into_iter()
            .skip(12)
            .take(1)
            .fold(0u32, |depth, x| (depth << 8) + x as u32)
    }
}

impl TryFrom<Vec<u8>> for EncPng {
    type Error = &'static str;

    fn try_from(buffer: Vec<u8>) -> Result<Self, Self::Error> {
        let mut out_png = EncPng::new();

        const PNG_HEADER: [u8; 8] = [137u8, 80u8, 78u8, 71u8, 13u8, 10u8, 26u8, 10u8];

        if buffer.iter().take(8).eq(&PNG_HEADER) {
            println!("Valid PNG header.");
        } else {
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
            let chunk_length: usize =
                u32::from_be_bytes(chunk_length_bytes.try_into().unwrap()) as usize;

            //Type of chunk is stored in the 4th-8th bytes of the chunk
            let chunk_type_bytes = &buffer_mut[4..8];
            let chunk_type_string = String::from_utf8_lossy(chunk_type_bytes);
            let chunk_type = ChunkType::type_from_bytes(chunk_type_bytes.try_into().unwrap());

            //Every byte between type and CRC is chunk data
            let chunk_data = &buffer_mut[8..8 + chunk_length];

            //CRC code is stored in the last four bytes of the chunk
            let chunk_crc_bytes = &buffer_mut[8 + chunk_length..12 + chunk_length];

            let png_chunk = PngChunk::new(
                chunk_length,
                chunk_type,
                chunk_data.to_vec(),
                chunk_crc_bytes.try_into().unwrap(),
            );
            
            match png_chunk.verify_crc() {
                Ok(false) => return Err("Invalid CRC!"),
                Ok(_) => (), 
                Err(_) if *png_chunk.get_type() == ChunkType::Unknown => (),
                Err(x) => return Err(x),
            };

            out_png.add_chunk(png_chunk);

            buffer_mut = &buffer_mut[12 + chunk_length..];
        }
        Ok(out_png)
    }
}
pub struct DecPng {
    scanlines: Vec<Pixels>,
}

impl DecPng {
    pub fn new() -> DecPng {
        DecPng { scanlines: vec![] }
    }

    pub fn set_scanlines(&mut self, scanlines: Vec<Pixels>) {
        self.scanlines = scanlines;
    }
}

impl From<Vec<Pixels>> for DecPng {
    fn from(scanlines: Vec<Pixels>) -> Self {
        DecPng { scanlines }
    }
}

impl TryFrom<EncPng> for DecPng {
    type Error = &'static str;

    fn try_from(encpng: EncPng) -> Result<Self, Self::Error> {
        //let scanlines = encpng.get_deflate_stream().decompress().scalines().defilter()
        let (height, width, depth, color, il) = (encpng.get_height(), encpng.get_width(), encpng.get_pixel_depth(), encpng.get_color_type(), encpng.get_interlace_type());
        
        let bpp = match depth / 8 {
            0 => 1,
            _ => depth / 8,
        } as usize;

        println!("PNG DIMENSIONS : width {} height {}", width, height);
        println!("depth {} bpp {} color {} il {}", depth, bpp, color, il);

        let compressed_stream = encpng.get_deflate_stream();
        let decoded_stream = Deflate::decompress(compressed_stream)?;
        println!("{}", decoded_stream.len());
        let defiltered_stream = Deflate::defilter(decoded_stream, height, width, bpp)?;
        
        println!("depth {} bpp {} color {} il {}", depth, bpp, color, il);

        let mut write_string = format!("P3\n{} {}\n {}\n", width, height, 255);

        let mut char_count = 0;
        for scanline in defiltered_stream {

            /*
            let skip_take = |obj : &Vec<u8>, skip : usize, n: usize| obj.clone().into_iter().skip(skip * n).take(n);

            let channels : Vec<(u8,u8,u8)> = skip_take(&scanline, 0, 32)
                                        .zip(skip_take(&scanline, 1, 32))
                                        .zip(skip_take(&scanline, 2, 32))
                                        .map(|((x,y), z)| (x,y,z))
                                        .collect(); 
            */

            for pattern in scanline.chunks(3) {
                let triple_str = match pattern {
                    &[r,g,b] => format!("{r} {g} {b}  "),
                    _ => String::from(" "),
                };

                if char_count + 13 > 70 {
                    write_string.push_str("\n");
                    write_string.push_str(&triple_str);
                    char_count = 0;
                    continue;
                }

                char_count += 13;
                write_string.push_str(&triple_str);
            }
            write_string.push_str("\n");
        }
        
        fs::write("out.ppm", write_string).expect("Unable to write file");

        Ok(DecPng::new())
    }
}




