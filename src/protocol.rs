use num_enum::TryFromPrimitive;
use std::convert::TryFrom;

#[derive(Copy, Clone, Debug, PartialEq, TryFromPrimitive)]
#[repr(u8)]
pub enum Code {
    // Request operation code.
    ConnWithoutPass = 0,
    ConnWithPass = 1,
    PreSendFile = 10,
    PreSendContent = 11,
    EndSendFile = 12,
    PreSendDir = 20,
    EndSendDir = 21,
    PreSendMsg = 30,

    // Mutual operation code.
    // Put it in between for the convinience of parsing.
    EndConn = 100,

    // Response operation code.
    ConnSuccess = 101,
    ConnRefuse = 102,
    ConnFail = 103,
    RequestSuccess = 110,
    RequestRefuse = 111,
    RequestFail = 112,
    AbortFile = 120,
    AbortDir = 121,
}

impl Default for Code {
    fn default() -> Self {
        Code::ConnWithoutPass
    }
}

#[derive(Debug, Default, PartialEq)]
pub struct Instruction {
    pub id: u16,
    pub code: Code,
    pub buffer: bool,
    pub length: u16,
}

impl Instruction {
    // The very first request from the sender.
    // Check whether a password is provided.
    pub fn init(s: &Option<String>) -> Instruction {
        match s {
            Some(pw) => init_with_password(&pw),
            None => Instruction::default(),
        }
    }

    pub fn encode(&self) -> [u8; 6] {
        let mut buf = [0; 6];
        let mut i = 0;

        for e in self.id.to_be_bytes().iter() {
            buf[i] = *e;
            i += 1;
        }

        buf[i] = (self.code as u8).to_be_bytes()[0];
        i += 1;

        buf[i] = if self.buffer {1} else {0};
        i += 1;

        for e in self.length.to_be_bytes().iter() {
            buf[i] = *e;
            i += 1;
        }

        buf
    }

    pub fn decode(buf: &[u8; 6]) -> Instruction {
        let id = u16::from_be_bytes([buf[0], buf[1]]);

        let code_num = u8::from_be_bytes([buf[2]]);
        let code = Code::try_from(code_num)
            .expect("Cannot parse code");

        let buffer_num = u8::from_be_bytes([buf[3]]);
        let buffer = if buffer_num == 1 {true} else {false};

        let length = u16::from_be_bytes([buf[4], buf[5]]);

        Instruction{
            id, code, buffer, length,
        }
    }
}

fn init_with_password(pw: &str) -> Instruction {
    Instruction {
        id: 0,
        code: Code::ConnWithPass,
        buffer: true,
        length: pw.len() as u16,
    }
}

#[cfg(test)]
mod test{
    use super::*;

    #[test]
    fn instruction_encode_test() {
        let ins = Instruction::init(&None);
        assert_eq!(ins.encode(), [0, 0, 0, 0, 0, 0]);
    }

    #[test]
    fn instruction_decode_test() {
        let ins = Instruction{
            id: 2,
            code: Code::EndConn,
            buffer: true,
            length: 6
        };
        
        assert_eq!(ins, Instruction::decode(&[0, 2, 100, 1, 0, 6]));
        assert_ne!(ins, Instruction::decode(&[0, 3, 100, 1, 0, 6]));
    }
}