use core::ops::Range;


pub trait BitOpt {
    fn length() -> usize;
    fn get_bit(&self, size: usize) -> bool;
    fn get_bits(&self, range: Range<usize>) -> Self;
    fn set_bit(&mut self, bit: usize, value: bool) -> &mut Self;
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
                assert!(range.start < Self::length());
                assert!(range.end <= Self::length());
                assert!(range.end > range.start);

                let shift_bits = Self::length() - range.end;
                let mask = !0 << shift_bits >> shift_bits >> range.start << range.start;

                *self = (*self & mask) | (value << range.start);

                self
            }
        }
    )*)
}

bit_opt_impl! { u8 u16 u32 u64 usize i8 i16 i32 i64 isize }