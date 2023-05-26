use crate::crc;

pub struct PngChunk {
    chunk_length : usize,
    chunk_type : ChunkType,
    chunk_data : Vec<u8>,
    chunk_crc : [u8; 4],
}

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
        let mut crc_data = ChunkType::bytes_from_type(self.get_type()).to_vec();
        crc_data.append(&mut chunk_data.clone());
        
        match crc::png_crc(crc_data) {
            Ok(x) if x == self.get_crc() => Ok(true),
            Ok(_) => Ok(false),
            Err(x) => Err(x),
        }
    }

    pub fn new(c_length: usize, c_type: ChunkType, c_data: Vec<u8>, c_crc: [u8;4]) -> PngChunk {
        PngChunk {
            chunk_length : c_length,
            chunk_type : c_type,
            chunk_data : c_data,
            chunk_crc : c_crc, 
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
    
    pub fn decompress(&mut self) -> Result<(), & 'static str>{
        Err("DECOMPRESSION FAILED!")
    }
}

impl ChunkType{
    pub fn type_from_bytes(bytes : [u8; 4]) -> ChunkType {
        match bytes {
            [73u8, 72u8, 68u8, 82u8] => ChunkType::IHDR,
            [80u8, 76u8, 84u8, 69u8] => ChunkType::PLTE,
            [73u8, 68u8, 65u8, 84u8] => ChunkType::IDAT,
            [73u8, 69u8, 78u8, 68u8] => ChunkType::IEND,
            _ => ChunkType::Unknown,
        }
    }
    pub fn bytes_from_type(chunktype: &ChunkType) -> [u8; 4] {
        match chunktype {
            ChunkType::IHDR => [73u8, 72u8, 68u8, 82u8],
            ChunkType::PLTE => [80u8, 76u8, 84u8, 69u8], 
            ChunkType::IDAT => [73u8, 68u8, 65u8, 84u8], 
            ChunkType::IEND => [73u8, 69u8, 78u8, 68u8],
            ChunkType::Unknown => panic!("ChunkType::Unknown has no defined bytes!"),           
        }
    }
}

impl std::fmt::Display for ChunkType {
    fn fmt(&self, f :&mut std::fmt::Formatter) -> std::fmt::Result {
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
    fn fmt(&self, f :&mut std::fmt::Formatter) -> std::fmt::Result {
        let mut crc_str = "".to_string();
        for byte in self.get_crc() {
            crc_str += &format!("{} ", byte);
        }

        let display_str = format!("TYPE : {}, LENGTH : {}, CRC : {}", self.get_type(), self.get_length(), crc_str);
        write!(f, "{}", display_str)
    }
}
