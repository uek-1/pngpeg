use std::fs::File;
use std::io;
use std::io::{BufReader, Read};

mod png;
use png::{EncPng, DecPng, WriteToPPM};
mod jpeg;
use jpeg::{EncJpeg, DecJpeg};
mod utils;
mod pixel;

struct Cli {
    input_path: String,
    output_path : String,
}

fn main() -> io::Result<()> {
    let args = Cli {
        input_path: std::env::args().nth(1).expect("No input png filename specified!"),
        output_path: std::env::args().nth(2).expect("No output jpeg filename specified!")
    };

    let f = File::open(args.input_path)?;
    let mut reader = BufReader::new(f);
    let mut buffer = vec![];

    reader.read_to_end(&mut buffer)?;
    let buffer = buffer;

    let png_file: EncPng = buffer.try_into().expect("Couldn't read PNG file!");

    png_file.print_chunks();
    let dec_png_file : DecPng = png_file.decompress().expect("Couldn't decompress PNG file");
    dec_png_file.write_to_p3(args.output_path);


    Ok(())
}
