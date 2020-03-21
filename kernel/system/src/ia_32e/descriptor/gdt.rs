use crate::ia_32e::descriptor::{Descriptor, DescriptorTablePointer, SegmentSelector};
use crate::ia_32e::PrivilegedLevel;
use crate::bits::flags::{GdtAccessFlags, GdtFlags};
use core::fmt;


#[derive(Copy, Clone, Debug)]
#[repr(packed)]
pub struct GdtEntry {
    pub limit_low: u16,
    pub offset_low: u16,
    pub offset_mid: u8,
    pub access: u8,
    pub flags_limit_high: u8,
    pub offset_high: u8,
}


impl GdtEntry {
    pub fn new(offset: u32, limit: u32, access: GdtAccessFlags, flags: GdtFlags) -> Self {
        GdtEntry {
            limit_low: limit as u16,
            offset_low: offset as u16,
            offset_mid: (offset >> 16) as u8,
            access: access.bits(),
            flags_limit_high: flags.bits() & 0xF0 | ((limit >> 16) as u8) & 0x0F,
            offset_high: (offset >> 24) as u8,
        }
    }

    pub fn privileged_level(&self) -> PrivilegedLevel {
        let flags = GdtAccessFlags::from_bits_truncate(self.access);
        return if flags.contains(GdtAccessFlags::RING_0) {
            PrivilegedLevel::Ring0
        } else if flags.contains(GdtAccessFlags::RING_1) {
            PrivilegedLevel::Ring1
        } else if flags.contains(GdtAccessFlags::RING_1) {
            PrivilegedLevel::Ring2
        } else {
            PrivilegedLevel::Ring3
        };
    }
}

/// 在加载段选择子的过程中，处理器会将选择子作为索引，
/// 从GDTR寄存器指向的描述符表中索引(找出)段描述符，
/// GDTR寄存器是一个48位伪描述符(Pseudo-Descriptor)保存着全局描述符表的首地址和长度，
/// GDT不是段描述符，而是在线性地址中的一个数据结构，
/// GDT需要将自己的基地址（线性地址）和长度使用lgdt指令加载到GDTR寄存器中，
/// GDT的长度为8N-1(N为描述符的个数)
/// 全局描述符表的第0项被作为空选择子(NULL Segment Selector),
/// 处理器的CS和SS段寄存器不能加载空段，否则会发生#GP异常，
/// 其他寄存器可以使用空段选择子初始化
/// `GlobalDescriptorTable`会自动添加一个空段选择子，不需要手动添加！
/// # Example
///
/// ```
/// use system::ia_32e::descriptor::GlobalDescriptorTable;
/// use system::ia_32e::descriptor::Descriptor;
///
/// #[no_mangle]
/// pub extern "C" fn _start() -> ! {
///     let mut gdt = GlobalDescriptorTable::new();
///     gdt.add_descriptor(Descriptor::kernel_code_segment());
///     gdt.load()
/// }
/// ```
#[derive(Clone)]
pub struct GlobalDescriptorTable {
    table: [u64; 8],
    next_free: usize,
}

impl fmt::Debug for GlobalDescriptorTable {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        use crate::bits::flags::DescriptorFlags as Flags;
        f.write_str("GlobalDescriptorTable\n")?;
        for descriptor in self.table.iter() {
            let flags = Flags::from_bits_truncate(*descriptor);
            f.write_fmt(format_args!("descriptor: {:#X}\tflags:{:?}\n", descriptor, flags))?;
        }
        f.write_fmt(format_args!("total: {}", self.table.len()))
    }
}

impl GlobalDescriptorTable {
    /// 用于初始化全局描述符表，`next_free`用于添加的描述符个数
    /// 在初始化时已经添加过一个空段选择子，因此next_free为1
    pub fn new() -> GlobalDescriptorTable {
        GlobalDescriptorTable {
            table: [0; 8],
            next_free: 1,
        }
    }

    pub fn add_entry(&mut self, entry: GdtEntry) -> SegmentSelector{
        let rpl = entry.privileged_level();
        let e = &entry as *const _ as u64;
        let index = self.push(e);
        SegmentSelector::new(index as u16, rpl)
    }

    /// 从给定DescriptorTablePointer指针返回GDT结构,GDT项默认为8个
    pub unsafe fn from_ptr(ptr: DescriptorTablePointer) -> GlobalDescriptorTable {
        let table = core::slice::from_raw_parts(ptr.base as *const u64, 8);
        let mut t = [0_u64; 8];
        for i in 0..t.len() {
            t[i] = table[i].clone();
        }
        GlobalDescriptorTable {
            table: t,
            next_free: 1,
        }
    }
    /// 用于添加段选择子，如果添加的段选择子超过最大长度将会Panic
    /// 该方法是私有方法，只用于`add_descriptor`函数
    fn push(&mut self, value: u64) -> usize {
        if self.next_free < self.table.len() {
            let index = self.next_free;
            self.table[index] = value;
            self.next_free += 1;
            return index;
        }
        panic!("GDT max descriptor length is 8")
    }
    /// 将描述符注册到指定的索引处，SystemSegment描述符占用2项，因此0<index<=6
    pub fn add_indexed_descriptor(&mut self, index: usize, descr: Descriptor) {
        assert!(index > 0 && index < 8);
        let mut index = index;
        match descr {
            Descriptor::UserSegment(value) | Descriptor::KernelSegment(value) => self.table[index] = value,
            Descriptor::SystemSegment(value_low, value_high) => {
                assert!(index < 7);
                self.table[index] = value_low;
                index += 1;
                self.table[index] = value_high;
            }
        };
    }

    /// 添加描述符，添加描述符时会区分描述符的类型（用户，系统），再使用时需要指定当前描述符类型
    pub fn add_descriptor(&mut self, descr: Descriptor) -> SegmentSelector {
        let index = match descr {
            Descriptor::UserSegment(value) | Descriptor::KernelSegment(value) => self.push(value),
            Descriptor::SystemSegment(value_low, value_hight) => {
                let index = self.push(value_low);
                self.push(value_hight);
                index
            }
        };
        SegmentSelector::new(index as u16, PrivilegedLevel::Ring0)
    }

    /// 加载GDT描述符，加载描述符时需要将描述符结构转换为指针的形式
    /// 然后通过`system::ia_32e::instructions::tables::ldgt;`加载GDT
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