use anyhow::Result;
use num_enum::TryFromPrimitive;
use std::convert::TryFrom;

#[derive(Clone, Copy, Debug, Eq, PartialEq, TryFromPrimitive)]
#[repr(u8)]
pub enum Operation {
    // Request operation code.
    Connect = 1,    // with or without password, needs reply
    StartSendFile = 10,     // with file name, needs reply
    SendFileSize = 11,      // with file size, needs reply
    SendFileContent = 12,   // with file content
    EndSendFile = 13,       // needs reply
    StartSendDir = 20,      // with dir name
    EndSendDir = 21,        // needs reply
    SendMsg = 30,           // with message length
    SendMsgContent = 31,    // with message content, needs reply

    // Mutual operation code.
    // Put it in between for the convinience of parsing.
    EndConn = 100,          // needs reply

    // Response operation code.
    ConnSuccess = 101,
    ConnRefuse = 102,           // with reply content, no need to retry
    ConnError = 103,            // with reply content, needs to retry
    RequestSuccess = 110,   
    RequestRefuse = 111,        // with reply content, no need to retry
    RequestError = 112,         // with reply content, needs to retry
}

impl Default for Operation {
    fn default() -> Self {
        Operation::Connect
    }
}

#[derive(Clone, Copy, Debug, Default)]
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