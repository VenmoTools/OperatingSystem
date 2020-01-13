// VGA文本缓冲区是一个二维数组，通常具有25行80列，直接显示在屏幕上
// 0-7      位为  ASCII字符内码列表为 Code page 437 最初的IBM PC代码页，实现了扩展ASCII字符集 // 8-11     位为  前景色 // 12-14	位为  背景颜色
// 15       位为  闪烁文字

use core::fmt;

use system::Mutex;
use volatile::Volatile;

use lazy_static::lazy_static;

pub const BUFFER_HEIGHT: usize = 25;
pub const BUFFER_WIDTH: usize = 80;



// static 懒加载无需在编译时计算其值，而是在首次访问时进行初始化
lazy_static! {
    // 该互斥锁不需要操作系统功能
    // 保证内部可变性是安全的
    pub static ref WRITER: Mutex<Writer> = Mutex::new(
    Writer {
        row_position: 0,
        column_position: 0,
        color_code: ColorCode::new(Color::White, Color::Black),
        buffer: unsafe { &mut *(0xb8000 as *mut Buffer) },
    });
}

#[doc(hidden)]
pub fn _print(arg: fmt::Arguments) {
    use core::fmt::Write;
    use x86_64::instructions::interrupts;
    //引入时钟中断后该行代码将会引起死锁 WRITER.lock().write_fmt(arg).unwrap();
    // without_interrupts函数将会 运行给定的闭包，在运行中断之前禁用中断（如果尚未禁用）。
    // 如果以前启用了中断，则再次启用中断
    interrupts::without_interrupts(|| {
        WRITER.lock().write_fmt(arg).unwrap();
    });
}

#[doc(hidden)]
pub fn _log(line: &str) {
    use core::{line, file};
    _print(format_args!("{} ,in file:{},at line:{} ", line, file!(), line!()));
}

#[macro_export]
macro_rules! log {
    ($arg:tt) => (crate::devices::vga::_log($arg));
}

#[macro_export]
macro_rules! print {
    ($($arg:tt)*) => (crate::devices::vga::_print(format_args!($($arg)*)));
}

#[macro_export]
macro_rules! println {
    ($($arg:tt)*) => ($crate::print!("{}\n",format_args!($($arg)*)));
}

#[allow(dead_code)]
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
// 指定字节对齐方式
#[repr(u8)]
pub enum Color {
    // 黑色
    Black = 0,
    // 蓝色
    Blue = 1,
    // 绿色
    Green = 2,
    // 青色
    Cyan = 3,
    // 红色
    Red = 4,
    // 品红色
    Magenta = 5,
    // 棕色
    Brown = 6,
    // 浅灰色
    LightGray = 7,
    // 深灰色
    DarkGray = 8,
    // 浅蓝色
    LightBlue = 9,
    // 浅绿色
    LightGreen = 10,
    // 浅青色
    LightCyan = 11,
    // 亮红色
    LightRed = 12,
    // 粉色
    Pink = 13,
    // 黄色
    Yellow = 14,
    // 白色
    White = 15,
}

#[derive(Debug, Copy, Clone, Eq, PartialEq)]
#[repr(transparent)]
pub struct ColorCode(u8);

impl ColorCode {
    pub fn new(foreground: Color, background: Color) -> ColorCode { // foreground: 前景色 // background: 背景色
        ColorCode((background as u8) << 4 | foreground as u8)
    }
}


#[derive(Debug, Copy, Clone, Eq, PartialEq)]
#[repr(C)]
struct ScreenChar {
    // ASCII 字符
    ascii_char: u8,
    // 字符颜色
    color: ColorCode,
}

pub struct Buffer {
    // 二维数组，使用VGA 模式显示文本
    chars: [[Volatile<ScreenChar>; BUFFER_WIDTH]; BUFFER_HEIGHT],
}

// Writer总是以\n结尾
pub struct Writer {
    // 当前行号
    pub row_position: usize,
    // 当前列号
    pub column_position: usize,
    // 字符颜色
    pub color_code: ColorCode,
    // 屏幕缓冲区 生命周期位整个程序运行时间
    pub buffer: &'static mut Buffer,
}


impl Writer {
    pub fn write_color_bytes(&mut self, byte: u8, color: ColorCode) {
        match byte {
            b'\n' => self.new_line(),
            byte => {
                // 检查是否超过每行最大字符数
                if self.column_position >= BUFFER_WIDTH {
                    // 超过需要换行
                    self.new_line()
                }

                let row = self.row_position;
                let col = self.column_position;
                // 使用write方法来代替= 保证编译器将永远不会优化写操作。
                self.buffer.chars[row][col].write(ScreenChar {
                    ascii_char: byte,
                    color,
                });

                self.column_position += 1;
            }
        }
    }


    pub fn write_bytes(&mut self, byte: u8) {
        self.write_color_bytes(byte, self.color_code);
    }

    pub fn write_color_string(&mut self, s: &str, color: ColorCode) {
        for byte in s.bytes() {
            match byte {
                //32(空格) - 126(~)
                0x20..=0x7e | b'\n' => self.write_color_bytes(byte, color),
                // 不是ASCII可打印字符 会打印■
                _ => self.write_bytes(0xfe),
            }
        }
    }

    pub fn write_string(&mut self, s: &str) {
        self.write_color_string(s, self.color_code);
    }

    fn new_line(&mut self) {
        // 如果超过最大行数
        if self.row_position >= BUFFER_HEIGHT {
            // 清空屏幕
            self.clear_screen();
            self.row_position = 0;
        }
        self.row_position += 1;
        self.column_position = 0;
    }

    fn clear_screen(&mut self) {
        let blank = ScreenChar {
            ascii_char: b' ',
            color: self.color_code,
        };
        for row in 0..BUFFER_HEIGHT {
            for col in 0..BUFFER_WIDTH {
                self.buffer.chars[row][col].write(blank);
            }
        }
    }
}


impl fmt::Write for Writer {
    fn write_str(&mut self, s: &str) -> fmt::Result {
        self.write_string(s);
        Ok(())
    }
}









