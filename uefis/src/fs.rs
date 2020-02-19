use alloc::string::String;
use alloc::vec::Vec;
use core::borrow::Borrow;
use core::convert::TryInto;
use core::ptr;
use uefi::{Completion, ResultExt, Status};
use uefi::proto::media::file::{FileAttribute, FileType, RegularFile};
use uefi::proto::media::file::{Directory, File as efiFile, FileInfo, FileMode};
use uefi::proto::media::fs::SimpleFileSystem;
use uefi::table::boot::BootServices;
use uefi::table::boot::MemoryType;

use crate::result::{err, Error, ErrorKind, ok, Result, UefiResult};

pub struct FileOperator {
    file: RegularFile,
    current: u64,
}

/// 将此文件句柄的游标的位置设置为指定的绝对位置。
/// 允许使用`End`将游标设置超过文件末尾的位置，它将在下次写入时触发文件增长。
///
/// * `SeekFrom::Start(size)` 将游标移至文件起始位置`size`个字节处
/// * `SeekFrom::End(size)` 将游标移至设置为此对象的大小加上指定的`size`个字节处
/// * `SeekFrom::Current(size)` 将游标移至设置为当前位置加上指定的`size`个字节处
#[derive(Copy, Clone, Debug, Eq, PartialEq)]
pub enum SeekFrom {
    Start(u64),
    End(u64),
    Current(u64),
}


pub trait Seek {
    fn seek(&mut self, pos: SeekFrom) -> Result<()>;
}

pub trait Read {
    fn read(&mut self, buf: &mut [u8]) -> UefiResult<Result<usize>>;

    fn read_exact(&mut self, mut buf: &mut [u8]) -> Result<()> {
        while !buf.is_empty() {
            match self.read(buf).log_warning().unwrap() {
                Ok(0) => break,
                Ok(n) => {
                    let tmp = buf;
                    buf = &mut tmp[n..];
                }
                Err(ref e) if e.kind() == ErrorKind::Interrupted => {}
                Err(e) => return Err(e),
            }
        }
        if !buf.is_empty() {
            Err(Error::new(ErrorKind::UnexpectedEof, "failed to fill whole buffer"))
        } else {
            Ok(())
        }
    }

    #[inline]
    unsafe fn initializer(&self) -> Initializer {
        Initializer::zeroing()
    }

    fn read_to_end(&mut self, buf: &mut Vec<u8>) -> Result<usize> {
        read_to_end(self, buf)
    }
}

pub trait Write {
    fn write(&mut self, buf: &[u8]) -> usize;

    fn flush(&mut self) -> Result<()>;

    fn write_all(&mut self, mut buf: &[u8]) -> Result<()> {
        while !buf.is_empty() {
            match self.write(buf) {
                0 => {
                    return Err(Error::new(ErrorKind::WriteZero, "failed to write whole buffer"));
                }
                n => buf = &buf[n..],
            }
        }
        Ok(())
    }

    /// 关闭并删除整个文件 ,调用该方法后会产生所有权移动
    fn delete(self) -> Result<()>;
}

impl Seek for FileOperator {
    fn seek(&mut self, pos: SeekFrom) -> Result<()> {
        let result = match pos {
            SeekFrom::Start(p) => self.file.set_position(p).log_warning(),
            SeekFrom::End(p) => self.file.set_position(RegularFile::END_OF_FILE + p).log_warning(),
            SeekFrom::Current(p) => self.file.set_position(self.current + p).log_warning(),
        };
        match result {
            Ok(_) => Ok(()),
            Err(e) => Err(Error::from_uefi_status(e.status(), None))
        }
    }
}

impl Read for FileOperator {
    fn read(&mut self, buf: &mut [u8]) -> UefiResult<Result<usize>> {
        match self.file.read(buf).log_warning() {
            Ok(size) => {
                self.current += size as u64;
                ok(size)
            }
            Err(e) => {
                match e.data() {
                    Some(size) => err(Error::from_uefi_status(e.status(), Some(format!("buffer to small need {}", size).as_str()))),
                    None => err(Error::from_uefi_status(e.status(), None))
                }
            }
        }
    }
}

impl Write for FileOperator {
    fn write(&mut self, buf: &[u8]) -> usize {
        match self.file.write(buf).log_warning() {
            Ok(_) => buf.len(),
            Err(size) => *size.data()
        }
    }

