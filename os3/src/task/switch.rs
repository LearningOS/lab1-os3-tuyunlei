use crate::task::context::TaskContext;

core::arch::global_asm!(include_str!("switch.S"));

extern "C" {
    pub fn __switch(
        current_task_ctx_ptr: *mut TaskContext,
        next_task_ctx_ptr: *const TaskContext,
    );
}
