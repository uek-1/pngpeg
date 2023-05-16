
pub struct PngChunk {
    chunk_length : usize,
    chunk_type : ChunkType,
    chunk_data : Vec<u8>,
    chunk_crc : [u8; 4],
}

pub enum ChunkType {
    IHDR,   
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
        ChunkType::IHDR
    }
}

impl std::fmt::Display for ChunkType {
    fn fmt(&self, f :&mut std::fmt::Formatter) -> std::fmt::Result {
        match self {
            ChunkType::IHDR => write!(f, "IHDR"),
        }
    }
}
