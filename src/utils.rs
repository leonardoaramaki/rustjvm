/// Convert a byte array or slice to a [u32].
pub fn slice_as_u32(bytes: &[u8], s: usize) -> u32 {
    u32::from_be_bytes(bytes[s..s + 4].try_into().unwrap())
}

pub fn slice_as_i32(bytes: &[u8], s: usize) -> i32 {
    i32::from_be_bytes(bytes[s..s + 4].try_into().unwrap())
}

pub fn slice_as_f32(bytes: &[u8], s: usize) -> f32 {
    f32::from_be_bytes(bytes[s..s + 4].try_into().unwrap())
}

/// Convert a byte array or slice to a [u16].
pub fn slice_as_u16(bytes: &[u8], s: usize) -> u16 {
    u16::from_be_bytes(bytes[s..s + 2].try_into().unwrap())
}
