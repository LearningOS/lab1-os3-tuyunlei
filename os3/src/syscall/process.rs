use crate::config::MAX_SYSCALL_NUM;
use crate::task::{exit_current_and_run_next, get_current_task_info, suspend_current_and_run_next, TaskInfo, TaskStatus};
use crate::timer::{get_time, get_time_ms, get_time_us};

#[repr(C)]
pub struct TimeVal {
    pub sec: usize,
    pub usec: usize,
}

pub fn sys_yield() -> isize {
    suspend_current_and_run_next();
    0
}

pub fn sys_exit(exit_code: i32) -> ! {
    info!("[kernel] Application exited with code {}", exit_code);
    exit_current_and_run_next();
    panic!("Unreachable in sys_exit!");
}

pub fn sys_get_time(ts: *mut TimeVal, _tz: usize) -> isize {
    let us = get_time_us();
    unsafe {
        *ts = TimeVal {
            sec: us / 1_000_000,
            usec: us % 1_000_000,
        };
    }
    0
}

pub fn sys_task_info(task_info: *mut TaskInfo) -> isize {
    unsafe {
        get_current_task_info(task_info);
    }
    0
}
