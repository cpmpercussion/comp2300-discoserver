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
pub fn shifted_sign_extend(value: u32, sign_bit: u32, shift: u32) -> u32 {
    assert!(sign_bit < 32 && shift < 32);
    return (((value << (31 - sign_bit)) as i32) >> (31 - sign_bit - shift)) as u32;
}

pub fn ror_c(input: u32, shift: u32) -> (u32, bool) {
    // p27
    assert!(shift != 0);
    let result = input.rotate_right(shift % 32);
    return (result, bitset(result, 31));
}

pub fn rrx_c(input: u32, carry_in: u32) -> (u32, bool) {
    // p27
    assert!(carry_in == 1 || carry_in == 0);
    let result = (input >> 1) + (carry_in << 31);
    let carry_out = bitset(input, 0);
    return (result, carry_out);
}

pub fn lsl_c(input: u32, shift: u32) -> (u32, bool) {
    // p26
    assert!(shift > 0);
    return match shift {
        1..=31 => {
            let result = input << shift;
            let carry_out = bitset(input, 32 - shift);
            (result, carry_out)
        }
        32 => (0, bitset(input, 0)),
        _ => (0, false),
    }
}

pub fn lsr_c(input: u32, shift: u32) -> (u32, bool) {
    // p26
    assert!(shift > 0);
    return match shift {
        1..=31 => {
            let result = input >> shift;
            let carry_out = bitset(input, shift - 1);
            (result, carry_out)
        }
        32 => (0, bitset(input, 31)),
        _ => (0, false),
    }
}

pub fn asr_c(input: u32, shift: u32) -> (u32, bool) {
    // p27
    assert!(shift > 0);
    return match shift {
        1..=31 => {
            let result = ((input as i32) >> shift) as u32;
            let carry_out = bitset(input, shift - 1);
            (result, carry_out)
        }
        _ => {
            // 32 and greater cases are the same
            let result = ((input as i32) >> 31) as u32;
            (result, bitset(result, 0))
        }
    }
}

