mod fs;
mod process;

pub const SYSCALL_WRITE: usize = 64;
pub const SYSCALL_EXIT: usize = 93;
pub const SYSCALL_YIELD: usize = 124;
pub const SYSCALL_GET_TIME: usize = 169;
pub const SYSCALL_GET_TASK_INFO: usize = 410;

use fs::*;
use process::*;
use crate::task::{decrease_syscall_times, increase_syscall_times};

pub fn syscall(syscall_id: usize, args: [usize; 3]) -> isize {
    increase_syscall_times(syscall_id);
    match syscall_id {
        SYSCALL_WRITE => sys_write(args[0], args[1] as *const u8, args[2]),
        SYSCALL_EXIT => sys_exit(args[0] as i32),
        SYSCALL_YIELD => sys_yield(),
        SYSCALL_GET_TIME => sys_get_time(args[0] as *mut _, args[1]),
        SYSCALL_GET_TASK_INFO => sys_task_info(args[0] as *mut _),
        _ => {
            decrease_syscall_times(syscall_id);
            panic!("Unsupported syscall_id: {}", syscall_id)
        },
    }
}
