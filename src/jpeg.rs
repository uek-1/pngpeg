use crate::{pixel::{Pixel, Pixels, ColorType}, png::DecPng};
use std::fs;
use crate::utils;

pub struct JpegBlock {
    luma : Vec<Vec<u8>>,
    diff_blue : Vec<Vec<u8>>,
    diff_red : Vec<Vec<u8>>,
}

impl JpegBlock {
    pub fn new(luma : Vec<Vec<u8>>, diff_blue : Vec<Vec<u8>>, diff_red : Vec<Vec<u8>>) -> JpegBlock {
        //4 : 2 : 0 only
        JpegBlock {
            luma,
            diff_blue,
            diff_red
        }
    } 
}

pub struct JpegBlocks(Vec<JpegBlock>);

impl From<Pixels> for JpegBlocks {
    
    fn from(pixels : Pixels) -> JpegBlocks {
        //expects 4:2:0 compression
        let blocks = vec![];
        JpegBlocks(blocks)
    }
}

pub struct EncJpeg {

}

impl EncJpeg {
    pub fn write_to_path(&self, path : String) {
        //SOI, APP0 <metadata> , ...
        
    }
}

impl TryFrom<DecJpeg> for EncJpeg {
    type Error = &'static str;

    fn try_from(decpng: DecJpeg) -> Result<Self, Self::Error> {
        //subsample
        let subsampled = decpng.pixels.subsample_ycbcr();
        let blocks = JpegBlocks::from(subsampled);
        //DCT
        //quantize
        //RLE wiht entropy coding
        //let entropy = blocks.dct().quantize().to_entropy()
        //encode(entropy)
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
