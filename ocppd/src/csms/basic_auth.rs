const ALPHABET: &[u8; 64] = b"ABCDEFGHIJKLMNOPQRSTUVWXYZabcdefghijklmnopqrstuvwxyz0123456789+/";

pub fn header(user: &str, password: &[u8]) -> String {
    let credentials = [user.as_bytes(), b":", password].concat();
    format!("Basic {}", base64(&credentials))
}

fn base64(bytes: &[u8]) -> String {
    let mut encoded = String::with_capacity(bytes.len().div_ceil(3) * 4);
    for chunk in bytes.chunks(3) {
        let padded = [
            chunk[0],
            *chunk.get(1).unwrap_or(&0),
            *chunk.get(2).unwrap_or(&0),
        ];
        let bits = u32::from_be_bytes([0, padded[0], padded[1], padded[2]]);
        for position in 0..4 {
            if position <= chunk.len() {
                let index = (bits >> (18 - 6 * position)) & 0x3f;
                encoded.push(ALPHABET[index as usize] as char);
            } else {
                encoded.push('=');
            }
        }
    }
    encoded
}

#[cfg(test)]
#[path = "basic_auth.test.rs"]
mod tests;
