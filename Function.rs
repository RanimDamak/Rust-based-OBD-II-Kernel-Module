fn ascii_to_decimal(ascii_code: u8) -> Result<u8, &'static str> {
    if ascii_code >= b'0' && ascii_code <= b'9' {
        Ok(ascii_code - b'0')
    } else {
        Err("Invalid ASCII code for a decimal digit")
    }
}

fn decimal_to_hex(decimal: u8) -> Result<[u8; 2], &'static str> {
    if decimal > 255 {
        return Err("Input out of range, should be between 0 and 255");
    }

    const HEX_DIGITS: &[u8; 16] = b"0123456789ABCDEF";
    let high_nibble = (decimal >> 4) & 0xF;
    let low_nibble = decimal & 0xF;

    Ok([HEX_DIGITS[high_nibble as usize], HEX_DIGITS[low_nibble as usize]])
}

fn main() {
    let decimal_value = 59; // Example decimal value

    match decimal_to_hex(decimal_value) {
        Ok(hex_digits) => {
            // Replace these lines with your kernel's print mechanism
            // Example: print the hexadecimal digits
            // print!("Hex value: {}{}", hex_digits[0] as char, hex_digits[1] as char); // Adjust according to your environment
        },
        Err(e) => {
            // Handle the error, e.g., print an error message
            // print!("Error: {}", e); // Adjust according to your environment
        },
    }
}
