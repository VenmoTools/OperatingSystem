#![no_std]
#![no_main]
#![deny(warnings)]
#[cfg(feature = "bios")]
#[macro_use]
extern crate kernel;
#[cfg(feature = "efi")]
extern crate kernel;

use system::ia_32e::PhysAddr;

use kernel::{Initializer, loop_hlt};
use kernel::graphic::frame::RawFrameBuffer;
use kernel::KernelArgs;

struct Writer {
    raw: RawFrameBuffer,
    line: usize,
    col: usize,
}

impl Writer {
    pub fn new(frame: RawFrameBuffer) -> Self {
        Self {
            raw: frame,
            line: 0,
            col: 0,
        }
    }

    fn line_(&self) -> usize {
        self.line * 1024 * 20
    }

    pub fn write<T: Clone + Copy>(&mut self, value: T) {
        let next = self.line_() + self.col;
        assert!(self.col <= self.raw.max_size());
        for i in self.line_()..next {
            unsafe { self.raw.write_value(i, value) };
        }
        if self.col >= 1024 {
            self.col = 0;
        } else {
            self.col += 20;
        }
    }

    pub fn write_line<T: Clone + Copy>(&mut self, value: T) {
        self.col = 20;
        self.write(value);
        self.line += 1;
    }
}


#[cfg(feature = "efi")]
#[no_mangle]
pub extern "C" fn _start(_args: KernelArgs) -> ! {
    let zero = [0x0045, 0x00, 0x00, 0x00, 0x00, 0xFE, 0x66, 0x62, 0x60, 0x68, 0x78, 0x68, 0x60, 0x60, 0x62, 0x66, 0xFE, 0x00, 0x00, 0x00, 0x00];
    let video_frame = PhysAddr::new(0x80000000);
    let mut writer = Writer::new(RawFrameBuffer::from_raw(video_frame.as_mut() as *mut u8));
    for _i in 0..100 {
        writer.write_line(zero);
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


