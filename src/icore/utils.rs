use anyhow::Result;
use async_std::prelude::*;
use async_std::net::TcpStream;
use super::instruction::{Instruction, INS_SIZE, Operation};

// Send instruction along with its content to target.
pub async fn send_ins(stream: &mut TcpStream, id: u16, 
    operation: Operation, content: Option<&String>) -> Result<()> {

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

    Ok(())
}

pub async fn send_ins_bytes(stream: &mut TcpStream, id: u16, 
    operation: Operation, content: &Vec<u8>) -> Result<()> {
    let ins = Instruction {id, operation, buffer: true, length: content.len() as u32};

    send(stream, &ins, Some(Box::new(&content))).await?;

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

    log::debug!("Instruction sent: {:?}", &ins);

    Ok(())
}

// Receive instruction from the stream and decode it
pub async fn recv_ins(stream: &mut TcpStream) -> Result<Instruction> {
    let mut buf = Vec::with_capacity(INS_SIZE);
    stream.by_ref().take(INS_SIZE as u64).read_to_end(&mut buf).await?;
    let ins = Instruction::decode(&buf)?;

    log::debug!("Receive instruction: {:?}", &ins);
    
    Ok(ins)
}

// Use take().read_to_end() instead of read() as the latter causes reading problem.
pub async fn recv_content(stream: &mut TcpStream, length: usize) -> Result<Vec<u8>> {
    let mut buf = Vec::with_capacity(length);
    stream.by_ref().take(length as u64).read_to_end(&mut buf).await?;

    Ok(buf)
}

