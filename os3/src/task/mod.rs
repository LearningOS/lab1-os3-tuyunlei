use lazy_static::lazy_static;

use crate::config::MAX_APP_NUM;
use crate::loader::{get_num_app, init_app_ctx};
use crate::sync::UPSafeCell;
use crate::task::context::TaskContext;
use crate::task::switch::__switch;

pub mod context;
pub mod switch;

pub struct TaskManager {
    num_app: usize,
    inner: UPSafeCell<TaskManagerInner>,
}

#[derive(Copy, Clone)]
/// task control block structure
pub struct TaskControlBlock {
    pub task_status: TaskStatus,
    pub task_ctx: TaskContext,
    // LAB1: Add whatever you need about the Task.
}

#[derive(Copy, Clone, PartialEq)]
/// task status: UnInit, Ready, Running, Exited
pub enum TaskStatus {
    UnInit,
    Ready,
    Running,
    Exited,
}

struct TaskManagerInner {
    tasks: [TaskControlBlock; MAX_APP_NUM],
    current_task: usize,
}

impl TaskManager {
    fn run_first_task(&self) -> ! {
        let mut inner = self.inner.exclusive_access();
        let task0 = &mut inner.tasks[0];
        task0.task_status = TaskStatus::Running;
        let next_task_ctx_ptr = &task0.task_ctx as *const TaskContext;
        drop(inner);
        let mut _unused = TaskContext::zero_init();
        unsafe {
            __switch(&mut _unused as *mut TaskContext, next_task_ctx_ptr);
        }
        panic!("unreachable in run_first_task!");
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
        (current + 1..current + self.num_app + 1)
            .map(|id| id % self.num_app)
            .find(|id| inner.tasks[*id].task_status == TaskStatus::Ready)
    }
}

lazy_static! {
    pub static ref TASK_MANAGER: TaskManager = {
        let num_app = get_num_app();
        let mut tasks = [TaskControlBlock {
            task_ctx: TaskContext::zero_init(),
            task_status: TaskStatus::UnInit,
        }; MAX_APP_NUM];
        for (i, t) in tasks.iter_mut().enumerate().take(num_app) {
            t.task_ctx = TaskContext::goto_restore(init_app_ctx(i));
            t.task_status = TaskStatus::Ready;
        }
        TaskManager {
            num_app,
            inner: unsafe {
                UPSafeCell::new(TaskManagerInner {
                    tasks,
                    current_task: 0,
                })
            },
        }
    };
}

pub fn run_first_task() -> ! {
    TASK_MANAGER.run_first_task()
}

pub fn suspend_current_and_run_next() {
    TASK_MANAGER.mark_current_suspended();
    TASK_MANAGER.run_next_task();
}

pub fn exit_current_and_run_next() {
    TASK_MANAGER.mark_current_exited();
    TASK_MANAGER.run_next_task();
}
