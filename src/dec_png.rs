use crate::pixel::Pixel;

pub struct DecPng {
    scanlines: Vec<Pixel>,
}

impl DecPng {
    pub fn new() -> DecPng {
        DecPng { scanlines: vec![] }
    }

    pub fn set_scanlines(&mut self, scanlines: Vec<Pixel>) {
        self.scanlines = scanlines;
    }
}

impl From<Vec<Pixel>> for DecPng {
    fn from(scanlines: Vec<Pixel>) -> Self {
        DecPng { scanlines }
    }
}

impl TryFrom<Vec<u8>> for DecPng {
    type Error = &'static str;

    fn try_from(unfiltered_stream: Vec<u8>) -> Result<Self, Self::Error> {
        Ok(DecPng::new())
    }
}
