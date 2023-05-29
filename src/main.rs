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
        path: match std::env::args().nth(1) {
            Some(x) => x,
            None => panic!("No file specified!"),
        },
    };

    let f = File::open(args.path)?;
    let mut reader = BufReader::new(f);
    let mut buffer = vec![];

    reader.read_to_end(&mut buffer);
    let buffer = buffer;

    let png_file: EncPng = match buffer.try_into() {
        Ok(x) => x,
        Err(e) => panic!("{}", e),
    };

    png_file.print_chunks();
    let dec_png_file : DecPng = match png_file.decompress() {
        Ok(x) => x,
        Err(e) => panic!("{}", e),
    };


    Ok(())
}
