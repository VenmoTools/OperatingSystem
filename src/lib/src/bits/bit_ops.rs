
use core::ops::Range;

/// 实现 u8 u16 u32 u64 usize i8 i16 i32 i64 isize 等类型的未操作
pub trait BitOpt {
    /// 用于返回当前操作数的位长度
    fn length() -> usize;
    /// 用于返回第size位的值
    fn get_bit(&self, size: usize) -> bool;
    /// 用于获取[start,end)范围的比特位
    fn get_bits(&self, range: Range<usize>) -> Self;
    /// 获取设置第bit位的值
    fn set_bit(&mut self, bit: usize, value: bool) -> &mut Self;
    /// 用于设置[start,end)范围的比特位
    fn set_bits(&mut self, range: Range<usize>, value: Self) -> &mut Self;
}


macro_rules! bit_opt_impl {
    ( $($t:ty)* ) => ($(
        impl BitOpt for $t {
            fn length() -> usize {
                ::core::mem::size_of::<Self>() as usize * 8
            }

            fn get_bit(&self, size: usize) -> bool {
                assert!(size < Self::length());
                (*self & (1 << size)) != 0
            }

            fn get_bits(&self, range: Range<usize>) -> Self {
                assert!(range.start < Self::length());
                assert!(range.end <= Self::length());
                assert!(range.end > range.start);

                let shift_bits = Self::length() - range.end;
                let bits = *self << shift_bits >> shift_bits;

                bits >> range.start
            }

            fn set_bit(&mut self, bit: usize, value: bool) -> &mut Self {
                assert!(bit < Self::length());
                let mask = 1 << bit;
                if value {
                    *self |= mask;
                } else {
                    *self &= !(mask);
                }
                self
            }

            fn set_bits(&mut self, range: Range<usize>, value: Self) -> &mut Self {
                let length = Self::length();
                assert!(range.start < length);
                assert!(range.end <= length);
                assert!(range.start < range.end);
                assert!(value << (length - (range.end - range.start)) >>
                        (length - (range.end - range.start)) == value,
                        "value does not fit into bit range");

                let mask: Self = !(!0 << (length - range.end) >>
                                    (length - range.end) >>
                                    range.start << range.start);

                *self = (*self & mask) | (value << range.start);

                self
            }
        }
    )*)
}

bit_opt_impl! { u8 u16 u32 u64 usize i8 i16 i32 i64 isize }