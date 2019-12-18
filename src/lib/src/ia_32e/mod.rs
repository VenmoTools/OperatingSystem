
pub mod addr;
pub mod descriptor;
pub mod instructions;

#[derive(Debug, Copy, Clone, PartialEq, Eq)]
#[repr(u8)]
pub enum PrivilegedLevel {
    Ring0 = 0,
    Ring1 = 1,
    Ring2 = 2,
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

