use system::ia_32e::ApicInfo;
use system::ia_32e::instructions::interrupt::{disable_interrupt, enable_interrupt};
use system::ia_32e::paging::frame_allocator::MemoryAreaManagement;
use system::SystemInformation;

use crate::descriptor::{init_gdt, init_idt, init_tss};
use crate::memory::{add_to_heap, FRAME_ALLOCATOR, init_frame_allocator, RECU_PAGE_TABLE};
use crate::process::{init_process, process_mut};
use crate::utils::initialize_apic;

pub struct Initializer(SystemInformation);

impl Initializer {
    pub fn new(info: SystemInformation) -> Self {
        Self(info)
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
        init_frame_allocator(self.0.kernel_start(), self.0.kernel_end());
        {
            let mut allocator = FRAME_ALLOCATOR.lock();
            println!("add memory area");
            for area in self.0.mem_area_iter() {
                let adder = allocator.as_mut().unwrap();
                adder.add_area(area.start_addr, area.end_addr, area.ty, area.length);
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
