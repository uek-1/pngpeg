use crate::{pixel::{Pixel, Pixels, ColorType}, png::DecPng};
use std::fs;
use crate::utils;

pub struct EncJpeg {

}

impl EncJpeg {
    pub fn write_to_path(&self, path : String) {
        
    }
}

impl TryFrom<DecJpeg> for EncJpeg {
    type Error = &'static str;

    fn try_from(value: DecJpeg) -> Result<Self, Self::Error> {
        //subsample
        //DCT
        //quantize
        //RLE wiht entropy coding
        //Huffman code
        Err("Not implmented yet!") 
    }
}

pub struct DecJpeg {
    pixels : Pixels
}

impl TryFrom<DecPng> for DecJpeg {
    type Error = &'static str;

    fn try_from(value: DecPng) -> Result<Self, Self::Error> {
        let pixels = value.get_scanlines().to_ycbcr();

        Ok(
            DecJpeg {
                pixels
            }
        )
    }
} 
