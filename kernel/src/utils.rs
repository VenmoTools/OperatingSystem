use raw_cpuid::CpuId;
use system::ia_32e::{ApicInfo, PhysAddr, VirtAddr};
use system::ia_32e::cpu::control::CR3;
use system::ia_32e::paging::PageTable;
use system::ia_32e::paging::result::FrameError;

use crate::descriptor::init_apic;

#[derive(Default, Debug)]
pub struct SystemFunctionalCheck {
    xapic: bool,
    x2apic: bool,
    acpi: bool,
    msr: bool,
    initial_local_apic_id: u8,
}

macro_rules! attr_impl {
    ($name:ident,$attr:ident,$result:ident) => {
        pub fn $name(&self) -> $result{
            self.$attr
        }
    };
}

impl SystemFunctionalCheck {
    pub fn get_check_result() -> Self {
        let cpuid = CpuId::new();
        return match cpuid.get_feature_info() {
            Some(info) => {
                Self {
                    x2apic: info.has_x2apic(),
                    xapic: info.has_apic(),
                    acpi: info.has_acpi(),
                    msr: info.has_msr(),
                    initial_local_apic_id: info.initial_local_apic_id(),
                }
            }
            None => Default::default()
        };
    }

    attr_impl!(support_x2apic,x2apic,bool);
    attr_impl!(support_xapic,xapic,bool);
    attr_impl!(support_acpi,acpi,bool);
    attr_impl!(support_msr,msr,bool);
    attr_impl!(initial_local_apic_id,initial_local_apic_id,u8);
}

pub fn loop_hlt() -> ! {
    loop {
        system::ia_32e::instructions::interrupt::hlt();
    }
}

pub fn initialize_apic(info: ApicInfo) -> Result<(), &'static str> {
    let result = SystemFunctionalCheck::get_check_result();
    if cfg!(x2apic) && !result.support_x2apic() {
        return Err("the cpu don't support x2apic");
    }
    if cfg!(xapic) && !result.support_acpi() {
        return Err("the cpu don't support xapic");
    }
    init_apic(info);
    Ok(())
}

/// 此函数只能调用一次 避免给`＆mut`引用造成别名
pub unsafe fn get_pml4t(offset: VirtAddr) -> &'static PageTable {
    let (frame, _) = CR3::read();
    let phys = frame.start_address();
    let virt = offset + phys.as_u64();
    let ptr: *mut PageTable = virt.as_mut_ptr();
    &mut *ptr
}

pub unsafe fn translate_address(addr: VirtAddr, offset: VirtAddr) -> Option<PhysAddr> {
    __translate_address(addr, offset)
}

fn __translate_address(addr: VirtAddr, offset: VirtAddr) -> Option<PhysAddr> {
    let (frame, _) = CR3::read();
    let index = &[addr.page4_index(), addr.page3_index(), addr.page2_index(), addr.page1_index()];
    let mut frame = frame;
    for &i in index.iter() {
        let virt = offset + frame.start_address().as_u64();
        let ptr: *const PageTable = virt.as_ptr();
        let table = unsafe { &*ptr };
        let entry = &table[i];
        frame = match entry.frame() {
            Ok(f) => f,
            Err(FrameError::FrameNotPresent) => return None,
            Err(FrameError::HugeFrame) => {
                println!("addr: {:#?} is already mapped", addr.as_u64());
                return None;
            }
        }
    }

    Some(frame.start_address() + u64::from(addr.page_offset()))
}