    fn flush(&mut self) -> Result<()> {
        match self.file.flush().log_warning() {
            Ok(()) => Ok(()),
            Err(e) => Err(Error::from_uefi_status(e.status(), None))
        }
    }

    fn delete(self) -> Result<()> {
        let res = self.file.delete();
        match res {
            Ok(_) => Ok(()),
            Err(e) => Err(Error::from_uefi_status(e.status(), None))
        }
    }
}

pub struct File {
    root: Directory,
    buffer: Vec<u8>,
}

impl File {
    /// 根据BootServices创建File实例，
    /// # Panic
    /// 当尝试读取根目录失败后，文件系统出错，设备驱动等错误均会导致panic
    pub fn new(bt: &BootServices) -> Self {
        match File::try_new(bt) {
            Ok(f) => f,
            Err(e) => {
                panic!("occur an error during open root folder! {}", e);
            }
        }
    }

    pub fn _new(bt: &BootServices) -> UefiResult<Self> {
        let f = unsafe { &mut *bt.locate_protocol::<SimpleFileSystem>().log_warning()?.get() };
        let mut volume = f.open_volume().log_warning()?;
        Ok(Completion::from(File {
            root: volume,
            buffer: vec![0_u8; 4096],
        }))
    }

    /// new的方法
    pub fn try_new(bt: &BootServices) -> Result<Self> {
        match Self::_new(bt).log_warning() {
            Ok(f) => Ok(f),
            Err(e) => Err(Error::from_uefi_status(e.status(), None)),
        }
    }

    /// 指定缓冲区容量大小
    pub fn with_buffer_capacity(bt: &BootServices, size: usize) -> UefiResult<Self> {
        let f = unsafe { &mut *bt.locate_protocol::<SimpleFileSystem>().log_warning()?.get() };
        let volume = f.open_volume().log_warning()?;
        Ok(Completion::from(File {
            root: volume,
            buffer: vec![0_u8; size],
        }))
    }

    fn read_entry(&mut self) -> Result<&mut FileInfo> {
        return match self.root.read_entry(self.buffer.as_mut_slice()).log_warning() {
            Ok(info) => {
                if let Some(f) = info { Ok(f) } else { Err(Error::new(ErrorKind::NotFound, "the file info header not found!")) }
            }
            Err(e) => Err(Error::from_uefi_status(e.status(), None))
        };
    }

    fn root_attribute(&mut self) -> Result<FileAttribute> {
        match self.read_entry() {
            Ok(info) => Ok(info.attribute()),
            Err(e) => Err(e),
        }
    }

    /// 遍历指定目录
    ///
    /// # Usage
    ///
    /// ```
    /// #[entry]
    /// fn efi_main(image: Handle, st: SystemTable<Boot>) -> Status{
    ///     let file = File::new(st.boot_services());
    ///     file.walk_dir(r"EFI\Boot\",|filename, dir, info|{
    ///         info!("entry name {}",filename);
    ///     })
    /// }
    /// ```
    fn walk_dir(&mut self, folder_name: &str, mut func: impl FnMut(&String, &mut Directory, &mut FileInfo)) -> UefiResult<Result<()>> {
        let attr = self.root_attribute().unwrap();
        return match self.root.open(folder_name, FileMode::Read, attr).log_warning() {
            Ok(handle) => {
                if let FileType::Dir(mut sub) = handle.into_type().log_warning()? {
                    while let Ok(f_info) = sub.read_entry(self.buffer.as_mut_slice()) {
                        let (status, f) = f_info.split();
                        if status == Status::SUCCESS {
                            let files = f.unwrap();
                            let u16_slice = files.file_name().to_u16_slice();
                            let name = String::from_utf16_lossy(u16_slice);
                            func(&name, &mut sub, files)
                        }
                    }
                    ok(())
                } else {
                    err(Error::new(ErrorKind::InvalidFile, format!("{} is not folder!", folder_name).as_str()))
                }
            }
            Err(e) => err(Error::from_uefi_status(e.status(), None))
        };
    }


