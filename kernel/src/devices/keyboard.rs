use bitflags::_core::pin::Pin;
use bitflags::_core::task::{Context, Poll};
use crossbeam_queue::ArrayQueue;
use futures_util::stream::Stream;
use futures_util::StreamExt;
use futures_util::task::AtomicWaker;
use pc_keyboard::{DecodedKey, HandleControl, Keyboard, ScancodeSet1};
use pc_keyboard::layouts::Us104Key;
use spin::{Once, RwLock, RwLockReadGuard, RwLockWriteGuard};

static SCAN_CODE_WAKER: AtomicWaker = AtomicWaker::new();

static SCAN_CODE_QUEUE: Once<RwLock<ArrayQueue<u8>>> = Once::new();


fn init_contexts() -> RwLock<ArrayQueue<u8>> {
    RwLock::new(ArrayQueue::new(4096))
}


pub fn scan_queue() -> RwLockReadGuard<'static, ArrayQueue<u8>> {
    SCAN_CODE_QUEUE.call_once(init_contexts).read()
}

pub fn scan_queue_mut() -> RwLockWriteGuard<'static, ArrayQueue<u8>> {
    SCAN_CODE_QUEUE.call_once(init_contexts).write()
}

pub fn init() {}

pub fn add_scan_code(code: u8) {
    let lock = scan_queue_mut();
    if let Err(_) = lock.push(code) {
        println!("scan code queue full dropping keyboard input")
    } else {
        SCAN_CODE_WAKER.wake();
    }
}

pub struct ScanCodeStream;

impl ScanCodeStream {
    pub fn new() -> Self {
        Self
    }
}

impl Stream for ScanCodeStream {
    type Item = u8;

    fn poll_next(self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<Option<Self::Item>> {
        let lock = scan_queue_mut();
        if let Ok(code) = lock.pop() {
            return Poll::Ready(Some(code));
        }
        SCAN_CODE_WAKER.register(&cx.waker());
        match lock.pop() {
            Ok(code) => Poll::Ready(Some(code)),
            Err(crossbeam_queue::PopError) => Poll::Pending
        }
    }
}

pub async fn print_scan_code() {
    let mut codes = ScanCodeStream::new();
    let mut keyboard = Keyboard::new(Us104Key, ScancodeSet1, HandleControl::Ignore);
    while let Some(code) = codes.next().await {
        if let Ok(Some(event)) = keyboard.add_byte(code) {
            if let Some(key) = keyboard.process_keyevent(event) {
                match key {
                    DecodedKey::Unicode(character) => print!("{}", character),
                    DecodedKey::RawKey(key) => print!("{:?}", key),
                }
            }
        }
    }
}