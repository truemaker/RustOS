static mut TICKS: u64 = 0;

pub fn timer_tick() {
    unsafe {
        TICKS += 1;
    }
}

pub fn sleep(duration: u64) {
    unsafe {
        let end = TICKS + duration;
        while TICKS < end {}
    }
}
