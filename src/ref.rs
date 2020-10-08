use std::net::UdpSocket;
use std::time::Duration;

fn main() -> std::io::Result<()> {
    use std::fs;

    let metadata = fs::metadata("Cargo.toml")?;

    println!("{:?}", metadata.permissions());
    let socket: UdpSocket = UdpSocket::bind("0.0.0.0:34345")?;
    socket.set_read_timeout(Some(Duration::new(5, 0)))?;
    socket.set_broadcast(true)?;
    //socket.connect("255.255.255.255:34345")?;

    println!("Broadcast: {:?}", socket.broadcast());

    match socket.send_to(&[1, 2], "192.168.0.255:34345"){
        Ok(n) => println!("{}", n),
        Err(e) => println!("{}", e)
    }
    println!("Awaiting response...");

    let mut buf = [0; 10];
    while let Ok((n, addr)) = socket.recv_from(&mut buf) {
        println!("{} bytes response from {:?}", n, addr);
    }

    Ok(())
}