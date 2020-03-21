// 初始化UART，并通过串行端口发送数据
// 这样可以直接与本地控制台传送数据

use spin::Mutex;
use system::console::Writer;
use uart_16550::SerialPort;

use lazy_static::lazy_static;

// 使用懒加载进行静态初始化
lazy_static! {
    pub static ref SERIAL: Mutex<SerialPort> = {
        let mut serial = unsafe{ SerialPort::new(0x3F8) };
        serial.init();
        Mutex::new(serial)
    };
}

pub struct PortPrinter {
    inner: SerialPort,
}

impl PortPrinter {
    pub fn new() -> Self {
        let mut serial = unsafe { SerialPort::new(0x3F8) };
        serial.init();
        Self {
            inner: serial
        }
    }
}

impl Writer for PortPrinter {
    fn print_to(&mut self, arg: core::fmt::Arguments) {
        use core::fmt::Write;
        use system::ia_32e::instructions::interrupt::without_interrupts;
        without_interrupts(|| {
            self.inner.write_fmt(arg).expect("Printing to Serial failed!");
        });
    }
}

#[doc(hidden)]
pub fn _print(arg: ::core::fmt::Arguments) {
    use core::fmt::Write;
    use system::ia_32e::instructions::interrupt::without_interrupts;
    without_interrupts(|| {
        SERIAL.lock().write_fmt(arg).expect("Printing to Serial failed!");
    });
}

#[macro_export]
macro_rules! print {
    ($($arg:tt)*) => {
        $crate::serial::_print(format_args!($($arg)*));
    };
}

#[macro_export]
macro_rules! println {
    ()=>($crate::print!("\n"));
    ($fmt:expr) => ($crate::print!(concat!($fmt,"\n")));
    ($fmt:expr, $($args:tt)*) => ($crate::print!(concat!($fmt,"\n"),$($args)*));
}
