pub mod keyboard;
pub mod vga;

pub fn device_init() {
    keyboard::init();
}