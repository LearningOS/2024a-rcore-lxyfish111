//! Process management syscalls
use crate::{
    config::MAX_SYSCALL_NUM,
    task::{exit_current_and_run_next, 
        suspend_current_and_run_next, 
        TaskStatus, 
        get_current_task_status, 
        get_current_task_syscall_count,
        get_current_task_time,
    },
    timer::{get_time_us, get_time_ms},
};

#[repr(C)]
#[derive(Debug)]
pub struct TimeVal {
    pub sec: usize,
    pub usec: usize,
}

/// Task information
#[allow(dead_code)]
pub struct TaskInfo {
    /// Task status in it's life cycle
    status: TaskStatus,
    /// The numbers of syscall called by task
    syscall_times: [u32; MAX_SYSCALL_NUM],
    /// Total running time of task
    time: usize,
}

/// task exits and submit an exit code
pub fn sys_exit(exit_code: i32) -> ! {
    trace!("[kernel] Application exited with code {}", exit_code);
    exit_current_and_run_next();
    panic!("Unreachable in sys_exit!");
}

/// current task gives up resources for other tasks
pub fn sys_yield() -> isize {
    trace!("kernel: sys_yield");
    suspend_current_and_run_next();
    0
}

/// get time with second and microsecond
pub fn sys_get_time(ts: *mut TimeVal, _tz: usize) -> isize {
    trace!("kernel: sys_get_time");
    let us = get_time_us();
    unsafe {
        *ts = TimeVal {
            sec: us / 1_000_000,
            usec: us % 1_000_000,
        };
    }
    0
}

/// YOUR JOB: Finish sys_task_info to pass testcases
pub fn sys_task_info(_ti: *mut TaskInfo) -> isize {
    trace!("kernel: sys_task_info");
    // let status = get_current_task_status();
    // let tm = get_current_task_time();

    // let current_tm = get_time();
    // let mut tm_ret = if status.clone() == TaskStatus::Exited {
    //     println!("kernel: sys_task_info TaskStatus::Exited: {}", tm.clone());
    //     tm
    // } else {
    //     println!("kernel: sys_task_info else: {}, current_tm: {}, tm:{}", (current_tm.clone() - tm.clone()), current_tm.clone(), tm.clone());
    //     current_tm - tm
    // };
    

    // tm_ret = tm_ret * 1000 / CLOCK_FREQ;
    // println!("tm_ret:{}", tm_ret);

    unsafe {
        *_ti = TaskInfo {
            status: get_current_task_status(),
            syscall_times: get_current_task_syscall_count(),
            time: (get_time_ms() - get_current_task_time())
        }
    };

    0
}
