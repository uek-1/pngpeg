use std::fs::File;
use std::io;
use std::io::BufReader;
use std::io::Read;

mod enc_png;
use enc_png::EncPng;

mod crc;
mod dec_png;
use dec_png::DecPng;
mod deflate;
mod pixel;
mod png_chunk;
mod bits;

struct Cli {
    path: String,
}

fn main() -> io::Result<()> {
    let args = Cli {
        path: std::env::args().nth(1).expect("No file specified!")
    };

    let f = File::open(args.path)?;
    let mut reader = BufReader::new(f);
    let mut buffer = vec![];

    reader.read_to_end(&mut buffer);
    let buffer = buffer;

    let png_file: EncPng = buffer.try_into().expect("Couldn't read PNG file!");

    png_file.print_chunks();
    let dec_png_file : DecPng = png_file.decompress().expect("Couldn't decompress PNG file");


    Ok(())
}
