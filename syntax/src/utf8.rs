pub fn convert_byte_slice_into_utf8(slice: &[u8]) -> String {
    let slice = Vec::from(slice);
    String::from_utf8(slice).unwrap()
}