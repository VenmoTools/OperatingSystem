use core::sync::atomic::{AtomicBool, Ordering, spin_loop_hint};
use core::cell::UnsafeCell;
use core::marker::Sync;
use core::ops::{Drop, Deref, DerefMut};
use core::option::Option::{self, None, Some};
use core::default::Default;
use core::fmt;

// 在编译时已经确定大小
// 所有参数都必须实现了Sized绑定
// 特殊语法是？Sized表示如果绑定不适合使用将会移除
/// 使用原子操作完成的自旋锁
/// # Example
/// ```
/// use system::mutex::Mutex;
///
/// let lock = Mutex::new(1);
/// let value = lock.lock();
/// value += 1;
///
/// ```
pub struct Mutex<T: ?Sized> {
    /// 标志位，表明是否被上锁
    lock: AtomicBool,
    /// 为了保持内部可变性，使用了UnsafeCell
    data: UnsafeCell<T>,
}
/// 调用Lock时返回MutexGuard获取上锁的值
#[derive(Debug)]
pub struct MutexGuard<'a, T: ?Sized + 'a> {
    lock: &'a AtomicBool,
    data: &'a mut T,
}


impl<T> Mutex<T> {
    /// 使用给定值创建原子锁
    pub const fn new(data: T) -> Mutex<T> {
        Mutex {
            lock: AtomicBool::new(false),
            data: UnsafeCell::new(data),
        }
    }
    /// 使用into_runner来获取锁住的内部数据
    pub fn into_runner(self) -> T {
        // 注意data的变量名一定要跟Mutex中的成员名一致
        // 这里只获取Mutex.data
        let Mutex { data, .. } = self;
        data.into_inner()
    }
}

unsafe impl<T: ?Sized + Sync> Sync for Mutex<T> {}

unsafe impl<T: ?Sized + Sync> Send for Mutex<T> {}

impl<T: Sized + fmt::Debug> fmt::Debug for Mutex<T>
{
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result
    {
        match self.try_lock()
            {
                Some(guard) => write!(f, "Mutex {{ data: ")
                    .and_then(|()| (&*guard).fmt(f))
                    .and_then(|()| write!(f, "}}")),
                None => write!(f, "Mutex {{ <locked> }}"),
            }
    }
}

impl<T: ?Sized> Mutex<T> {
    /// 尝试获取锁，如果没有获取到则阻塞运行
    fn obtain_lock(&self) {
        // 尝试获得锁
        while self.lock.compare_and_swap(false, true, Ordering::Acquire) {
            // 循环判断是否已经解锁如果没有解锁
            while self.lock.load(Ordering::Relaxed) {
                // 向处理器发出信号，表明现在处于自旋状态
                spin_loop_hint();
            }
        }
        // 跳出循环后表明获得锁
    }
    /// 上锁，在当前作用域过后释放锁
    pub fn lock(&self) -> MutexGuard<T> {
        self.obtain_lock();
        MutexGuard {
            lock: &self.lock,
            data: unsafe { &mut *self.data.get() },
        }
    }

    /// 强制释放锁
    pub unsafe fn force_unlock(&self) {
        self.lock.store(false, Ordering::Release)
    }

    /// 尝试获取锁，如果获取到返回Some(T)否则返回None，如果没有获取到锁不会陷入自旋状态
    pub fn try_lock(&self) -> Option<MutexGuard<T>> {
        if self.lock.compare_and_swap(false, true, Ordering::Acquire) == false {
            Some(MutexGuard {
                lock: &self.lock,
                data: unsafe { &mut *self.data.get() },
            })
        } else {
            None
        }
    }
}

impl<T: Sized + Default> Default for Mutex<T> {
    fn default() -> Mutex<T> {
        Mutex::new(Default::default())
    }
}

impl<'a, T: Sized> DerefMut for MutexGuard<'a, T> {
    fn deref_mut(&mut self) -> &mut T { &mut *self.data }
}

impl<'a, T: Sized> Deref for MutexGuard<'a, T> {
    type Target = T;

    fn deref(&self) -> &T {
        &*self.data
    }
}

impl<'a, T: ?Sized> Drop for MutexGuard<'a, T> {
    fn drop(&mut self) {
        self.lock.store(false, Ordering::Release);
    }
}