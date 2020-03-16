use crate::bits::flags::LVTEntryFlags;

pub struct LocalVectorTableEntry {
    entry: u32,
}

impl LocalVectorTableEntry{
    pub fn flags(&self) -> LVTEntryFlags{
        LVTEntryFlags::from_bits_truncate(self.entry)
    }
}