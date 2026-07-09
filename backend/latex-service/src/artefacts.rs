use std::path::PathBuf;
use uuid::Uuid;
use std::fs;
pub struct ArtefactStore { pub root: PathBuf }
impl ArtefactStore {
    pub fn new(root: impl Into<PathBuf>) -> Self { Self { root: root.into() } }
    pub fn write(&self, request_id: Uuid, ext: &str, body: &[u8]) -> std::io::Result<PathBuf> {
        let dir = self.root.clone();
        fs::create_dir_all(&dir)?;
        let p = dir.join(format!("{request_id}.{ext}"));
        fs::write(&p, body)?;
        Ok(p)
    }
    pub fn read(&self, request_id: Uuid, ext: &str) -> std::io::Result<Vec<u8>> {
        fs::read(self.root.join(format!("{request_id}.{ext}")))
    }
}
