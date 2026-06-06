pub fn checksum(buf: &[u8]) -> u8 {
    let mut sum = 0u8;

    for i in buf {
        sum = sum.wrapping_add(*i);
    }

    0u8.wrapping_sub(sum)
}
