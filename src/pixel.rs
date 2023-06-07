use std::ops::{Deref, DerefMut};

#[derive(Copy, Clone, PartialEq)]
pub enum ColorType {
    GS,
    GSA,
    RGB,
    RGBA,
    PLTE,
    YCbCr,
}

impl ColorType {
    pub fn from_png_color_type(color_type : usize) -> ColorType {
        match color_type {
            0 => ColorType::GS,
            2 => ColorType::RGB,
            3 => ColorType::PLTE,
            4 => ColorType::GSA,
            6 => ColorType::RGBA,
            _ => panic!("Invalid color type!")
        }
    }

    pub fn to_channels(&self) -> usize {
        match self {
            ColorType::GS => 1,
            ColorType::GSA => 2,
            ColorType::RGB => 3,
            ColorType::RGBA => 4,
            ColorType::PLTE => 1,
            ColorType::YCbCr => 3,
        }
    }
}

#[derive(Clone)]
pub struct Pixel {
    color_type : ColorType,
    channels : usize,
    color_values : Vec<u8>
}

impl Pixel {
    pub fn new(color_type: ColorType, data: Vec<u8>) -> Pixel {
        let channels = color_type.to_channels();
        let color_values = data[0..channels].to_vec();

        Pixel {
            color_type, 
            channels,
            color_values 
        } 
    }
    
    fn rgb_to_ycbcr(r: u8, g: u8, b: u8) -> Vec<u8> {
        // SEE: https://learn.microsoft.com/en-us/openspecs/windows_protocols/ms-rdprfx/b550d1b5-f7d9-4a0c-9141-b3dca9d7f525
        // This function uses the above method but with the Cr and Cb values shifted by +128 to
        // make all three values fit into a u8.
        let y_value_f64 = 0.299 * (r as f64) + -0.168935 * (g as f64) + 0.499813 * (b as f64);
        let y_value_u8 = y_value_f64 as u8;

        let cb_value_f64 = 128f64 + 0.587 * (r as f64) + -0.331665 * (g as f64)  + -0.418531 * (b as f64);
        let cb_value_u8 = cb_value_f64 as u8;

        let cr_value_f64 = 128f64 + 0.114 * (r as f64) + 0.50059 * (g as f64)  + -0.081282 * (b as f64);
        let cr_value_u8 = cr_value_f64 as u8;
        
        vec![y_value_u8,cb_value_u8,cr_value_u8]
    }

    fn decode_plte(&mut self) {}
    
    pub fn to_rgb(&self) -> Pixel {
        let rgb_data = match self.color_type {
            ColorType::RGB => self.color_values.clone(),
            ColorType::RGBA => self.color_values.clone()[0..3].to_vec(),
            ColorType::GS | ColorType::GSA => vec![self.color_values[0]; 3],
            _ => panic!(),
        };

        Pixel::new(ColorType::RGB, rgb_data)
    }

    pub fn to_ycbcr(&self) -> Pixel {
        let rgb_self : Pixel = match self.color_type {
            ColorType::RGB => self.clone(),
            _ => self.to_rgb(),
        };
        
        let ycbcr_data : Vec<u8> = match rgb_self.color_values[0..3] {
            [r,g,b] => Self::rgb_to_ycbcr(r, g, b),
            _ => panic!("RGB conversion invalid")
        };

        Pixel::new(ColorType::YCbCr, ycbcr_data)
    }
}


pub struct Pixels(Vec<Vec<Pixel>>);

impl Pixels {
    pub fn new() -> Pixels {
        Pixels(vec![])
    }
}

impl From<Vec<Vec<Pixel>>> for Pixels {
    fn from (pixel_vec : Vec<Vec<Pixel>>) -> Self {
        Pixels(pixel_vec)
    }
}

impl Deref for Pixels {
    type Target = Vec<Vec<Pixel>>;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

impl DerefMut for Pixels { 

    fn deref_mut(&mut self) -> &mut Vec<Vec<Pixel>> {
        &mut self.0
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    
    #[test]
    fn test_rgb_ycbcr() {
        assert_eq!(vec![0, 255, 255], Pixel::rgb_to_ycbcr(255, 255, 255))
    }
}

