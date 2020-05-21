use bitflags::_core::sync::atomic::Ordering;
use lazy_static::lazy_static;
use spin::Mutex;
use system::interrupt;

use crate::{print, println};
use crate::descriptor::{CONTROLLER, InterruptIndex, TICKS};
use crate::process::scheduler::switch;

interrupt!(timer,{
    println!("1");
    if TICKS.fetch_add(1,Ordering::SeqCst) >= 10{
        switch();
    }
    CONTROLLER.lock().eoi(Some(InterruptIndex::Timer.into()))
});

interrupt!(keyboard,{
    use system::ia_32e::cpu::Port;
    use pc_keyboard::{Keyboard, layouts,HandleControl, ScancodeSet1, DecodedKey};

    lazy_static! {
        static ref KEYBOARD:Mutex<Keyboard<layouts::Us104Key,ScancodeSet1>> =
        Mutex::new(Keyboard::new(layouts::Us104Key,ScancodeSet1,HandleControl::MapLettersToUnicode));
    }

    let mut keyboard = KEYBOARD.lock();
    let mut port = Port::new(0x60);
    let scan_code: u8 = port.read();

    if let Ok(Some(key_event)) = keyboard.add_byte(scan_code) {
        if let Some(key) = keyboard.process_keyevent(key_event) {
            match key {
                DecodedKey::Unicode(character) => print!("{}", character),
                DecodedKey::RawKey(key) => print!("{:?}", key),
            }
        }
    }
    CONTROLLER.lock().eoi(Some(InterruptIndex::KeyBoard.into()))
});

interrupt!(com2,{
    CONTROLLER.lock().eoi(Some(InterruptIndex::Com2.into()))
});
interrupt!(com1,{
    CONTROLLER.lock().eoi(Some(InterruptIndex::Com1.into()))
});

interrupt!(lpt2, {
    CONTROLLER.lock().eoi(Some(InterruptIndex::Lpt2.into()));
});

interrupt!(floppy, {
    CONTROLLER.lock().eoi(Some(InterruptIndex::Floppy.into()));
});

interrupt!(lpt1, {
    CONTROLLER.lock().eoi(Some(InterruptIndex::Lpt1.into()));
});

interrupt!(rtc, {
    CONTROLLER.lock().eoi(Some(InterruptIndex::Rtc.into()));
});

interrupt!(pci1, {
    CONTROLLER.lock().eoi(Some(InterruptIndex::Pci1.into()));
});

interrupt!(pci2, {
    CONTROLLER.lock().eoi(Some(InterruptIndex::Pci2.into()));
});

interrupt!(pci3, {
    CONTROLLER.lock().eoi(Some(InterruptIndex::Pci3.into()));
});

interrupt!(mouse, {
    CONTROLLER.lock().eoi(Some(InterruptIndex::Mouse.into()));
});

interrupt!(fpu, {
    CONTROLLER.lock().eoi(Some(InterruptIndex::Fpu.into()));
});

interrupt!(ata1, {
    CONTROLLER.lock().eoi(Some(InterruptIndex::Ata1.into()));
});

interrupt!(ata2, {
    CONTROLLER.lock().eoi(Some(InterruptIndex::Ata2.into()));
});