use std::io;
use std::io::Read;
use std::io::BufReader;
use std::fs::File;

mod png_file;
use png_file::EncPng;

mod png_chunk;
mod crc;
mod png_stream;

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

    let png_file = match EncPng::from_bytes(buffer) {
        Ok(x) => x,
        Err(e) => panic!("{}", e),
    };
    
    png_file.print_chunks();
    
    Ok(())
}


