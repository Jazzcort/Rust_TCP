pub fn read_to_string(buf: &[u8]) -> String {
    for ind in 0..buf.len() {
        if buf[ind] == 0 {
            return String::from_utf8_lossy(&buf[..ind]).to_string();
        }
    }
    String::from_utf8_lossy(&buf).to_string()
}