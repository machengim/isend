use std::io::{Error, ErrorKind, Result};
use std::net::{IpAddr, Ipv4Addr, SocketAddr, TcpListener, UdpSocket};
use crate::utils;


pub enum SocketType {
    UDP,
    TCP,
}

pub enum OpenSocket {
    Udp(UdpSocket),
    Tcp(TcpListener),
}

impl OpenSocket {
    pub fn get_socket_port(&self) -> std::io::Result<u16> {
        let port = match self {
            OpenSocket::Tcp(tcp) => tcp.local_addr()?.port(),
            OpenSocket::Udp(udp) => udp.local_addr()?.port()
        };
    
        Ok(port)
    }
}

pub fn bind_socket(socket: &SocketAddr, stype: SocketType)
    -> std::io::Result<OpenSocket> {
    
    let ip = socket.ip();
    let port = socket.port();

    let socket = match stype {
        SocketType::TCP => bind_tcp(ip, port)?,
        SocketType::UDP => bind_udp(ip, port)?,
    };

    Ok(socket)
}

fn bind_udp(ip: IpAddr, port: u16) -> std::io::Result<OpenSocket> {
    let udp_socket = UdpSocket::bind((ip, port))?;
    let socket = OpenSocket::Udp(udp_socket);

    Ok(socket)
}

fn bind_tcp(ip: IpAddr, port: u16) -> std::io::Result<OpenSocket> {
    let tcp_socket = TcpListener::bind((ip, port))?;
    let socket = OpenSocket::Tcp(tcp_socket);

    Ok(socket)
}

pub fn listen_udp(socket: &OpenSocket, pass: &str) -> Result<SocketAddr>{
    if let OpenSocket::Udp(udp_socket) = socket {
        let mut buf = [0; 6];
        while let Ok((_, addr)) = udp_socket.recv_from(&mut buf) {
            let buf_str = std::str::from_utf8(&buf)
                .expect("Cannot convert buf to string");

            if utils::validate_hex_str(buf_str) && utils::compare_buf_pass(&buf, pass) {
                let port = utils::hex_to_decimal(&buf_str[..4]);
                return Ok(SocketAddr::new(addr.ip(), port));
            }
        }
    }

    Err(Error::new(ErrorKind::Other, "Unknown UDP socket to listen on."))
}

pub fn get_socket_addr(ip_str: &str, port: u16) -> SocketAddr {
    let ip: Ipv4Addr = ip_str.parse()
        .expect(&format!("Cannot parse IP address from {}", ip_str));

    SocketAddr::new(IpAddr::V4(ip), port)
}