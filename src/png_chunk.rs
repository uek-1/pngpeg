
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
    pub fn verify_crc(&self) -> bool { 
        true
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

    pub fn type_from_bytes(bytes : &[u8]) -> ChunkType {
        match bytes {
            [73u8, 72u8, 68u8, 82u8] => ChunkType::IHDR,
            [80u8, 76u8, 84u8, 69u8] => ChunkType::PLTE,
            [73u8, 68u8, 65u8, 84u8] => ChunkType::IDAT,
            [73u8, 69u8, 78u8, 68u8] => ChunkType::IEND,
            _ => ChunkType::Unknown,
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
