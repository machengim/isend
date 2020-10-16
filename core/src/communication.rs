#[derive(Debug)]
pub enum Message {
    Done,
    Error(String),
    Progress(u8),
    Status(String),
}

