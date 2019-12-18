use crate::ia_32e::descriptor::{Descriptor, SegmentSelector, DescriptorTablePointer};
use crate::ia_32e::PrivilegedLevel;


#[derive(Debug, Clone)]
pub struct GlobalDescriptorTable {
    table: [u64; 8],
    next_free: usize,
}

impl GlobalDescriptorTable {
    pub fn new() -> GlobalDescriptorTable {
        GlobalDescriptorTable {
            table: [0; 8],
            next_free: 1,
        }
    }

    fn push(&mut self, value: u64) -> usize {
        if self.next_free < self.table.len() {
            let index = self.next_free;
            self.table[index] = value;
            self.next_free += 1;
            return index;
        }
        panic!("GDT max descriptor length is 8")
    }

    pub fn add_descriptor(&mut self, descr: Descriptor) -> SegmentSelector {
        let index = match descr {
            Descriptor::UserSegment(value) => self.push(value),
            Descriptor::SystemSegment(value_low, value_hight) => {
                let index = self.push(value_low);
                self.push(value_hight);
                index
            }
        };
        SegmentSelector::new(index as u16, PrivilegedLevel::Ring0)
    }

    #[cfg(target_arch = "x86_64")]
    pub fn load(&'static self) {
        use crate::ia_32e::instructions::tables::ldgt;
        use core::mem::size_of;

        let ptr = DescriptorTablePointer {
            base: self.table.as_ptr() as u64,
            limit: (self.table.len() * size_of::<u64>() - 1) as u16,
        };

        unsafe {
            ldgt(&ptr);
        }
    }
}