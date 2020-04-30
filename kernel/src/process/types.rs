use core::sync::atomic::{AtomicUsize, Ordering};
use system::atomic_type;

atomic_type!(ProcessId,usize,AtomicProcessId,AtomicUsize);
atomic_type!(SchemeId,usize,AtomicSchemeId,AtomicUsize);
atomic_type!(FileDescriptor,usize,AtomicFileDescriptor,AtomicUsize);

pub const MAX_PROCESS: usize = (isize::max_value() as usize) - 1;
pub const MAX_FILES: usize = 65_536;