// NOTE: shift_n has already been adjusted for asr/lsr
pub fn shift_c(input: u32, shift_t: u32, shift_n: u32, carry_in: u32) -> (u32, bool) {
    // A7.4.2
    if shift_n == 0 {
        return (input, carry_in == 1);
    }
    return match shift_t {
        0b00 => lsl_c(input, shift_n),
        0b01 => lsr_c(input, shift_n),
        0b10 => asr_c(input, shift_n),
        0b11 => rrx_c(input, carry_in),
        0b100 => ror_c(input, shift_n),
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

// Replaces original<msbit:lsbit> (inclusive) with provider<msbit-lsbit:0>
pub fn bit_field_insert(original: u32, provider: u32, msbit: u32, lsbit: u32) -> u32 {
    let mask = get_mask(msbit, lsbit);
    let width = msbit - lsbit;
    let insert = (provider & (0xFFFF_FFFF >> (31 - width))) << lsbit;
    return (original & mask) | insert;
}

pub fn split_u64(large: u64) -> (u32, u32) {
    let upper = (large >> 32) as u32;
    let lower = (large & 0xFFFF_FFFF) as u32;
    return (upper, lower);
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_bitset() {
        for i in 0..32u32 {
            assert!(bitset(0xFFFF_FFFF, i));
            assert!(!bitset(0x0000_0000, i));
            assert_eq!(bitset(0x5555_5555, i), i % 2 == 0);
            assert_eq!(bitset(0x0010_0000, i), i == 20);
        }
    }

    #[test]
    fn test_matches() {
        assert!(matches(0b1100_1010, 4, 0b0110u32, 0b0100u32));
    }

    #[test]
    fn test_align() {
        assert_eq!(align(0b1111, 1), 0b1111);
        assert_eq!(align(0b1111, 2), 0b1110);
        assert_eq!(align(0b1111, 4), 0b1100);
    }

    #[test]
    fn test_shifted_sign_extend() {
        assert_eq!(shifted_sign_extend(0xFF, 7, 0), 0xFFFF_FFFF);
        assert_eq!(shifted_sign_extend(0xFF, 8, 0), 0x0000_00FF);
        assert_eq!(shifted_sign_extend(0xFF, 7, 4), 0xFFFF_FFF0);
        assert_eq!(shifted_sign_extend(0xAABB_77DD, 15, 4), 0x0007_7DD0);
        assert_eq!(shifted_sign_extend(0xAABB_87DD, 15, 4), 0xFFF8_7DD0);
        assert_eq!(shifted_sign_extend(0xAABB_87DD, 31, 0), 0xAABB_87DD);
    }

    #[test]
    fn test_ror_c() {
        assert_eq!(ror_c(0x0000_00AF, 4), (0xF000_000A, true));
        assert_eq!(ror_c(0x0000_00F7, 4), (0x7000_000F, false));
    }

    #[test]
    fn test_rrx_c() {
        assert_eq!(rrx_c(0x0000_0001, 1), (0x8000_0000, true));
        assert_eq!(rrx_c(0x0000_0001, 0), (0x0000_0000, true));
        assert_eq!(rrx_c(0xAAAA_AAAA, 1), (0xD555_5555, false));
    }

    #[test]
    fn test_lsl_c() {
        assert_eq!(lsl_c(0xFFFF_FFFF, 255), (0x0000_0000, false));
        assert_eq!(lsl_c(0xFFFF_FFFF, 32), (0x0000_0000, true));
        assert_eq!(lsl_c(0xFFFF_FFFF, 1), (0xFFFF_FFFE, true));
        assert_eq!(lsl_c(0x0001_ABCD, 16), (0xABCD_0000, true));
        assert_eq!(lsl_c(0x0000_ABCD, 16), (0xABCD_0000, false));
    }

    #[test]
    fn test_lsr_c() {
        assert_eq!(lsr_c(0xFFFF_FFFF, 255), (0x0000_0000, false));
        assert_eq!(lsr_c(0xFFFF_FFFF, 32), (0x0000_0000, true));
        assert_eq!(lsr_c(0xFFFF_FFFF, 1), (0x7FFF_FFFF, true));
        assert_eq!(lsr_c(0xAABB_CC80, 8), (0x00AA_BBCC, true));
        assert_eq!(lsr_c(0xAABB_CC00, 8), (0x00AA_BBCC, false));
    }

    #[test]
    fn test_asr_c() {
        assert_eq!(asr_c(0x8000_0000, 255), (0xFFFF_FFFF, true));
        assert_eq!(asr_c(0x7000_0000, 255), (0x0000_0000, false));
        assert_eq!(asr_c(0x8000_0000, 32), (0xFFFF_FFFF, true));
        assert_eq!(asr_c(0x7000_0000, 32), (0x0000_0000, false));
        assert_eq!(asr_c(0xFFFF_FFFF, 1), (0xFFFF_FFFF, true));
        assert_eq!(asr_c(0xAABB_CC80, 8), (0xFFAA_BBCC, true));
        assert_eq!(asr_c(0xAABB_CC00, 8), (0xFFAA_BBCC, false));
        assert_eq!(asr_c(0xFFFF_FFD4, 0x27), (0xFFFF_FFFF, true));
    }

    #[test]
    fn test_add_with_carry() {
        assert_eq!(add_with_carry(0xFFFF_FFFF, 0xFFFF_FFFF, 0), (0xFFFF_FFFE, true, false));
        assert_eq!(add_with_carry(0xFFFF_FFFF, 0xFFFF_FFFF, 1), (0xFFFF_FFFF, true, false));
        assert_eq!(add_with_carry(0x0000_FFFF, 0xFFFF_0000, 1), (0x0000_0000, true, false));
        assert_eq!(add_with_carry(0x7FFF_FFFF, 0x0000_0001, 0), (0x8000_0000, false, true));
        assert_eq!(add_with_carry(0x7FFF_FFFF, 0x7FFF_FFFF, 0), (0xFFFF_FFFE, false, true));
    }

    #[test]
    fn test_bit_field_clear() {
        assert_eq!(bit_field_clear(0xFFFF_FFFF, 0, 0), 0xFFFF_FFFE);
        assert_eq!(bit_field_clear(0xFFFF_FFFF, 31, 31), 0x7FFF_FFFF);
        assert_eq!(bit_field_clear(0xFFFF_FFFF, 31, 0), 0x0000_0000);
        assert_eq!(bit_field_clear(0xFFFF_FFFF, 10, 8), 0xFFFF_F8FF);
    }

    #[test]
    fn test_bit_field_insert() {
        assert_eq!(bit_field_insert(0x0000_0000, 0xFFFF_FFFF, 0, 0), 0x0000_0001);
        assert_eq!(bit_field_insert(0x0000_0000, 0xFFFF_FFFF, 31, 31), 0x8000_0000);
        assert_eq!(bit_field_insert(0x0000_0000, 0xFFFF_FFFF, 31, 0), 0xFFFF_FFFF);
        assert_eq!(bit_field_insert(0x0000_0000, 0x0000_0007, 10, 8), 0x0000_0700);
        assert_eq!(bit_field_insert(0xAAAA_AAAA, 0x0000_0005, 19, 16), 0xAAA5_AAAA);
    }

    #[test]
    fn test_split_u64() {
        assert_eq!(split_u64(0xFFFF_FFFF_FFFF_FFFF), (0xFFFF_FFFF, 0xFFFF_FFFF));
        assert_eq!(split_u64(0xAAAA_AAAA_BBBB_BBBB), (0xAAAA_AAAA, 0xBBBB_BBBB));
    }
}
