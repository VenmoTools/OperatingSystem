#![no_std]
#![no_main]
#![deny(warnings)]
#[cfg(feature = "bios")]
#[macro_use]
extern crate kernel;
#[cfg(feature = "efi")]
extern crate kernel;

use core::mem;

use uefi::prelude::SystemTable;
use uefi::table::Runtime;

use kernel::{Initializer, loop_hlt};

#[repr(C)]
pub struct KernelArgs {
    st: SystemTable<Runtime>,
    // iter: MemoryMapIter<'a>,
    frame_ptr: *mut u8,
    frame_size: usize,
}

pub struct RawFrameBuffer {
    size: usize,
    base: *mut u8,
}

impl RawFrameBuffer {
    pub fn new(args: &KernelArgs) -> Self {
        Self {
            size: args.frame_size,
            base: args.frame_ptr,
        }
    }
    pub fn from_raw(args: *mut u8) -> Self {
        Self {
            size: 3145728,
            base: args,
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

#[cfg(feature = "efi")]
#[no_mangle]
pub extern "C" fn _start(args: KernelArgs) -> ! {
    let mut args_frame = RawFrameBuffer::new(&args);
    for i in 0..100000 {
        unsafe { args_frame.write_value(i, 255) };
    }
    let mut frame = RawFrameBuffer::from_raw(0x80000000 as *mut u8);
    for i in 100000..200000 {
        unsafe { frame.write_value(i, 255) };
    }
    Initializer::initialize_all();
    loop_hlt()
}

#[cfg(feature = "bios")]
#[no_mangle]
pub extern "C" fn _start() -> ! {
    println!("Hello World");
    let cpuid = raw_cpuid::CpuId::new();
    println!("cpu info:{:?}", cpuid.get_vendor_info().unwrap().as_string());
    Initializer::initialize_all();
    loop_hlt()
}


