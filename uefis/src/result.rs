use alloc::string::{String, ToString};
use core::convert::TryInto;
use core::fmt;
use core::fmt::Formatter;
use uefi::{Completion, Status};

const ERROR_BIT: usize = 1 << (core::mem::size_of::<usize>() * 8 - 1);


// 表示UEFI服务使用的Result
pub type UefiResult<T> = uefi::Result<T>;
// 表示UEFI应用程序使用的Result
pub type Result<T> = core::result::Result<T, Error>;


#[derive(Debug, Clone)]
struct Custom {
    /// UEFI应用执行错误类型
    kind: ErrorKind,
    /// UEFI应用执行错误说明
    err_msg: String,
}

#[derive(Debug, Clone)]
struct Efi {
    /// UEFI服务执行结果状态码
    status: Status,
    /// UEFI状态对应说明
    err_msg: Option<String>,
}

// 用于表示UEFI应用异常类型
#[derive(Clone, Copy, Debug, PartialOrd, PartialEq)]
pub enum ErrorKind {
    /// 文件未找到
    NotFound,
    /// 在读取文件中读取到了EOF
    UnexpectedEof,
    /// 无效的文件
    InvalidFile,
    /// 无效的文件模式
    InvalidFileMode,
    /// UEFI错误，包含错误码
    UefiErrorCode,
    /// 终止
    Interrupted,
    /// 未写入任何数据
    WriteZero,
}

impl ErrorKind {
    pub fn as_str(&self) -> &'static str {
        match *self {
            ErrorKind::NotFound => "entity not found",
            ErrorKind::UnexpectedEof => "unexpected end of file",
            ErrorKind::InvalidFileMode => "invalid file mode",
            ErrorKind::UefiErrorCode => "uefi error status code",
            ErrorKind::InvalidFile => "specif file is invalid",
            ErrorKind::Interrupted => "interrupted",
            ErrorKind::WriteZero => "no data written"
        }
    }
}


impl Efi {
    pub fn as_str(&self) -> &'static str {
        match self.status.0 {
            0 => "The operation completed successfully.",
            1 => "The string contained characters that could not be rendered and were skipped.",
            2 => "The handle was closed, but the file was not deleted.",
            3 => "The handle was closed, but the data to the file was not flushed properly.",
            4 => "The resulting buffer was too small, and the data was truncated.",
            5 => "The data has not been updated within the timeframe set by local policy.",
            6 => "The resulting buffer contains UEFI-compliant file system.",
            7 => "The operation will be processed across a system reset.",
            ERROR_BIT | 1 => "The image failed to load.",
            ERROR_BIT | 2 => "A parameter was incorrect.",
            ERROR_BIT | 3 => "The operation is not supported.",
            ERROR_BIT | 4 => "The buffer was not the proper size for the request.The buffer is not large enough to hold the requested data.",
            ERROR_BIT | 5 => "The required buffer size is returned in the appropriate parameter.",
            ERROR_BIT | 6 => "There is no data pending upon return.",
            ERROR_BIT | 7 => "The physical device reported an error while attempting the operation.",
            ERROR_BIT | 8 => "The device cannot be written to.",
            ERROR_BIT | 9 => "A resource has run out.",
            ERROR_BIT | 10 => "An inconstency was detected on the file system.",
            ERROR_BIT | 11 => "There is no more space on the file system.",
            ERROR_BIT | 12 => "The device does not contain any medium to perform the operation.",
            ERROR_BIT | 13 => "The medium in the device has changed since the last access.",
            ERROR_BIT | 14 => "The item was not found.",
            ERROR_BIT | 15 => "Access was denied.",
            ERROR_BIT | 16 => "The server was not found or did not respond to the request.",
            ERROR_BIT | 17 => "A mapping to a device does not exist.",
            ERROR_BIT | 18 => "The timeout time expired.",
            ERROR_BIT | 19 => "The protocol has not been started.",
            ERROR_BIT | 20 => "The protocol has already been started.",
            ERROR_BIT | 21 => "The operation was aborted.",
            ERROR_BIT | 22 => "An ICMP error occurred during the network operation.",
            ERROR_BIT | 23 => "A TFTP error occurred during the network operation.",
            ERROR_BIT | 24 => "A protocol error occurred during the network operation. The function encountered an internal version that was",
            ERROR_BIT | 25 => "incompatible with a version requested by the caller.",
            ERROR_BIT | 26 => "The function was not performed due to a security violation.",
            ERROR_BIT | 27 => "A CRC error was detected.",
            ERROR_BIT | 28 => "Beginning or end of media was reached",
            ERROR_BIT | 31 => "The end of the file was reached.",
            ERROR_BIT | 32 => "The language specified was invalid.The security status of the data is unknown or compromised and",
            ERROR_BIT | 33 => "the data must be updated or replaced to restore a valid security status.",
            ERROR_BIT | 34 => "There is an address conflict address allocation",
            ERROR_BIT | 35 => "A HTTP error occurred during the network operation.",
            _ => "Unknown status"
        }
    }
}

// 用于表示异常的种类，分为UEFI执行错误和UEFI应用程序错误
#[derive(Debug)]
enum Repr {
    /// UEFI服务错误
    Uefi(Efi),
    /// UEFI应用错误
    Custom(Custom),
}

#[derive(Debug)]
pub struct Error {
    repr: Repr,
}

impl Error {
    /// 根据给定的错误类型创建UEFI应用异常
    pub fn new(kind: ErrorKind, msg: &str) -> Error {
        Error { repr: Repr::Custom(Custom { kind, err_msg: msg.to_string() }) }
    }
    /// 根据给定的错误类型创建UEFI应用异常 支持String 主要方便使用format!
    pub fn with_string(kind: ErrorKind, msg: String) -> Error {
        Error { repr: Repr::Custom(Custom { kind, err_msg: msg }) }
    }
    /// 根据传递的状态码创建UEFI服务错误
    pub fn from_uefi_status(status: Status, msg: Option<&str>) -> Error {
        Error {
            repr: Repr::Uefi(Efi {
                status,
                err_msg: match msg {
                    Some(msg) => Some(msg.to_string()),
                    None => None,
                },
            })
        }
    }

    /// 提供错误类型的判断
    pub fn kind(&self) -> ErrorKind {
        match self.repr {
            Repr::Uefi(ref efi) => ErrorKind::UefiErrorCode,
            Repr::Custom(ref cu) => cu.kind,
        }
    }
}

impl fmt::Display for Error {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self.repr {
            Repr::Custom(ref cu) => {
                write!(f, "{}: {}", cu.kind.as_str(), cu.err_msg)
            }
            Repr::Uefi(ref efi) => {
                match efi.err_msg {
                    None => write!(f, "got uefi status `{}` info: {}", efi.status.0, efi.as_str()),
                    Some(ref other) => write!(f, "got uefi status `{}` info: {}", efi.status.0, other),
                }
            }
        }
    }
}

pub trait AppResultExt<T> {
    fn unwrap(self) -> T;
}

impl<T> AppResultExt<T> for Result<T> {
    fn unwrap(self) -> T {
        match self {
            core::result::Result::Ok(d) => d,
            core::result::Result::Err(e) => panic!("{:?}", e)
        }
    }
}

pub fn ok<T>(t: T) -> UefiResult<Result<T>> {
    Ok(Completion::from(core::result::Result::Ok(t)))
}

pub fn err<T>(e: Error) -> UefiResult<Result<T>> {
    Ok(Completion::from(core::result::Result::Err(e)))
}