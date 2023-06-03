pub struct Pixel {}

impl Pixel {}


pub struct Pixels(Vec<Pixel>);

impl TryFrom<Vec<u8>> for Pixels {
    type Error = &'static str;

    fn try_from(defiltered_stream: Vec<u8>) -> Result<Self, Self::Error> {
        Ok(Pixels(vec![]))
    }
}
