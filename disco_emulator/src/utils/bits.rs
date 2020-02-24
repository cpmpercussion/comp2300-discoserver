use crate::{Shift, ShiftType};

pub fn bitset<T: Into<u32>>(word: T, bit: T) -> bool {
    let word = word.into();
    let bit = bit.into();
    return (word & (1 << bit)) > 0;
}

pub fn matches<T: Into<u32>>(word: T, shift: T, mask: T, expected: T) -> bool {
    let word = word.into();
    let shift = shift.into();
    let mask = mask.into();
    let expected = expected.into();
    return ((word >> shift) & mask) == expected;
}

pub fn decode_imm_shift<T: Into<u32>>(encoded: T) -> Shift {
    // A7.4.2
    let encoded = encoded.into();
    let shift_n = (encoded & 0x1F) as u32;
    return match encoded >> 5 {
        0b00 => Shift {shift_t: ShiftType::LSL, shift_n},
        0b01 => Shift {shift_t: ShiftType::LSR, shift_n},
        0b10 => Shift {shift_t: ShiftType::ASR, shift_n},
        0b11 if shift_n == 0 => Shift {shift_t: ShiftType::RRX, shift_n: 1},
        0b11 if shift_n != 0 => Shift {shift_t: ShiftType::ROR, shift_n},
        _ => panic!(),
    }
}

pub fn align(address: u32, size: u32) -> u32 {
    assert!(size == 1 || size == 2 || size == 4);
    return address & !(size - 1);
}

pub fn word_align(address: u32) -> u32 {
    return align(address, 4);
}

pub fn sign_extend(num: u32, bits: u32) -> u32 {
    return shifted_sign_extend(num, bits, 0);
}

/**
 * Sign extends the number value[0:bits], and shifts it left by shift.
 * Guarantees that only the _bits_ lowest bits are maintained, and higher
 * bits are cleared (pre shift). Therefore, it is safe to use directly in
 * an encoding such as foo[8]-offset[8]
 */
pub fn shifted_sign_extend(value: u32, bits: u32, shift: u32) -> u32 {
    assert!(bits < 32 && shift < 32);
    return (((value << (31 - bits)) as i32) >> (31 - bits - shift)) as u32;
}

pub fn ror_c(input: u32, shift: u32) -> (u32, bool) {
    // p27
    assert!(shift != 0);
    let result = input.rotate_right(shift % 32);
    return (result, bitset(result, 31));
}

pub fn rrx_c(input: u32, carry_in: u32) -> (u32, bool) {
    // p27
    let result = (input >> 1) + (carry_in << 31);
    let carry_out = bitset(result, 0);
    return (result, carry_out);
}

pub fn rrx(input: u32, carry_in: u32) -> u32 {
    // p27
    return rrx_c(input, carry_in).0;
}

pub fn lsl_c(input: u32, shift: u32) -> (u32, bool) {
    // p26
    let result = input.checked_shl(shift).unwrap_or(0);
    let carry_out = bitset(input, 32 - shift);
    return (result, carry_out);
}

pub fn lsr_c(input: u32, shift: u32) -> (u32, bool) {
    // p26
    let result = input.checked_shr(shift).unwrap_or(0);
    let carry_out = bitset(input, shift - 1);
    return (result, carry_out);
}

pub fn asr_c(input: u32, mut shift: u32) -> (u32, bool) {
    // p27
    if shift >= 32 { shift = 31; } // safe, because 32 shift == 31 shift with ASR
    let result = ((input as i32) >> shift) as u32;
    let carry_out = bitset(input, shift - 1);
    return (result, carry_out);
}

pub fn shift_c(input: u32, shift_t: u32, shift_n: u32, carry_in: u32) -> (u32, bool) {
    // A7.4.2
    if shift_n == 0 {
        return (input, carry_in == 1);
    }
    return match shift_t {
        0b00 => lsl_c(input, shift_n),
        0b01 => lsr_c(input, shift_n),
        0b10 => asr_c(input, shift_n),
        0b11 if shift_n == 0 => rrx_c(input, carry_in),
        0b11 if shift_n != 0 => ror_c(input, shift_n),
        _ => unreachable!(),
    }
}

pub fn shift(input: u32, shift_t: u32, shift_n: u32, carry_in: u32) -> u32 {
    // p181
    return shift_c(input, shift_t, shift_n, carry_in).0;
}

pub fn add_with_carry(x: u32, y: u32, carry_in: u32) -> (u32, bool, bool) {
    // p28
    let unsigned_sum = (x as u64) + (y as u64) + (carry_in as u64);
    let result = unsigned_sum & 0xFFFF_FFFF;
    let carry_out = result != unsigned_sum;

    let x_neg = bitset(x, 31);
    let y_neg = bitset(y, 31);
    let result_neg = bitset(result as u32, 31);
    let overflow = (x_neg == y_neg) && (x_neg != result_neg);

    return (result as u32, carry_out, overflow);
}

pub fn extract_value(raw: u32, start: u32, size: u32) -> u32 {
    return (raw >> start) & (!0 >> (32 - size));
}

pub fn is_wide_thumb(word: u32) -> bool {
    return ((word >> 29) == 0b111) && ((word >> 27) != 0b11100);
}

fn get_mask(msbit: u32, lsbit: u32) -> u32 {
    assert!(msbit < 32 && lsbit < 32);
    let mut mask = 0u32;
    // 32 size shift will panic (and probably doesn't do what you'd expect on the x86 CPU)
    if msbit < 31 {
        mask |= 0xFFFF_FFFFu32 << (msbit + 1);
    }
    if lsbit > 0 {
        mask |= 0xFFFF_FFFFu32 >> (32 - lsbit);
    }
    return mask;
}

pub fn bit_field_clear(val: u32, msbit: u32, lsbit: u32) -> u32 {
    return val & get_mask(msbit, lsbit);
}

pub fn bit_field_insert(original: u32, provider: u32, msbit: u32, lsbit: u32) -> u32 {
    let mask = get_mask(msbit, lsbit);
    return (original & mask) | (provider | !mask);
}

pub fn split_u64(large: u64) -> (u32, u32) {
    let upper = (large >> 32) as u32;
    let lower = (large & 0xFFFF_FFFF) as u32;
    return (upper, lower);
}
