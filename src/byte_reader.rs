use std::io::{Read, Result, Seek};

pub fn le_u32<R: Read>(input: &mut R) -> Result<u32> {
    let mut buf = [0; 4];
    input.read_exact(&mut buf)?;
    Ok(u32::from_le_bytes(buf))
}

pub fn u24_from_le_bytes(bytes: [u8; 3]) -> u32 {
    u32::from_le_bytes([bytes[0], bytes[1], bytes[2], 0])
}

pub fn le_u24<R: Read>(input: &mut R) -> Result<u32> {
    let mut buf = [0; 3];
    input.read_exact(&mut buf)?;
    Ok(u24_from_le_bytes(buf))
}

pub fn le_u16<R: Read>(input: &mut R) -> Result<u16> {
    let mut buf = [0; 2];
    input.read_exact(&mut buf)?;
    Ok(u16::from_le_bytes(buf))
}

pub fn u8<R: Read>(input: &mut R) -> Result<u8> {
    let mut buf = [0; 1];
    input.read_exact(&mut buf)?;
    Ok(buf[0])
}

pub fn take_const<const C: usize, R: Read>(input: &mut R) -> Result<[u8; C]> {
    let mut buf = [0; C];
    input.read_exact(&mut buf)?;
    Ok(buf)
}

pub fn take<R: Read>(count: usize, input: &mut R) -> Result<Vec<u8>> {
    let mut buf = vec![0; count];
    input.read_exact(&mut buf)?;
    Ok(buf)
}

pub fn skip<S: Seek>(count: i64, input: &mut S) -> Result<()> {
    input.seek_relative(count)
}
