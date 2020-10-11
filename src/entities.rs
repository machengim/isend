#[derive(Debug)]
pub enum Argument {
    S(SendArg),
    R(RecvArg),
}

pub enum ArgType {
    Send,
    Recv,
}

pub trait Arg {
    fn get_arg_type(&self) -> ArgType;
}

#[derive(Debug, Default)]
pub struct SendArg {
    pub expire: u16,
    pub port: u16,
    pub msg: Option<String>,
    pub files: Option<Vec<String>>,
    pub shutdown: bool
}

#[derive(Debug, Default)]
pub struct RecvArg {
    pub retry: u8,
    pub port: u16,
    pub dir: Option<String>,
    pub code: Option<String>,
    pub shutdown: bool,
    pub overwrite: bool,
}

macro_rules! impl_arg_type {
    (for $($t:ty),+) => {
        $(impl Arg for $t {
            fn get_arg_type(&self) -> ArgType {
                match stringify!($t) {
                    "SendArg" => ArgType::Send,
                    "RecvArg" => ArgType::Recv,
                    _ => panic!("Cannot parse argument type"),
                }
            }
        })*
    }
}

impl_arg_type!(for SendArg, RecvArg);

impl SendArg {
    pub fn new() -> Self { SendArg{expire: 10, ..Default::default() } }
}

impl RecvArg {
    pub fn new() -> Self { RecvArg{retry: 5, ..Default::default() } }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn arg_new_test() {
        assert_eq!(SendArg::new().expire, 10);
        assert!(!RecvArg::new().shutdown);
    }

    #[test]
    fn arg_type_test() {
        assert!(matches!(SendArg::new().get_arg_type(), ArgType::Send));
        assert!(matches!(RecvArg::new().get_arg_type(), ArgType::Recv));
    }
}
