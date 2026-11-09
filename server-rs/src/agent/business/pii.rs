//! @docs ARCHITECTURE:Core
//! ### AI Assist Note
//! Detects US SSN and Luhn-valid payment card numbers to prevent PII leakage in mock SMB environments.
//! ### 🔍 Debugging & Observability
//! Traces identify forbidden PII violations without leaking raw PII.

fn luhn_digit(idx: usize, d: u8) -> u32 {
    let mut val = u32::from(d);
    if idx % 2 == 1 {
        val *= 2;
        if val > 9 {
            val -= 9;
        }
    }
    val
}

pub fn luhn_valid(digits: &[u8]) -> bool {
    if !(13..=19).contains(&digits.len()) || digits.iter().all(|&d| d == 0) {
        return false;
    }
    let sum: u32 = digits
        .iter()
        .rev()
        .enumerate()
        .map(|(idx, &d)| luhn_digit(idx, d))
        .sum();
    sum.is_multiple_of(10)
}

fn is_valid_ssn_match(bytes: &[u8], i: usize) -> bool {
    let prev_ok = i == 0 || !bytes[i - 1].is_ascii_alphanumeric();
    let next_ok = i + 11 == bytes.len() || !bytes[i + 11].is_ascii_alphanumeric();
    if !prev_ok || !next_ok {
        return false;
    }
    let sep1 = bytes[i + 3];
    let sep2 = bytes[i + 6];
    let valid_sep = (sep1 == b'-' && sep2 == b'-') || (sep1 == b' ' && sep2 == b' ');
    if !valid_sep {
        return false;
    }
    let digit_indices = [0, 1, 2, 4, 5, 7, 8, 9, 10];
    digit_indices
        .iter()
        .all(|&idx| bytes[i + idx].is_ascii_digit())
}

fn detect_ssn(bytes: &[u8]) -> bool {
    if bytes.len() < 11 {
        return false;
    }
    for i in 0..=bytes.len() - 11 {
        if is_valid_ssn_match(bytes, i) {
            return true;
        }
    }
    false
}

fn extract_card_digits(bytes: &[u8], start: usize) -> (Vec<u8>, usize) {
    let mut digits = Vec::with_capacity(19);
    let mut j = start;
    while j < bytes.len() {
        if bytes[j].is_ascii_digit() {
            digits.push(bytes[j] - b'0');
            if digits.len() > 19 {
                break;
            }
        } else if (bytes[j] == b' ' || bytes[j] == b'-')
            && j + 1 < bytes.len()
            && bytes[j + 1].is_ascii_digit()
        {
            // allow single separator between digit groups
        } else {
            break;
        }
        j += 1;
    }
    (digits, j)
}

fn has_valid_card_prefix(digits: &[u8]) -> bool {
    (13..=19).any(|len| len <= digits.len() && luhn_valid(&digits[..len]))
}

fn detect_payment_card(bytes: &[u8]) -> bool {
    let mut i = 0;
    while i < bytes.len() {
        let is_digit_start =
            bytes[i].is_ascii_digit() && (i == 0 || !bytes[i - 1].is_ascii_digit());
        if !is_digit_start {
            i += 1;
            continue;
        }
        let (digits, j) = extract_card_digits(bytes, i);
        if luhn_valid(&digits) || has_valid_card_prefix(&digits) {
            return true;
        }
        i = if j > i { j } else { i + 1 };
    }
    false
}

pub fn detect_sensitive_pii(text: &str) -> Option<&'static str> {
    let bytes = text.as_bytes();
    if detect_ssn(bytes) {
        return Some("SSN");
    }
    if detect_payment_card(bytes) {
        return Some("Payment Card");
    }
    None
}
