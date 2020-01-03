pub mod addr;
pub mod descriptor;
pub mod instructions;
pub mod cpu;
use core::fmt;
/// 系统特权级
#[derive(Debug, Copy, Clone, PartialEq, Eq)]
#[repr(u8)]
pub enum PrivilegedLevel {
    /// 特权级0
    Ring0 = 0,
    /// 特权级1
    Ring1 = 1,
    /// 特权级2
    Ring2 = 2,
    /// 特权级3
    Ring3 = 3,
}

impl PrivilegedLevel {
    pub fn from_u16(level: u16) -> PrivilegedLevel {
        match level {
            0 => PrivilegedLevel::Ring0,
            1 => PrivilegedLevel::Ring1,
            2 => PrivilegedLevel::Ring2,
            3 => PrivilegedLevel::Ring3,
            other => panic!("invalid privileged level `{}`", other),
        }
    }
}

/// 用于将u64以十六进制显示
#[repr(transparent)]
struct Hex(u64);

impl Hex {
    pub fn new(d: u64) -> Hex {
        Hex(d)
    }
}

impl fmt::Debug for Hex {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "{:#X}", self.0)
    }
}

/// 用于将u64以二进制显示
#[repr(transparent)]
struct Binary(u64);

impl fmt::Debug for Binary {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "{:#b}", self.0)
    }
}