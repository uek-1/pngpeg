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
        let mut write_bytes : Vec<u8> = vec![];
        let lines_in_image = 0;
        let samples_per_line = 0;
        // SOI
        write_bytes.append(&mut vec![0xFF, 0xD8]);
        
        // APP0
        write_bytes.append(&mut vec![0xFF, 0xE0]);
        // Length [2] (16)
        write_bytes.append(&mut vec![0, 16]);
        // Identifier  [5]
        write_bytes.append(&mut vec![0x4A, 0x46, 0x49, 0x46, 0x00]);
        // Version [2]
        write_bytes.append(&mut vec![0x01, 0x02]);
        // Density [1]
        write_bytes.push(0x00);
        // XDensity [2]
        write_bytes.append(&mut vec![0, 1]);
        // YDensity [2]
        write_bytes.append(&mut vec![0, 1]);
        // X,Y Thumbnail [1] + [1]
        write_bytes.append(&mut vec![0, 0]);

        // Frame :
        let mut frame_bytes : Vec<u8> = vec![];

        // Frame -> Header (SOF0)
        frame_bytes.append(&mut vec![0xFF, 0xC0]);
        // Lf (length [2]) (17)
        frame_bytes.append(&mut vec![0,17]);
        // P (sample precision[1])
        frame_bytes.push(8);
        // Y (lines in image [2])
        frame_bytes.push(lines_in_image);
        // X (samples per line [2])
        frame_bytes.push(samples_per_line);
        // Nf (components [1])
        frame_bytes.push(3);

        // Frame -> Header (SOF0) -> COMPONENTS :
        // C0 (Luma)
        frame_bytes.push(0);
        // Hi, Vi (4,4)
        frame_bytes.push(0b01000100);
        // Tqi (Luma quantization table - 0)
        frame_bytes.push(0);
        
        // C1 (Chroma B)
        frame_bytes.push(1);
        // Hi, Vi (2,2)
        frame_bytes.push(0b00100010);
        // Tqi (Chroma quantization table - 1)
        frame_bytes.push(1);
        
        // C2 (Chroma R)
        frame_bytes.push(1);
        // Hi, Vi (2,2)
        frame_bytes.push(0b00100010);
        // Tqi (Chroma quantization table - 1)
        frame_bytes.push(1);
        
        // Scan 1 (only 1 scan in baseline jpeg)
        let mut scan_bytes : Vec<u8> = vec![];
        
        // SOS
        scan_bytes.append(&mut vec![0xFF, 0xDA]);
        // Length [2] (14)
        scan_bytes.append(&mut vec![0, 14]);
        // Ns [1] (3)
        scan_bytes.push(3);
        // Cs1, Td1, Ta1 [2]
        scan_bytes.push(0);
        scan_bytes.push(0);
        // Cs2, Td2, Ta2 [2];
        scan_bytes.push(1);
        scan_bytes.push(0b00010001);
        // Cs3, Td3, Ta3 [2];
        scan_bytes.push(2);
        scan_bytes.push(0b00010001);
        // SS, Se , AH, Al [3]
        scan_bytes.append(&mut vec![0, 63, 0]);

        // Entropy Coded MCU segment 

        frame_bytes.append(&mut scan_bytes);
        write_bytes.append(&mut frame_bytes);

        // EOI
        write_bytes.append(&mut vec![0xFF, 0xD9]);
        
        std::fs::write(path, write_bytes);
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

#[cfg(test)]
mod test {
    use super::EncJpeg;

    //#[test]
    fn write_jpeg_test() {
        let enc = EncJpeg {};
        enc.write_to_path("test.jpeg".to_string());
    }
}
