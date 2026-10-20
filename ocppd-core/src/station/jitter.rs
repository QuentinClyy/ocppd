const FNV_OFFSET: u64 = 0xcbf2_9ce4_8422_2325;
const FNV_PRIME: u64 = 0x0000_0100_0000_01b3;

pub fn fraction(station_id: &str, salt: u64) -> f64 {
    let mut hash = FNV_OFFSET;
    for byte in station_id.bytes().chain(salt.to_le_bytes()) {
        hash ^= u64::from(byte);
        hash = hash.wrapping_mul(FNV_PRIME);
    }
    hash ^= hash >> 33;
    hash = hash.wrapping_mul(0xff51_afd7_ed55_8ccd);
    hash ^= hash >> 33;
    (hash >> 11) as f64 / (1_u64 << 53) as f64
}

#[cfg(test)]
#[path = "jitter.test.rs"]
mod tests;
