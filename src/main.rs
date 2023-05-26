use std::io;
use std::io::Read;
use std::io::BufReader;
use std::fs::File;

mod enc_png;
use enc_png::EncPng;

mod png_chunk;
mod crc;
mod dec_png;
mod pixel;
mod deflate;

struct Cli {
    path: String,
}

fn main() -> io::Result<()> {
    let args = Cli {
        path : match std::env::args().nth(1) {
            Some(x) => x,
            None => panic!("No file specified!"),
        },
    };

    let f = File::open(args.path)?;
    let mut reader = BufReader::new(f);
    let mut buffer = vec![];
  
    reader.read_to_end(&mut buffer);
    let buffer = buffer;

    let png_file : EncPng = match buffer.try_into() {
        Ok(x) => x,
        Err(e) => panic!("{}", e),
    };
    
    png_file.print_chunks();
    
    Ok(())
}


