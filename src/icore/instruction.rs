use anyhow::Result;
use num_enum::TryFromPrimitive;
use std::convert::TryFrom;
use super::message::{Message, send_msg};

pub const INS_SIZE: usize = 8;

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

#[derive(Clone, Copy, Debug, Default, PartialEq)]
pub struct Instruction {
    pub id: u16,
    pub operation: Operation,
    pub buffer: bool,
    pub length: u32,    // max 16M for one frame
}

impl Instruction {
    // Encode an instruction to an array of u8
    pub fn encode(&self) -> [u8; INS_SIZE] {
        let mut buf = [0u8; INS_SIZE];
        // Position 1~2 bytes are instruction id, a u16.
        buf[0] = u16::to_be_bytes(self.id)[0];
        buf[1] = u16::to_be_bytes(self.id)[1];
        // Position 3 is the operation code.
        buf[2] = (self.operation as u8).to_be_bytes()[0];
        // Position 4 is a boolean representing with buffer or not
        buf[3] = if self.buffer {1} else {0};
        // Position 5~8 is the length of the following content
        let len_bytes = u32::to_be_bytes(self.length);
        let mut i = 4;
        for byte in len_bytes.iter() {
            buf[i] = *byte;
            i += 1;
        }

        buf
    }

    // Decode a vector of u8 to an instruction
    pub fn decode(buf: &Vec<u8>) -> Result<Self> {
        if buf.len() != INS_SIZE {
            send_msg(Message::Fatal(format!("Unknown instruction format")));
        }

        let id = u16::from_be_bytes([buf[0], buf[1]]);

        let operation_num = u8::from_be_bytes([buf[2]]);
        let operation = Operation::try_from(operation_num)?;

        let buffer_num = u8::from_be_bytes([buf[3]]);
        let buffer = if buffer_num == 1 {true} else {false};

        let length = u32::from_be_bytes([buf[4], buf[5], buf[6], buf[7]]);

        Ok(Instruction{
            id, operation, buffer, length,
        })
    }
}

#[cfg(test)]
mod test {
    use super::*;

    #[test]
    fn encode_ins_test() {
        let ins = Instruction {id: 5, operation: Operation::Connect, buffer: true, length: 43375};
        let mut arr: [u8; 8] = [0, 5, 1, 1, 0, 0, 169, 111];
        assert_eq!(ins.encode(), arr);
        arr[7] += 1;
        assert_ne!(ins.encode(), arr);
    }

    #[test]
    fn decode_ins_test() {
        let ins = Instruction {id: 5, operation: Operation::Connect, buffer: true, length: 43375};
        let mut vec = vec![0, 5, 1, 1, 0, 0, 169, 111];
        assert_eq!(Instruction::decode(&vec).unwrap(), ins);
        vec[0] += 1;
        assert_ne!(Instruction::decode(&vec).unwrap(), ins);
    }

    #[test]
    #[should_panic]
    fn decode_ins_bad_format(){
        let vec = vec![0, 5, 1, 1, 0, 169, 111];
        let _ = Instruction::decode(&vec);
    }
}