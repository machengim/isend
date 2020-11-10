use anyhow::{anyhow, Result};
use async_std::fs::File;
use std::path::PathBuf;

// Used to record the current transmitting file.
#[derive(Debug, Default)]
pub struct CurrentFile {
    pub fd: Option<File>,     // used only in receiver as file descriptor.
    pub path: PathBuf,
    pub name: String,       // path may vary on different side.
    pub size: u64,          // name and size are meta info to send and receive.
    pub transmitted: u64,   // the size that has been transmitted.
}

impl CurrentFile {
    // used in sender side. Init an object with pathbuf.
    pub fn from(path: &PathBuf) -> Result<Self> {
        let name = match read_file_name(path) {
            Some(f) => f,
            None => return Err(anyhow!("Cannot read file name")),
        };

        let size = std::fs::metadata(path)?.len();
        let current = CurrentFile {
            path: path.clone(),
            name,
            size,
            ..Default::default()
        };
        log::debug!("Init current file in sender: {:?}", current);

        Ok(current)
    }

    // retrieve the fd of the current file object,
    // return error if not existed.
    pub fn must_get_fd(&self) -> Result<&File> {
        match &self.fd {
            Some(fd) => Ok(fd),
            None => Err(anyhow!("No file descriptor found")),
        }
    }

    pub fn meta_to_string(&self) -> String {
        format!("size:{};name:{}", self.size, self.name)
    }

    // Set the name and size field according to the meta string. used in receiver.
    pub fn meta_from_string(meta: &String) -> Result<(u64, String)> {
        let metas: Vec<&str> = meta.split(|c| c == ':' || c == ';').collect();
        if metas.len() != 4 {
            return Err(anyhow!("Invalid meta string format"));
        }

        let size = metas[1].parse()?;
        let name = String::from(metas[3]);

        Ok((size, name))
    }
}

// Helper function to read file name.
fn read_file_name(file: &PathBuf) -> Option<String> {
    let filename = file.file_name()?.to_str()?;

    Some(String::from(filename))
}

#[cfg(test)]
mod test {
    use super::*;

    #[test]
    fn meta_read_get_test() {
        let mut f = CurrentFile{
            name: String::from("Hello"),
            size: 2954040,
            ..Default::default()
        };

        if let Ok((size, name)) = CurrentFile::meta_from_string(&f.meta_to_string()) {
            f.name = name;
            f.size = size;
        }

        assert_eq!(f.name, String::from("Hello"));
        assert_eq!(f.size, 2954040);
    }
}