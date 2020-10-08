use std::env;
use std::str::FromStr;

pub fn port_to_hex(port: u16) -> String {
    let mut hex = format!("{:x}", port);
    while hex.len() < 4 {
        hex = format!("0{}", hex);
    }

    hex
}

pub fn read_env<T: FromStr>(key: &str, default: &str) -> T {
    match env::var(key).unwrap_or(String::from(default)).parse() {
        Ok(s) => return s,
        Err(_) => {
            eprintln!("Parse env error");
            std::process::exit(1);
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn port_to_hex_test() {
        assert_eq!(port_to_hex(2000), String::from("07d0"));
    }
}