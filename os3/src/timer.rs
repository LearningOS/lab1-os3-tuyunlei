use crate::config::CLOCK_FREQ;
use crate::sbi::set_timer;
use riscv::register::time;

const TICKS_PER_SEC: usize = 100;
const MILLI_PER_SEC: usize = 1_000;
const MICRO_PER_SEC: usize = 1_000_000;

#[inline]
pub fn get_time() -> usize {
    time::read()
}

#[inline]
pub fn get_time_us() -> usize {
    time::read() * 10 / 125
}

#[inline]
pub fn get_time_ms() -> usize {
    time::read() / (CLOCK_FREQ / MILLI_PER_SEC)
}

#[inline]
pub fn set_next_trigger() {
    set_timer(get_time() + CLOCK_FREQ / TICKS_PER_SEC);
}
