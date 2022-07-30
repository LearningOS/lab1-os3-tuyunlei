use crate::config::{MAX_APP_NUM, MAX_SYSCALL_NUM};
use crate::loader::{get_num_app, init_app_ctx};
use crate::sync::UPSafeCell;
use crate::task::context::TaskContext;
use crate::task::switch::__switch;
use crate::timer::{get_time, get_time_ms};

pub mod context;
pub mod switch;

pub struct TaskManager {
    inner: UPSafeCell<TaskManagerInner>,
}

pub struct TaskInfo {
    pub status: TaskStatus,
    pub syscall_times: [u32; MAX_SYSCALL_NUM],
    pub time: usize,
}

#[derive(Copy, Clone)]
/// task control block structure
pub struct TaskControlBlock {
    pub task_status: TaskStatus,
    pub task_ctx: TaskContext,
    // LAB1: Add whatever you need about the Task.
    pub syscall_times: [u32; MAX_SYSCALL_NUM],
    pub start_time_ms: usize,
}

#[derive(Copy, Clone, PartialEq, Debug)]
/// task status: UnInit, Ready, Running, Exited
pub enum TaskStatus {
    UnInit,
    Ready,
    Running,
    Exited,
}

struct TaskManagerInner {
    num_app: usize,
    tasks: [TaskControlBlock; MAX_APP_NUM],
    current_task: usize,
}

impl TaskManager {
    fn run_first_task(&self) -> ! {
        let mut inner = self.inner.exclusive_access();
        let task0 = &mut inner.tasks[0];
        task0.task_status = TaskStatus::Running;
        task0.start_time_ms = get_time_ms();
        let next_task_ctx_ptr = &task0.task_ctx as *const TaskContext;
        drop(inner);
        let mut _unused = TaskContext::default();
        unsafe {
            __switch(&mut _unused as *mut TaskContext, next_task_ctx_ptr);
        }
        panic!("Unreachable");
    }

    fn mark_current_exited(&self) {
        let mut inner = self.inner.exclusive_access();
        let current = inner.current_task;
        inner.tasks[current].task_status = TaskStatus::Exited;
    }

    fn mark_current_suspended(&self) {
        let mut inner = self.inner.exclusive_access();
        let current = inner.current_task;
        inner.tasks[current].task_status = TaskStatus::Ready;
    }

    fn run_next_task(&self) {
        if let Some(next) = self.find_next_task() {
            let mut inner = self.inner.exclusive_access();
            let current = inner.current_task;
            if inner.tasks[next].start_time_ms == 0 {
                inner.tasks[next].start_time_ms = get_time_ms();
            }
            inner.tasks[next].task_status = TaskStatus::Running;
            inner.current_task = next;
            let current_task_ctx_ptr = &mut inner.tasks[current].task_ctx as *mut TaskContext;
            let next_task_ctx_ptr = &inner.tasks[next].task_ctx as *const TaskContext;
            drop(inner);
            unsafe {
                __switch(current_task_ctx_ptr, next_task_ctx_ptr);
            }
        } else {
            panic!("All applications completed!");
        }
    }

    fn find_next_task(&self) -> Option<usize> {
        let inner = self.inner.exclusive_access();
        let current = inner.current_task;
        (current + 1..current + inner.num_app + 1)
            .map(|id| id % inner.num_app)
            .find(|id| inner.tasks[*id].task_status == TaskStatus::Ready)
    }

    fn increase_syscall_times(&self, id: usize) {
        let mut inner = self.inner.exclusive_access();
        let current = inner.current_task;
        inner.tasks[current].syscall_times[id] += 1;
    }

    fn decrease_syscall_times(&self, id: usize) {
        let mut inner = self.inner.exclusive_access();
        let current = inner.current_task;
        inner.tasks[current].syscall_times[id] -= 1;
    }

    unsafe fn get_current_task_info(&self, info_ptr: *mut TaskInfo) {
        let inner = self.inner.exclusive_access();
        let task = &inner.tasks[inner.current_task];
        (*info_ptr).status = task.task_status;
        (*info_ptr).syscall_times = task.syscall_times;
        (*info_ptr).time = get_time_ms() - task.start_time_ms;
    }
}

static TASK_MANAGER: TaskManager = TaskManager {
    inner: unsafe {
        UPSafeCell::new(TaskManagerInner {
            num_app: 0,
            tasks: [TaskControlBlock {
                task_status: TaskStatus::UnInit,
                task_ctx: TaskContext::default(),
                syscall_times: [0; MAX_SYSCALL_NUM],
                start_time_ms: 0,
            }; MAX_APP_NUM],
            current_task: 0,
        })
    },
};

fn init_task_manager() {
    let num_app = get_num_app();
    let mut inner = TASK_MANAGER.inner.exclusive_access();
    inner.num_app = num_app;
    inner.current_task = 0;
    for (i, t) in inner.tasks.iter_mut().enumerate().take(num_app) {
        t.task_ctx = TaskContext::goto_restore(init_app_ctx(i));
        t.task_status = TaskStatus::Ready;
    }
}

#[inline]
pub fn run_first_task() -> ! {
    init_task_manager();
    TASK_MANAGER.run_first_task()
}

#[inline]
pub fn suspend_current_and_run_next() {
    TASK_MANAGER.mark_current_suspended();
    TASK_MANAGER.run_next_task();
}

#[inline]
pub fn exit_current_and_run_next() {
    TASK_MANAGER.mark_current_exited();
    TASK_MANAGER.run_next_task();
}

#[inline]
pub fn increase_syscall_times(syscall_id: usize) {
    TASK_MANAGER.increase_syscall_times(syscall_id)
}

#[inline]
pub fn decrease_syscall_times(syscall_id: usize) {
    TASK_MANAGER.decrease_syscall_times(syscall_id)
}

#[inline]
pub unsafe fn get_current_task_info(info_ptr: *mut TaskInfo) {
    TASK_MANAGER.get_current_task_info(info_ptr)
}
