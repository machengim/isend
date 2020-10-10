pub enum ArgType {
    Send,
    Recv,
}

pub trait Arg {
    fn get_arg_type(&self) -> ArgType;
}

#[derive(Default)]
pub struct SendArg {
    pub expire: u16,
    pub port: u16,
    pub msg: Option<String>,
    pub files: Option<Vec<String>>,
    pub shutdown: bool
}

#[derive(Default)]
pub struct RecvArg {
    pub expire: u16,
    pub port: u16,
    pub dir: Option<String>,
    pub code: Option<String>,
    pub shutdown: bool,
    pub overwrite: bool,
}

impl Arg for SendArg {
    fn get_arg_type(&self) -> ArgType { ArgType::Send }
}

impl Arg for RecvArg {
    fn get_arg_type(&self) -> ArgType { ArgType::Recv }
}

impl SendArg {
    fn new() -> Self {
        SendArg{expire: 10, ..Default::default()}
    }
}

/*
macro_rules! impl_arg_new {
    (for $($t:ty),+) => {
        $(impl $t {
            fn new() -> Self { $t{expire:10, ..Default::default() }}
        })*
    }
}

impl_arg_new!(for SendArg, RecvArg);
*/

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn arg_new_test() {
        assert_eq!(SendArg::new().expire, 10);
    }
}

/*
pub trait Arg {
    fn get_expire(&self) -> u16;
    fn get_port(&self) -> u16;
    fn get_shutdown(&self) -> bool;
}

macro_rules! impl_Arg {
    (for $($t:ty),+) => {
        $(impl Arg for $t {
            fn get_expire(&self) -> u16 {self.expire }
            fn get_port(&self) -> u16 { self.port }
            fn get_shutdown(&self) -> bool { self.shutdown }
        })*
    }
}

impl_Arg!(for SendArg, RecvArg);
*/