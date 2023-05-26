use crate::pixel::Pixel;

pub struct DecPng {
    scanlines: Vec<Pixel>        
}

impl DecPng {
    pub fn new() -> DecPng{
        DecPng {
            scanlines : vec![]
        }
    }
}


