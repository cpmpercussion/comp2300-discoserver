use std::vec::Vec;
use std::str;

pub fn hex_to_word(hex: &[u8]) -> Result<u32, ()> {
    let s = match str::from_utf8(hex) {
        Ok(s) => s,
        Err(_) => return Err(()),
    };

    match u32::from_str_radix(s, 16) {
        Ok(u) => Ok(u),
        Err(_) => Err(()),
    }
}

pub fn word_to_hex(word: u32) -> String {
    return format!("{:08x}", word);
}

pub fn validate_packet(data: &[u8], check: u8) -> bool {
    let mut sum: u8 = 0;
    for &i in data {
        sum = sum.wrapping_add(i);
    }
    return sum == check;
}

pub fn is_hex_char(c: u8) -> bool {
    return match c {
        b'0'..=b'9' => true,
        b'a'..=b'f' => true,
        b'A'..=b'F' => true,
        _ => false,
    };
}

// Takes a ASCII hex number [0-9a-fA-F] and returns the value as a u8
pub fn hex_to_byte(c: u8) -> Result<u8, ()> {
    return match c {
        b'0'..=b'9' => Ok(c - b'0'),
        b'a'..=b'f' => Ok(c - b'a' + 10),
        b'A'..=b'F' => Ok(c - b'A' + 10),
        _ => Err(()),
    };
}

pub fn get_u8_from_hex(hex: (u8, u8)) -> Result<u8, ()> {
    if let (Ok(b1), Ok(b2)) = (hex_to_byte(hex.0), hex_to_byte(hex.1)) {
        return Ok(b1 * 16 + b2);
    } else {
        return Err(());
    }
}

pub fn leading_alpha(data: &[u8]) -> &[u8] {
    for i in 0..data.len() {
        match data[i] {
            b'a'..=b'z' | b'A'..=b'Z' => {},
            _ => {
                return &data[0..i];
            }
        }
    }
    return data;
}

pub fn get_checksum_hex(packet: &[u8]) -> String {
    let mut sum: u8 = 0;
    for &b in packet {
        sum = sum.wrapping_add(b);
    };
    return format!("{:02X?}", sum);
}
