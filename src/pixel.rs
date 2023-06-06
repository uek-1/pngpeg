use std::ops::{Deref, DerefMut};

#[derive(Clone, PartialEq)]
pub enum ColorType {
    GS,
    GSA,
    RGB,
    RGBA,
    PLTE,
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
        }
    }
}

pub struct Pixel {
    color_type : ColorType,
    channels : usize,
    color_values : Vec<u32>
}

impl Pixel {
    pub fn new(color_type: ColorType, data: Vec<u32>) -> Pixel {
        let channels = color_type.to_channels();
        let color_values = data[0..channels].to_vec();

        Pixel {
            color_type, 
            channels,
            color_values 
        } 
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

