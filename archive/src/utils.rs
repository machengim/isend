use std::env;
use std::str::FromStr;
use rand::Rng;


// Generate code from 0 to f with the specific length.
pub fn generate_rand_hex_code(length: usize) -> String {
    if length > 8 {
        eprintln!("The maximum password length is 8");
        std::process::exit(1);
    } else if length == 0 {
        return String::from("");
    }

    let max = u32::pow(16, length as u32);
    let pass = rand::thread_rng().gen_range(0, max);
    decimal_to_hex(pass, length)
}

// Convert the port number to hex string with specific length.
pub fn decimal_to_hex(num: u32, length: usize) -> String {
    let mut hex = format!("{:x}", num);
    while hex.len() < length {
        hex = format!("0{}", hex);
    }

    hex
}

// Covert a hex string to a decimal, used to translate the port number.
pub fn hex_to_decimal(s: &str) -> u16 {
    if s.len() != 4 || !validate_hex_str(s){
        eprintln!("Invalid hex string for port number.");
        std::process::exit(1);
    }

    let num = i64::from_str_radix(s, 16).expect("Cannot parse port string");
    num as u16
}

pub fn validate_hex_str(s: &str) -> bool {
    let bytes = s.as_bytes();

    for c in bytes.iter(){
        if !(c >= &48 && c <= &57) && !(c >= &97 && c <= &102) {
            return false;
        }
    }

    true
}

pub fn compare_buf_pass(buf: &[u8], pass: &str) -> bool {
    let buf_len = buf.len();
    let pass_len = pass.len();
    let pass_bytes = pass.as_bytes();
    for i in 1..3 {
        if buf[buf_len - i] != pass_bytes[pass_len - i] {
            return false;
        }
    }

    true
}

// Read environment variable by specif key and default value.
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
        assert_eq!(decimal_to_hex(2000, 4), String::from("07d0"));
    }

    #[test]
    fn validate_hex_str_test() {
        assert!(validate_hex_str("ae90ff02"));
        assert!(!validate_hex_str("s-1"));
    }

    #[test]
    fn hex_to_decimal_test() {
        assert_eq!(hex_to_decimal("f209"), 61961);
        assert_ne!(hex_to_decimal("f209"), 61960);
    }

    #[test]
    fn compare_buf_pass_test() {
        assert!(compare_buf_pass(&['0' as u8, '9' as u8], "09"));
        assert!(!compare_buf_pass(&['f' as u8, '9' as u8], "09"));
    }
}