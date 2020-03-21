use crate::alloc::boxed::Box;
use crate::Mutex;

pub trait Writer {
    fn print_to(&mut self, arg: ::core::fmt::Arguments);
}

pub static mut CONSOLE: Mutex<Option<Box<dyn Writer + Sync>>> = Mutex::new(None);

pub unsafe fn set_console(printer: Box<dyn Writer + Sync>) {
    CONSOLE = Mutex::new(Some(printer));
}

#[macro_export]
macro_rules! print {
    ($($arg:tt)*) => {
        unsafe{
            if let Some(console) = $crate::console::CONSOLE.lock().as_mut(){
                console.print_to(format_args!($($arg)*))
            }
        }
    };
}

#[macro_export]
macro_rules! println {
    ()=>($crate::print!("\n"));
    ($fmt:expr) => ($crate::print!(concat!($fmt,"\n")));
    ($fmt:expr, $($args:tt)*) => ($crate::print!(concat!($fmt,"\n"),$($args)*));
}
