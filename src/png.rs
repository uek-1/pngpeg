use crate::pixel::{Pixel, Pixels, ColorType};
use std::fs;
use crate::utils;
use crate::utils::Defilter;

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

        match utils::png_crc(crc_data) {
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

    pub fn get_plte_bytes(&self) -> Result<Vec<u8>, &'static str> {
        let plte_bytes = self.chunks
            .iter()
            .find(|x| *x.get_type() == ChunkType::PLTE);
        
        match plte_bytes {
            Some(chunk) => Ok(chunk.get_data().clone()),
            None => Err("Couldn't find PLTE chunk!")
        }
    }
    
    fn get_ihdr_info(&self, start: usize, bytes: usize) -> Result<u32, &'static str> {
        let info : Vec<u8> = self.chunks
            .iter()
            .find(|x| *x.get_type() == ChunkType::IHDR)
            .expect("IHDR not found!")
            .get_data()
            .clone()
            .into_iter()
            .skip(start)
            .collect();

        if info.len() < bytes {
            return Err("Not enough bytes in IHDR to read info from given start");
        }

        Ok(info
            .into_iter()
            .take(bytes)
            .fold(0u32, |width, x| (width << 8) + x as u32)
        )
    } 

    pub fn get_width(&self) -> Result<u32, &'static str> { 
       self.get_ihdr_info(0, 4) 
    }

    pub fn get_height(&self) -> Result<u32, &'static str> {
        self.get_ihdr_info(4, 4)
    }

    pub fn get_pixel_depth(&self) -> Result<u32, &'static str> { 
        self.get_ihdr_info(8, 1)
    }

    pub fn get_color_type(&self) -> Result<u32, &'static str> {
        self.get_ihdr_info(9, 1)
    }

    pub fn get_interlace_type(&self) ->Result<u32 , &'static str>{
        self.get_ihdr_info(12, 1)
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
    scanlines: Pixels,
}

impl DecPng {
    pub fn new() -> DecPng {
        DecPng { scanlines: Pixels::new() }
    }

    pub fn set_scanlines(&mut self, scanlines: Vec<Vec<Pixel>>) {
        self.scanlines = Pixels::from(scanlines);
    }

    pub fn get_scanlines(&self) -> Pixels {
        self.scanlines.clone()
    }

}

impl From<Pixels> for DecPng {
    fn from(scanlines: Pixels) -> Self {
        DecPng { scanlines }
    }
}

impl TryFrom<EncPng> for DecPng {
    type Error = &'static str;

    fn try_from(encpng: EncPng) -> Result<Self, Self::Error> {
        //let scanlines = encpng.get_deflate_stream().decompress().scalines().defilter()
        let (height, width, bit_depth, color, il) = (encpng.get_height()?, encpng.get_width()?, encpng.get_pixel_depth()?, encpng.get_color_type()?, encpng.get_interlace_type()?);
        let channels : usize = match color {
            0 => 1,
            2 => 3,
            3 => 1,
            4 => 2,
            6 => 4,
            _ => return Err("INVALID COLOR TYPE!")
        };

        let plte_bytes : Vec<u8> = match color {
            3 => encpng.get_plte_bytes()?,
            _ => vec![],
        };

        println!("PNG DIMENSIONS : width {} height {}", width, height);
        println!("depth {} bpp {} color {} channels {} il {}", bit_depth, bit_depth / 8, color, channels, il);

        let compressed_stream : Vec<u8> = encpng.get_deflate_stream();
        let decompressed_stream : Vec<u8> = utils::decompress(compressed_stream)?;

        println!("Decompressed bytes {}", decompressed_stream.len());
        
        let filtered_scanlines : Vec<Vec<u8>> = utils::decompressed_to_scanlines(decompressed_stream, height);
        
        let mut defilter = Defilter::new(channels, bit_depth, filtered_scanlines);

        let defiltered_scanlines : Vec<Vec<u8>> = defilter.defilter()?;

        println!("depth {} bpp {} color {} il {}", bit_depth, bit_depth / 8, color, il);

        let scanlines = utils::defiltered_to_pixels(defiltered_scanlines, color as usize);

        let scanlines_decoded_plte = match scanlines[0][0].get_color_type() {
            ColorType::PLTE => scanlines.decode_plte(plte_bytes), 
            _ => scanlines,
        };

        Ok(DecPng::from(scanlines_decoded_plte))
    }
}

impl WriteToPPM for DecPng {
    fn write_to_p3(&self, path: String) {
        let rgb_pixels : Pixels = self.scanlines.to_rgb();
        let mut write_string = format!("P3\n{} {}\n{}\n", rgb_pixels.len(), rgb_pixels[0].len(), 255);
        let mut char_count = 0;

        for row in rgb_pixels.iter() {
            for pixel in row {
                let triple_str : String = match pixel.get_color_values().as_slice() { 
                    &[r, g ,b] => format!("{r} {g} {b}  "),
                    _ => String::from(" "),
                };

                if char_count + 13 > 70 {
                    write_string.push_str("\n");
                    write_string.push_str(&triple_str);
                    char_count = 0;
                    continue;
                }

                write_string.push_str(&triple_str);
                char_count += 13;
            }
            write_string.push_str("\n");
        }
        fs::write(&path, write_string).expect("Unable to write file");
    }
}

pub trait WriteToPPM {
    fn write_to_p3(&self, path: String);
}


