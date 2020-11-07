use anyhow::{anyhow, Result};
use std::convert::TryFrom;

#[derive(Clone, Copy, Debug)]
pub enum Operation {
    // Request operation code.
    ConnWithoutPass = 1,
    ConnWithPass = 2,
    PreSendFile = 10,
    SendMeta = 11,
    SendContent = 12,
    EndContent = 13,
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

impl TryFrom<u8> for Operation {
    type Error = anyhow::Error;

    fn try_from(value: u8) -> Result<Self, Self::Error> {
        match value {
            1 => Ok(Operation::ConnWithoutPass),
            2 => Ok(Operation::ConnWithPass),
            10 => Ok(Operation::PreSendFile),
            11 => Ok(Operation::SendMeta),
            12 => Ok(Operation::SendContent),
            13 => Ok(Operation::EndContent),
            20 => Ok(Operation::PreSendDir),
            21 => Ok(Operation::EndSendDir),
            30 => Ok(Operation::PreSendMsg),
            100 => Ok(Operation::EndConn),
            101 => Ok(Operation::ConnSuccess),
            102 => Ok(Operation::ConnRefuse),
            103 => Ok(Operation::ConnFail),
            110 => Ok(Operation::RequestSuccess),
            111 => Ok(Operation::RequestRefuse),
            112 => Ok(Operation::RequestFail),
            120 => Ok(Operation::AbortFile),
            121 => Ok(Operation::AbortDir),

            _ => Err(anyhow!("Unknow operation code")),
        }
    }
}

#[derive(Clone, Copy, Debug)]
pub struct Instruction {
    pub id: u16,
    pub operation: Operation,
    pub buffer: bool,
    pub length: u16,
}

impl Instruction {
    pub fn encode(&self) -> [u8; 6] {
        let mut buf = [0; 6];
        let mut i = 0;

        for e in self.id.to_be_bytes().iter() {
            buf[i] = *e;
            i += 1;
        }

        buf[i] = (self.operation as u8).to_be_bytes()[0];
        i += 1;

        buf[i] = if self.buffer {1} else {0};
        i += 1;

        for e in self.length.to_be_bytes().iter() {
            buf[i] = *e;
            i += 1;
        }

        buf
    }

    pub fn decode(buf: &Vec<u8>) -> Result<Self> {
        let id = u16::from_be_bytes([buf[0], buf[1]]);

        let operation_num = u8::from_be_bytes([buf[2]]);
        let operation = Operation::try_from(operation_num)?;

        let buffer_num = u8::from_be_bytes([buf[3]]);
        let buffer = if buffer_num == 1 {true} else {false};

        let length = u16::from_be_bytes([buf[4], buf[5]]);

        Ok(Instruction{
            id, operation, buffer, length,
        })
    }
}

#[cfg(test)]
mod test {
    use super::*;

    #[test]
    fn encode_test() {
        let ins = Instruction{id: 5, operation: Operation::AbortDir, buffer: true, length: 12};
        let buf: [u8; 6] = [0, 5, 121, 1, 0, 12];
        assert_eq!(&ins.encode(), &buf);
    }
}