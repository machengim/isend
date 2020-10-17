use std::path::PathBuf;

pub enum Arg {
    S(SendArg),
    R(RecvArg),
}

#[derive(Clone, Copy, Debug)]
pub enum OverwriteStrategy {
    Ask,
    Rename,
    Overwrite,
    Skip,
}

impl Default for OverwriteStrategy {
    fn default() -> Self {
        OverwriteStrategy::Ask
    }
}

#[derive(Debug, Default)]
pub struct SendArg {
    pub expire: u8,
    pub files: Option<Vec<PathBuf>>,
    pub msg: Option<String>,
    pub password: Option<String>,
    pub port: u16,
    pub shutdown: bool,
}

#[derive(Debug, Default)]
pub struct RecvArg {
    pub code: String,
    pub expire: u8,
    pub dir: PathBuf,
    pub overwrite: OverwriteStrategy,
    pub password: Option<String>,
    pub port: u16,
    pub shutdown: bool,
}