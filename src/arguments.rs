use clap::{ArgMatches, Values};
use std::process::exit;
use crate::utils::{FileType, check_file_type, validate_hex_str};

#[derive(Debug)]
pub enum Argument {
    S(SendArg),
    R(RecvArg),
}

#[derive(Debug)]
pub enum OverwriteStrategy{
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
    pub expire: u16,
    pub files: Option<Vec<String>>,
    pub msg: Option<String>,
    pub password: Option<String>,
    pub port: u16,
    pub shutdown: bool
}

#[derive(Debug, Default)]
pub struct RecvArg {
    pub code: String,
    pub dir: Option<String>,
    pub overwrite: OverwriteStrategy,
    pub password: Option<String>,
    pub port: u16,
    pub retry: u8,
    pub shutdown: bool,
}

pub trait ArgTrait {
    fn from_match(m: &ArgMatches) -> Self;
    fn parse_password(&mut self, m: &ArgMatches);
    fn parse_port(&mut self, m: &ArgMatches);
    fn parse_shutdown(&mut self, m: &ArgMatches);
}

macro_rules! impl_arg_trait {
    (for $($t:ty),+) => {
        $(impl $t {
            pub fn from_match(m: &ArgMatches) -> Self {
                type T = $t; 
                let mut arg: T = T{..Default::default()};

                match stringify!($t) {
                    "SendArg" => arg.parse_sender(m),
                    "RecvArg" => arg.parse_receiver(m),
                    _ => panic!("Cannot parse argument type"),
                }

                arg.parse_password(m)
                    .parse_port(m)
                    .parse_shutdown(m);

                arg
            }

            fn parse_password(&mut self, m: &ArgMatches) -> &mut Self {
                if let Some(pw) = m.value_of("password") {
                    if pw.len() > 255 {
                        eprintln!("The password is too long");
                        exit(1);
                    }

                    self.password = Some(String::from(pw));
                }

                self
            }

            fn parse_port(&mut self, m: &ArgMatches) -> &mut Self {
                if let Some(port) = m.value_of("port") {
                    self.port = port.parse()
                        .expect("Cannot parse port number");
                }

                self
            }

            fn parse_shutdown(&mut self, m: &ArgMatches) -> &mut Self {
                if m.occurrences_of("shutdown") > 0 {
                    self.shutdown = true;
                }

                self
            }
        })*
    } 
}

impl_arg_trait!(for SendArg, RecvArg);

impl SendArg {
    fn parse_receiver(&mut self, _: &ArgMatches) { /* dumb function for macro */ }

    fn parse_sender(&mut self, m: &ArgMatches) { 
        self.parse_expire(m)
            .parse_msg(m)
            .parse_sending_files(m)
            .validate_sender();

        println!("{:?}", self);
    }

    fn parse_expire(&mut self, m: &ArgMatches) -> &mut Self {
        match m.value_of("expire") {
            Some(expire) => self.expire = expire.parse()
                .expect("Cannot parse expire minutes"),
            None => self.expire = 10,
        }

        self
    }

    // Return &mut Self to construct chain operations.
    fn parse_msg(&mut self, m: &ArgMatches) -> &mut Self {
        if let Some(msg) = m.value_of("message") {
            self.msg = Some(String::from(msg));
        }

        self
    }

    fn parse_sending_files(&mut self, m: &ArgMatches) -> &mut Self {
        if let Some(mut inputs) = m.values_of("INPUT") {
            self.files = parse_files(&mut inputs);
        }

        self
    }

    // Check if both `files` and `msg` fields are empty.
    fn validate_sender(&self) {
        if self.files.is_none() && self.msg.is_none() {
            eprintln!("No files or message specified");
            exit(1);
        }
    }
}

impl RecvArg {
    fn parse_sender(&mut self, _: &ArgMatches) { /* dumb function for macro */ }

    fn parse_receiver(&mut self, m: &ArgMatches) { 
        self.parse_code(m)
            .parse_dir(m)
            .parse_overwrite(m)
            .parse_retry(m)
            .validate_receiver();

        println!("{:?}", self);
     }

    // To receiver, code is compulsory.
    fn parse_code(&mut self, m: &ArgMatches) -> &mut Self {
        let mut inputs = match m.values_of("INPUT") {
            Some(ins) => if ins.len() > 1 {
                eprintln!("Only one code allowed");
                exit(1);
            } else { ins },

            None => {
                eprintln!("No code provided");
                exit(1);
            }
        };

        self.code = match inputs.next() {
            Some(c) => String::from(c),
            None => {
                eprintln!("No code found"); 
                exit(1)
            },
        };

        self
    }

    fn parse_dir(&mut self, m: &ArgMatches) -> &mut Self {
        if let Some(dir) = m.value_of("dir") {
            if !check_file_type(dir, FileType::Dir) {
                eprintln!("Invalid directory path");
                exit(1);
            }

            self.dir = Some(String::from(dir));
        }

        self
    }

    // Default overwrite option is `None` which means asking every time?
    fn parse_overwrite(&mut self, m: &ArgMatches) -> &mut Self {
        if let Some(s) = m.value_of("overwrite") {
            match s {
                "a" | "A" => self.overwrite = OverwriteStrategy::Ask,
                "o" | "O" => self.overwrite = OverwriteStrategy::Overwrite,
                "r" | "R" => self.overwrite = OverwriteStrategy::Rename,
                "s" | "S" => self.overwrite = OverwriteStrategy::Skip,
                _ => {
                    eprintln!("Unkown overwrite strategy input")
                }
            }
        }

        self
    }

    fn parse_retry(&mut self, m: &ArgMatches) -> &mut Self {
        match m.value_of("retry") {
            Some(retry) => self.retry = retry.parse()
                .expect("Cannot parse retry number"),
            None => self.retry = 10,
        }

        self
    }

    fn validate_receiver(&self) {
        if self.code.len() != 6 || !validate_hex_str(&self.code) {
            eprintln!("Invalid code format");
            exit(1);
        }
    }
}

// Helper function for parse_sending_files.
// May be required by other functions in the future.
fn parse_files(fs: &mut Values) -> Option<Vec<String>> {
    if fs.len() <= 0 {
        return None;
    }

    let mut files: Vec<String> = Vec::new();
    while let Some(f) = fs.next() {
        if !std::path::Path::new(f).exists() {
            eprintln!("Invalid path found");
            exit(1);
        }

        files.push(String::from(f));
    }

    Some(files)
}
