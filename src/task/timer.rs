use conquer_once::spin::OnceCell;
use crossbeam_queue::ArrayQueue;
use futures_util::task::AtomicWaker;

static TICK_QUEUE: OnceCell<ArrayQueue<i64>> = OnceCell::uninit();

use crate::println;

pub(crate) fn timer_tick() {
    if let Ok(queue) = TICK_QUEUE.try_get() {
        if queue.push(1).is_err() {
            println!("WARNING: tick queue full; dropping tick");
        } else {
            WAKER.wake(); // new
        }
    } else {
        println!("WARNING: tick queue uninitialized");
    }
}

pub struct TickStream {
    _private: (),
}

impl TickStream {
    pub fn new() -> Self {
        TICK_QUEUE
            .try_init_once(|| ArrayQueue::new(100))
            .expect("TickStream::new should only be called once");
        TickStream { _private: () }
    }
}

impl Default for TickStream {
    fn default() -> Self {
        Self::new()
    }
}

use core::{
    pin::Pin,
    task::{Context, Poll},
};
use futures_util::stream::Stream;

impl Stream for TickStream {
    type Item = i64;

    fn poll_next(self: Pin<&mut Self>, cx: &mut Context) -> Poll<Option<i64>> {
        let queue = TICK_QUEUE.try_get().expect("tick queue not initialized");

        // fast path
        if let Ok(tick) = queue.pop() {
            return Poll::Ready(Some(tick));
        }

        WAKER.register(cx.waker());
        match queue.pop() {
            Ok(tick) => {
                WAKER.take();
                Poll::Ready(Some(tick))
            }
            Err(crossbeam_queue::PopError) => Poll::Pending,
        }
    }
}

static WAKER: AtomicWaker = AtomicWaker::new();
static mut TICKS: i64 = 0;
use futures_util::stream::StreamExt;

fn handle_tick(count: i64) {
    unsafe {
        TICKS += count;
    }
}

pub async fn timer_handle() {
    let mut ticks = TickStream::new();

    while let Some(tick) = ticks.next().await {
        handle_tick(tick);
    }
}
