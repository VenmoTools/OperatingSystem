#![feature(asm)]
#![no_std]
#![no_main]
#![deny(warnings)]
#[macro_use]
extern crate alloc;
#[macro_use]
extern crate kernel;

use system::ia_32e::cpu::control::CR3;
use system::ia_32e::instructions::interrupt::int3;
use system::ia_32e::paging::{Frame, Page, Page2MB, PageTable};
use system::ia_32e::paging::mapper::{MapAllSize, MappedPageTable, Mapper};
use system::ia_32e::VirtAddr;
use system::KernelArgs;
use uefi::ResultExt;
use uefi::table::{Runtime, SystemTable};
use uefi::table::boot::MemoryMapIter;

use kernel::{Initializer, loop_hlt};

fn get_mem_iter(args: &KernelArgs) -> &mut MemoryMapIter {
    unsafe { &mut *(args.iter as *mut MemoryMapIter) }
}

#[allow(dead_code)]
fn get_system_table(args: &KernelArgs) -> &SystemTable<Runtime> {
    unsafe { &*(args.st as *const SystemTable<Runtime>) }
}

#[cfg(feature = "efi")]
#[no_mangle]
pub extern "C" fn _start(args: u64) -> ! {
    println!("ptr:{:?}", args);
    let args = args - 64;
    //BUG: the args will offset 64
    let args = unsafe { &*((args) as *const KernelArgs) };
    let iter = get_mem_iter(args);
    // initialization all we need
    let initializer = Initializer::new(args, iter);
    initializer.initialize_all();
    // test memory allocate, interrupt handle, page table translate and uefi runtime service
    functional_test(args);
    kernel_start()
}

fn functional_test(args: &KernelArgs) {
    // test page table work
    let mut page_table = unsafe { MappedPageTable::from_cr3(frame_to_pt) };
    match page_table.translate_page(Page::<Page2MB>::include_address(VirtAddr::new(0x2000))) {
        Ok(frame) => println!("{:?}", frame),
        Err(e) => println!("{:?}", e)
    }
    // test buddy system work
    let _v = vec![1, 2, 3];
    // test use uefi runtime service
    let rt = get_system_table(args);
    println!("{:?}", unsafe { rt.runtime_services().get_time().log_warning().unwrap() });
    // test interrupt work
    int3();
}

pub fn frame_to_pt(f: Frame) -> *mut PageTable {
    f.start_address().as_u64() as *mut PageTable
}

fn kernel_start() -> ! {
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

//todo: symbol trace
#[allow(unused_assignments)]
pub fn stack_trace() {
    let mut rbp: usize = 0;
    unsafe { asm!("" : "={rbp}"(rbp) : : : "intel", "volatile"); }
    println!("TRACE: {:>016X}", rbp);
    let pt = unsafe { MappedPageTable::from_cr3(frame_to_pt) };
    for _f in 0..64 {
        if let Some(rip) = rbp.checked_add(core::mem::size_of::<usize>()) {
            if pt.translate(VirtAddr::new(rbp as u64)).is_ok() && pt.translate(VirtAddr::new(rip as u64)).is_ok() {
                let rip = unsafe { *(rip as *const usize) };
                if rip == 0 {
                    println!(" {:>016X}: EMPTY RETURN", rbp);
                    break;
                }
                println!("  {:>016X}: {:>016X}", rbp, rip);
                rbp = unsafe { *(rbp as *const usize) };
            } else {
                println!("  {:>016X}: GUARD PAGE", rbp);
                break;
            }
        } else {
            println!("  {:>016X}: RBP OVERFLOW", rbp);
            break;
        }
    }
}

// already mapped
pub fn show_page_table() {
    let (frame, _) = CR3::read();
    let f = frame.start_address().as_u64();
    let pt4 = unsafe { &*(f as *const PageTable) };
    use system::bits::flags::PageTableFlags;
    for e in pt4.iter() {
        if !e.is_unused() {
            let pt3 = unsafe { &*(e.addr().as_u64() as *const PageTable) };
            for e2 in pt3.iter() {
                if !e2.is_unused() && e2.flags().contains(PageTableFlags::ACCESSED) {
                    let pt2 = unsafe { &*(e2.addr().as_u64() as *const PageTable) };
                    for e3 in pt2.iter() {
                        if !e3.is_unused() && e3.flags().contains(PageTableFlags::ACCESSED) {
                            println!("{:?}", e3);
                        }
                    }
                }
            }
        }
    }
}

