pub fn port_to_hex(port: u16) -> String {
    let mut hex = format!("{:x}", port);
    while hex.len() < 4 {
        hex = format!("0{}", hex);
    }

    hex
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn port_to_hex_test() {
        assert_eq!(port_to_hex(2000), String::from("07d0"));
    }
}