use serde::Deserialize;

pub const DOWNLOAD_MS: u64 = 10_000;
pub const UPLOAD_MS: u64 = 10_000;
const SCHEMES: [&str; 4] = ["http://", "https://", "ftp://", "ftps://"];

pub fn reachable(location: &str) -> bool {
    SCHEMES.iter().any(|scheme| location.starts_with(scheme))
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum TransferStage {
    Download,
    Signature,
    Install,
    Upload,
}

#[derive(Default)]
pub struct FailingTransfers(Vec<TransferStage>);

impl FailingTransfers {
    pub fn add(&mut self, stage: TransferStage) {
        if !self.0.contains(&stage) {
            self.0.push(stage);
        }
    }

    pub fn take(&mut self, stage: TransferStage) -> bool {
        let before = self.0.len();
        self.0.retain(|failing| *failing != stage);
        self.0.len() < before
    }
}