    /// 打开指定`filename`文件,`mode`
    ///
    /// # Arguments
    /// * `filename`    需要打开的文件的相对路径 路径分隔符为反斜杠`\\`
    /// * `mode`   文件打开的模式， `r`表示只读， `w`表示读取和写入， `c` 表示创建读取并写入
    ///
    /// # Usage
    ///
    /// ```
    /// #[entry]
    /// fn efi_main(image: Handle, st: SystemTable<Boot>) -> Status{
    ///     let file = File::new(st.boot_services());
    ///     file.open(r"EFI\Boot\kernel","r")
    /// }
    /// ```
    pub fn open(&mut self, filename: &str, mode: &str) -> UefiResult<Result<FileOperator>> {
        let attr = self.root_attribute().unwrap();
        let f_mode = match mode {
            "r" => FileMode::Read,
            "w" => FileMode::ReadWrite,
            "c" => FileMode::CreateReadWrite,
            other => return err(Error::new(ErrorKind::InvalidFileMode, format!("No Support mode: `{}`", other.clone()).as_str())),
        };
        self._open(filename, f_mode, attr)
    }

    /// `open`函数的底层方法
    fn _open(&mut self, filename: &str, mode: FileMode, mut attr: FileAttribute) -> UefiResult<Result<FileOperator>> {
        if let Ok(handle) = self.root.open(filename, mode, attr).log_warning() {}

        if let FileMode::CreateReadWrite = mode {
            attr = FileAttribute::VALID_ATTR;
        }

        return match self.root.open(filename, mode, attr).log_warning() {
            Ok(handle) => {
                match handle.into_type().log_warning()? {
                    FileType::Dir(_) => {
                        return err(Error::new(ErrorKind::InvalidFile, "except file found folder, if you want create folder please use `mkdir` method if you want read folder please use `walk` method"));
                    }
                    FileType::Regular(file) => {
                        ok(FileOperator { file, current: 0 })
                    }
                }
            }
            Err(e) => {
                err(Error::from_uefi_status(e.status(), None))
            }
        };
    }
}


struct Guard<'a> {
    buf: &'a mut Vec<u8>,
    len: usize,
}

impl Drop for Guard<'_> {
    fn drop(&mut self) {
        unsafe {
            self.buf.set_len(self.len);
        }
    }
}

fn read_to_end<R: Read + ?Sized>(r: &mut R, buf: &mut Vec<u8>) -> Result<usize> {
    read_to_end_with_reservation(r, buf, |_| 32)
}

fn read_to_end_with_reservation<R, F>(r: &mut R, buf: &mut Vec<u8>, mut reservation_size: F) -> Result<usize> where R: Read + ?Sized, F: FnMut(&R) -> usize, {
    let start_len = buf.len();
    let mut g = Guard { len: buf.len(), buf };
    let ret;
    loop {
        //
        if g.len == g.buf.len() {
            // 进行扩容
            g.buf.reserve(reservation_size(r));
            // 获得扩容后的缓冲区最大容量
            let capacity = g.buf.capacity();
            unsafe {
                // 设置缓冲区容量
                g.buf.set_len(capacity);
                // 初始化缓冲区新扩容的内存
                r.initializer().initialize(&mut g.buf[g.len..]);
            }
        }
        // 将数据读取至扩容部分
        match r.read(&mut g.buf[g.len..]).log_warning().unwrap() {
            Ok(0) => {
                ret = Ok(g.len - start_len);
                break;
            }
            Ok(n) => g.len += n,
            Err(ref e) if e.kind() == ErrorKind::Interrupted => {}
            Err(e) => {
                ret = Err(e);
                break;
            }
        }
    }

    ret
}


#[derive(Debug)]
pub struct Initializer(bool);

impl Initializer {
    /// Returns a new `Initializer` which will zero out buffers.
    #[inline]
    pub fn zeroing() -> Initializer {
        Initializer(true)
    }

    /// Returns a new `Initializer` which will not zero out buffers.
    ///
    /// # Safety
    ///
    /// This may only be called by `Read`ers which guarantee that they will not
    /// read from buffers passed to `Read` methods, and that the return value of
    /// the method accurately reflects the number of bytes that have been
    /// written to the head of the buffer.
    #[inline]
    pub unsafe fn nop() -> Initializer {
        Initializer(false)
    }

    /// 表示缓冲区是否应该被初始化
    #[inline]
    pub fn should_initialize(&self) -> bool {
        self.0
    }

    /// 如果需要的话会始化缓冲区(根据缓冲区长度将值设为0)
    #[inline]
    pub fn initialize(&self, buf: &mut [u8]) {
        if self.should_initialize() {
            unsafe { ptr::write_bytes(buf.as_mut_ptr(), 0, buf.len()) }
        }
    }
}
