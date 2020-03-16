use core::marker::PhantomData;
use core::mem;

use crate::KernelArgs;

pub struct RawFrameBuffer<'a> {
    size: usize,
    base: *mut u8,
    _mark: PhantomData<&'a mut u8>,
}

impl<'a> RawFrameBuffer<'a> {
    pub fn new(args: &KernelArgs) -> Self {
        Self {
            size: args.frame_size,
            base: args.frame_ptr,
            _mark: PhantomData,
        }
    }
    pub fn max_size(&self) -> usize {
        self.size
    }
    pub fn from_raw(args: *mut u8) -> Self {
        Self {
            size: 3145728,
            base: args,
            _mark: PhantomData,
        }
    }
    /// Modify the i-th byte of the frame buffer
    ///
    /// # Safety
    ///
    /// This operation is unsafe because...
    /// - You must honor the pixel format and stride specified by the mode info
    /// - There is no bound checking on memory accesses in release mode
    #[inline]
    pub unsafe fn write_byte(&mut self, index: usize, value: u8) {
        debug_assert!(index < self.size, "Frame buffer accessed out of bounds");
        self.base.add(index).write_volatile(value)
    }

    /// Read the i-th byte of the frame buffer
    ///
    /// # Safety
    ///
    /// This operation is unsafe because...
    /// - You must honor the pixel format and stride specified by the mode info
    /// - There is no bound checking on memory accesses in release mode
    #[inline]
    pub unsafe fn read_byte(&self, index: usize) -> u8 {
        debug_assert!(index < self.size, "Frame buffer accessed out of bounds");
        self.base.add(index).read_volatile()
    }

    /// Write a value in the frame buffer, starting at the i-th byte
    ///
    /// We only recommend using this method with [u8; N] arrays. Once Rust has
    /// const generics, it will be deprecated and replaced with a write_bytes()
    /// method that only accepts [u8; N] input.
    ///
    /// # Safety
    ///
    /// This operation is unsafe because...
    /// - It is your reponsibility to make sure that the value type makes sense
    /// - You must honor the pixel format and stride specified by the mode info
    /// - There is no bound checking on memory accesses in release mode
    #[inline]
    pub unsafe fn write_value<T>(&mut self, index: usize, value: T) {
        debug_assert!(
            index.saturating_add(mem::size_of::<T>()) <= self.size,
            "Frame buffer accessed out of bounds"
        );
        (self.base.add(index) as *mut T).write_volatile(value)
    }

    /// Read a value from the frame buffer, starting at the i-th byte
    ///
    /// We only recommend using this method with [u8; N] arrays. Once Rust has
    /// const generics, it will be deprecated and replaced with a read_bytes()
    /// method that only accepts [u8; N] input.
    ///
    /// # Safety
    ///
    /// This operation is unsafe because...
    /// - It is your reponsibility to make sure that the value type makes sense
    /// - You must honor the pixel format and stride specified by the mode info
    /// - There is no bound checking on memory accesses in release mode
    #[inline]
    pub unsafe fn read_value<T>(&self, index: usize) -> T {
        debug_assert!(
            index.saturating_add(mem::size_of::<T>()) <= self.size,
            "Frame buffer accessed out of bounds"
        );
        (self.base.add(index) as *const T).read_volatile()
    }
}