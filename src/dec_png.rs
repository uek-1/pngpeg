use crate::pixel::Pixel;
use crate::enc_png::EncPng;
use crate::deflate;

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
        let (height, width, depth) = (encpng.get_height(), encpng.get_width(), encpng.get_pixel_depth());
        
        let bpp = match (depth / 8) {
            0 => 1,
            _ => depth / 8,
        } as usize;

        println!("PNG DIMENSIONS : width {} height {}", width, height);

        let compressed_stream = encpng.get_deflate_stream();
        let decoded_stream = deflate::decompress(compressed_stream)?;
        let defiltered_stream = deflate::defilter(decoded_stream, height, width, bpp)?;
        
        Ok(DecPng::new())
    }
}


struct Pixels(Vec<Pixel>);

impl TryFrom<Vec<u8>> for Pixels {
    type Error = &'static str;

    fn try_from(defiltered_stream: Vec<u8>) -> Result<Self, Self::Error> {
        Ok(Pixels(vec![]))
    }
}
