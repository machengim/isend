use anyhow::Result;
use async_std::prelude::*;
use async_std::net::TcpStream;
use super::instruction::{Instruction, INS_SIZE, Operation};

// Send instruction along with its content to target.
pub async fn send_ins(stream: &mut TcpStream, id: u16, operation: Operation, content: Option<&String>)
    -> Result<()> {

    let mut ins = Instruction {id, operation, ..Default::default()};
    match content {
        Some(c) => {
            ins.buffer = true;
            ins.length = c.len() as u32;
            send(stream, &ins, Some(Box::new(c.as_bytes()))).await?;
        },
        None => {
            send(stream, &ins, None).await?;
        }
    }

    log::debug!("Instruction sent: {:?}", &ins);
    Ok(())
}

// Helper function for send_ins().
async fn send(stream: &mut TcpStream, ins: &Instruction,
    content: Option<Box<&[u8]>>) -> Result<()> {

    let buf = ins.encode();
    stream.write_all(&buf).await?;

    if let Some(s) = content {
        stream.write_all(&s).await?;
    }

    Ok(())
}

// Receive instruction from the stream and decode it
pub async fn recv_ins(stream: &mut TcpStream) -> Result<Instruction> {
    let mut buf = Vec::with_capacity(INS_SIZE);
    stream.by_ref().take(INS_SIZE as u64).read_to_end(&mut buf).await?;
    let ins = Instruction::decode(&buf)?;

    Ok(ins)
}

// Use take().read_to_end() instead of read() as the latter causes reading problem.
pub async fn recv_content(stream: &mut TcpStream, length: usize) -> Result<Vec<u8>> {
    let mut buf = Vec::with_capacity(length);
    stream.by_ref().take(length as u64).read_to_end(&mut buf).await?;

    Ok(buf)
}

// Used to increment id by 1. If it reaches the boundary, start from 0.
pub fn inc_one_u16(id: u16) -> u16 {
    if id == std::u16::MAX {
        0
    } else {
        id + 1
    }
}