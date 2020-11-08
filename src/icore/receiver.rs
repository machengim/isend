use anyhow::{anyhow, Result};
use async_std::net::{TcpListener, TcpStream, UdpSocket};
use log::{info, debug};
use std::sync::mpsc;
use super::arg::RecvArg;
use super::instruction::{Instruction, Operation};
use super::utils;

pub async fn launch(arg: RecvArg) -> Result<()> {
    info!("Start receiver function");
    let tcp_socket = TcpListener::bind(("0.0.0.0", 0)).await?;
    let tcp_port = tcp_socket.local_addr()?.port();
    debug!("Listen on TCP port {}", tcp_port);

    let (tx, rx) = mpsc::channel();
    let arg_code = arg.code.clone();
    async_std::task::spawn(async move {
        if let Err(e) = broadcast_udp(tcp_port, arg_code, rx).await {
            eprintln!("Error in UDP: {}", e);
            std::process::exit(1);
        }
    });

    let mut stream = listen_tcp_conn(&tcp_socket, arg.password.as_ref()).await?;

    Ok(())
}

// Send UDP broadcast 10 times unless signal received
// local_port: TCP port of local machine(receiver)
// target_port: UDP port of remote machine(sender) 
async fn broadcast_udp(local_port: u16, target_port: u16, rx: mpsc::Receiver<bool>) -> Result<bool> {
    let udp_socket = UdpSocket::bind("0.0.0.0:0").await?;
    udp_socket.set_broadcast(true)?;

    for _ in 0..10 {
        udp_socket.send_to(&u16::to_be_bytes(local_port), ("255.255.255.255", target_port)).await?;
        //debug!("UDP broadcast sent to port {}", target_port);
        async_std::task::sleep(std::time::Duration::from_secs(5)).await;

        // Check message in channel
        if let Ok(true) = rx.try_recv() {
            return Ok(true);
        }
    }

    Err(anyhow!("No connection established"))
}

// Wait for tcp connection on the tcp socket and validate it.
// TODO: terminate after time runs out
async fn listen_tcp_conn(socket: &TcpListener, password: Option<&String>) -> Result<TcpStream> {

    loop {
        let (mut stream, addr) = socket.accept().await?;
        info!("Receive connection request from {}", &addr);

        let ins = utils::recv_ins(&mut stream).await?;
        match ins.operation {
            Operation::Connect => {
                match valiate_tcp_conn(&mut stream, &ins, password).await {
                    Ok(true) => {
                        println!("Connection established");
                        // reply
                        utils::send_ins(&mut stream, 0, Operation::ConnSuccess, None).await?;
                        return Ok(stream);
                    },
                    Ok(false) => {
                        let reply = format!("Invalid password");
                        println!("{}", reply);
                        utils::send_ins(&mut stream, 0, Operation::ConnRefuse, Some(&reply)).await?;
                    }
                    Err(e) => {
                        let reply = format!("Get error when validating tcp connection: {}", e);
                        println!("{}", reply);
                        utils::send_ins(&mut stream, 0, Operation::ConnRefuse, Some(&reply)).await?;
                    }
                }
            },
            _ => { debug!("Unknown operation code when expecting connection request")},
        }
    }
}

// validate tcp connection, if both passwords are provided, check the password content.
async fn valiate_tcp_conn(stream: &mut TcpStream, ins: &Instruction, password: Option<&String>)
    -> Result<bool> {

    match (ins.buffer, password.is_some()) {
        (false, false) => Ok(true),
        (true, true) => {
            if compare_pass(stream, ins, password.unwrap()).await? {
                Ok(true)
            } else {
                Ok(false)
            }},
        _ => Ok(false),
    }
}

async fn compare_pass(stream: &mut TcpStream, ins: &Instruction, password: &String)
    -> Result<bool> {
    
    let buf = utils::recv_content(stream, ins.length as usize).await?;
    let req_pass = String::from_utf8(buf)?;
    debug!("Local password: {}, remote password: {}", password, req_pass);

    Ok(&req_pass == password)
}