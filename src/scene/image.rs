use std::path::PathBuf;

#[derive(Debug, Clone, PartialEq)]
pub struct ImageObject {
    pub path: PathBuf,
}

impl ImageObject {
    pub fn new(path: impl Into<PathBuf>) -> Self {
        Self { path: path.into() }
    }
}
