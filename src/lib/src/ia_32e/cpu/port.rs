#[cfg(any(target_arch = "x86", target_arch = "x86_64"))]
pub use crate::ia_32e::instructions::port::{inb, outb, inw, outw, inl, outl};
use core::marker::PhantomData;

pub trait InOut {
    unsafe fn port_in(port: u16) -> Self;
    unsafe fn port_out(port: u16, value: Self);
}

impl InOut for u8 {
    unsafe fn port_in(port: u16) -> Self {
        inb(port)
    }

    unsafe fn port_out(port: u16, value: Self) {
        outb(value, port)
    }
}

impl InOut for u16 {
    unsafe fn port_in(port: u16) -> Self {
        inw(port)
    }

    unsafe fn port_out(port: u16, value: Self) {
        outw(value, port)
    }
}

impl InOut for u32 {
    unsafe fn port_in(port: u16) -> Self {
        inl(port)
    }

    unsafe fn port_out(port: u16, value: Self) {
        outl(value, port)
    }
}

#[derive(Debug)]
pub struct Port<T> {
    port: u16,
    phantom: PhantomData<T>,
}

impl<T: InOut> Port<T> {
    pub const unsafe fn new(port: u16) -> Port<T> {
        Port { port, phantom: PhantomData{} }
    }

    pub fn read(&mut self) -> T {
        unsafe {
            T::port_in(self.port)
        }
    }

    pub fn write(&mut self, value: T) {
        unsafe {
            T::port_out(self.port, value)
        }
    }
}

#[derive(Debug)]
pub struct UnsafePort<T> {
    port: u16,
    phantom: PhantomData<T>,
}

impl<T: InOut> UnsafePort<T> {
    pub const unsafe fn new(port: u16) -> UnsafePort<T> {
        UnsafePort { port, phantom: PhantomData }
    }

    pub unsafe fn write(&mut self, value: T) {
        T::port_out(self.port, value)
    }

    pub unsafe fn read(&mut self) -> T {
        T::port_in(self.port)
    }
}