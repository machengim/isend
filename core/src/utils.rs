use rand::Rng;

// Covert a decimal to a hex string.
pub fn dec_to_hex(num: u16, length: usize) -> String {
    let mut hex_str = format!("{:x}", num);
    while hex_str.len() < length {
        hex_str = format!("0{}", hex_str);
    }

    hex_str
}

// Covert a hex string to a decimal, used to translate the port number.
pub fn hex_to_dec(s: &str) -> u16 {
    let num = u16::from_str_radix(s, 16)
        .expect("Cannot parse port string");
    num
}

pub fn rand_range(min: u8, max: u8) -> u8 {
    rand::thread_rng().gen_range(min, max)
}

// Valid hex string range: 0 ~ f. Only lower case allowed.
pub fn validate_hex_str(s: &str) -> bool {
    for c in s.as_bytes().iter(){
        if !(c >= &48 && c <= &57) && !(c >= &97 && c <= &102) {
            return false;
        }
    }

    true
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn port_to_hex_test() {
        assert_eq!(dec_to_hex(2000, 4), String::from("07d0"));
    }

    #[test]
    fn validate_hex_str_test() {
        assert!(validate_hex_str("ae90ff02"));
        assert!(!validate_hex_str("s-1"));
    }

    #[test]
    fn hex_to_decimal_test() {
        assert_eq!(hex_to_dec("f209"), 61961);
        assert_ne!(hex_to_dec("f209"), 61960);
    }

    #[test]
    fn num_to_byte_test() {
        let value = u16::from_be_bytes([0x2a, 0x34]);
        assert_eq!(value, 0x2a34);
    }
}