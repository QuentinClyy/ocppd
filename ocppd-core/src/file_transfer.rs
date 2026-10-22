pub const DOWNLOAD_MS: u64 = 10_000;
pub const UPLOAD_MS: u64 = 10_000;
const SCHEMES: [&str; 4] = ["http://", "https://", "ftp://", "ftps://"];

pub fn reachable(location: &str) -> bool {
    SCHEMES.iter().any(|scheme| location.starts_with(scheme))
}
