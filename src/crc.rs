use crate::bits::Bits;

pub fn png_crc(bytes: Vec<u8>) -> Result<[u8; 4], &'static str> {
    const POLY: u32 = 0x04C11DB7;
    const XOROUT: u32 = 0xFFFFFFFF;
    const INIT: u32 = 0xFFFFFFFF;
    const REFIN: bool = true;
    const REFOUT: bool = true;

    let mut padding = vec![0u8; 4];
    let mut padded_bytes = bytes;
    padded_bytes.append(&mut padding);
    let mut padded_bits = Bits::new(padded_bytes, REFIN);

    let mut register = match padded_bits.read_bits(32) {
        Some(x) => x,
        None => return Err("Error while reading initial 32 bits into register. It's likely that the padding failed."),
    };

    //INIT value - see https://stackoverflow.com/questions/43823923/implementation-of-crc-8-what-does-the-init-parameter-do:
    register ^= INIT;

    while let Some(next_bit) = padded_bits.read_bits(1) {
        let popped_bit = register >> 31;
        register = (register << 1) + next_bit;

        match popped_bit {
            0 => (),
            _ => register ^= POLY,
        };
    }
    if REFOUT {
        register = register.reverse_bits();
    }

    register ^= XOROUT;

    Ok(register.to_be_bytes())
}

