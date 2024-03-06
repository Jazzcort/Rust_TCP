// Read in a buffer and return a string up to the first null byte
pub fn read_to_string(buf: &[u8]) -> String {
    for ind in 0..buf.len() {
        if buf[ind] == 0 {
            return String::from_utf8_lossy(&buf[..ind]).to_string();
        }
    }
    String::from_utf8_lossy(&buf).to_string()
}

// Safely increment a sequence number
pub fn safe_increment(cur_seq: u32, add_bytes: u32) -> u32 {
    // Calculate how far we are from wrapping around
    let dst_to_wrap = u32::MAX - cur_seq;

    // Overflow
    if dst_to_wrap < add_bytes {
        add_bytes - dst_to_wrap
    } // No overflow 
    else {
        cur_seq + add_bytes
    }
}