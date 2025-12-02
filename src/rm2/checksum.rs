pub fn additive_checksum<'a, I>(bytes: I) -> u8
where
    I: Iterator<Item = &'a u8>,
{
    bytes.fold(0u8, |n, m| n.wrapping_add(*m))
}
