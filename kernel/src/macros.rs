#[macro_export]
macro_rules! print {
    ($($arg:tt)*) => {
        $crate::devices::vga::_print(format_args!($($arg)*));
        // $crate::serial::_print(format_args!($($arg)*));
    };
}

#[macro_export]
macro_rules! println {
    ()=>($crate::print!("\n"));
    ($fmt:expr) => ($crate::print!(concat!($fmt,"\n")));
    ($fmt:expr, $($args:tt)*) => ($crate::print!(concat!($fmt,"\n"),$($args)*));
}