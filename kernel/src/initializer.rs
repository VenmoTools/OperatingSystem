use multiboot2::{BootInformation, MemoryAreaType};
use system::ia_32e::{align_down, ApicInfo};
use system::ia_32e::instructions::interrupt::{disable_interrupt, enable_interrupt};
use system::ia_32e::paging::MemoryType;
use system::ia_32e::paging::frame_allocator::MemoryAreaManagement;

use crate::descriptor::{init_gdt, init_idt, init_tss};
use crate::memory::{add_to_heap, FRAME_ALLOCATOR, init_frame_allocator, RECU_PAGE_TABLE};
use crate::process::{init_process, process_mut};
use crate::utils::initialize_apic;

pub struct Initializer<'a>(&'a BootInformation);

impl<'a> Initializer<'a> {
    pub fn new(boot: &'a BootInformation) -> Self {
        Self(boot)
    }
    pub fn initialize(&self) {
        // init
        init_idt();
        println!("set up idt... done");
        // init gdt
        init_gdt();
        println!("set up gdt... done");
        init_tss();
        println!("set up tss... done");
        // init heap
        // todo:
        unsafe {
            add_to_heap(1048576, 134086656)
        }
        println!("set up buddy system allocator... done");
        // init apic
        disable_interrupt();
        #[cfg(feature = "pic")]
            initialize_apic(ApicInfo::default()).expect("init apic failed");
        #[cfg(feature = "xapic")]
            initialize_apic(ApicInfo::new().set_io_apic_offset(8).build().unwrap()).expect("init apic failed");
        //todo: x2apic build
        #[cfg(feature = "x2apic")]
            initialize_apic(ApicInfo::new()
            .build().unwrap()
        ).expect("init apic failed");
        println!("enable apic or pic... done");
        enable_interrupt();
        println!("enable interrupt... done");
        {
            RECU_PAGE_TABLE.lock();
        }
        init_frame_allocator(self.0);
        {
            let mut allocator = FRAME_ALLOCATOR.lock();
            println!("add memory area");
            for area in self.0.memory_map_tag().unwrap().memory_areas() {
                if area.start_address() == 0 {
                    continue;
                }
                let ty = match area.typ() {
                    MemoryAreaType::AcpiAvailable => MemoryType::ACPIArea,
                    MemoryAreaType::Reserved => MemoryType::ReservedArea,
                    MemoryAreaType::Available => MemoryType::FreeArea,
                    MemoryAreaType::ReservedHibernate => MemoryType::ReservedHibernate,
                    MemoryAreaType::Defective => MemoryType::Defective,
                };
                let mut addr = align_down(area.start_address(), 0x1000);
                println!("{:#X?}", addr);
                let adder = allocator.as_mut().unwrap();
                while addr < 0x100000 * 10 {
                    addr += 0x1000;
                    adder.add_area(addr, addr + 0x1000, ty, 0x1000);
                }
            }
        }
        println!("init frame allocator... done");
        init_process();
        println!("init first process... done");
        process_mut().spawn(|| {
            let mut i = 0;
            while i < 5 {
                println!("new process");
                i += 1;
            }
        }).expect("new process error");
    }
}
