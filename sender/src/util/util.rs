

// Buffer to string from UDP packet
pub fn read_to_string(buf: &[u8]) -> String {
    for ind in 0..buf.len() {
        if buf[ind] == 0 {
            return String::from_utf8_lossy(&buf[..ind]).to_string();
        }
    }
    String::from_utf8_lossy(&buf).to_string()
}

// Safe increment for sequence numbers
pub fn safe_increment(cur_seq: u32, add_bytes: u32) -> u32 {
    let dst_to_wrap = u32::MAX - cur_seq;

    if dst_to_wrap < add_bytes {
        add_bytes - dst_to_wrap
    } else {
        cur_seq + add_bytes
    }
}