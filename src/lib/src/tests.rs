use crate::bits::BitOpt;
use crate::ia_32e::cpu::pic::ChainedPics;
use crate::ia_32e::cpu::port::InOut;

#[test]
fn length_test() {
    assert_eq!(u8::length(), 8);
    assert_eq!(u16::length(), 16);
    assert_eq!(u32::length(), 32);
    assert_eq!(u64::length(), 64);

    assert_eq!(i8::length(), 8);
    assert_eq!(i16::length(), 16);
    assert_eq!(i32::length(), 32);
    assert_eq!(i64::length(), 64);
}

#[test]
fn test_set_bit_u8() {
    let mut field = 0b11110010u8;
    let mut func = |i| {
        field.set_bit(i, true);
        assert_eq!(field.get_bit(i), true);
        field.set_bit(i, false);
        assert_eq!(field.get_bit(i), false);
        field.set_bit(i, true);
        assert_eq!(field.get_bit(i), true);
    };
    for i in 0..8 {
        func(i);
    }
}

#[test]
fn test_set_bit_u16() {
    let mut field = 0b1111001010010110u16;
    let mut func = |i| {
        field.set_bit(i, true);
        assert_eq!(field.get_bit(i), true);
        field.set_bit(i, false);
        assert_eq!(field.get_bit(i), false);
        field.set_bit(i, true);
        assert_eq!(field.get_bit(i), true);
    };
    for i in 0..16 {
        func(i);
    }
}


#[test]
fn test_set_bit_u32() {
    let mut field = 0b1111111111010110u32;
    let mut func = |i| {
        field.set_bit(i, true);
        assert_eq!(field.get_bit(i), true);
        field.set_bit(i, false);
        assert_eq!(field.get_bit(i), false);
        field.set_bit(i, true);
        assert_eq!(field.get_bit(i), true);
    };
    for i in 0..32 {
        func(i);
    }
}

#[test]
fn test_set_bit_u64() {
    let mut field = 0b1111111111010110u64 << 32;
    let mut func = |i| {
        field.set_bit(i, true);
        assert_eq!(field.get_bit(i), true);
        field.set_bit(i, false);
        assert_eq!(field.get_bit(i), false);
        field.set_bit(i, true);
        assert_eq!(field.get_bit(i), true);
    };
    for i in 0..64 {
        func(i);
    }
}