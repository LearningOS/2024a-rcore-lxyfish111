//! Task management implementation
//!
//! Everything about task management, like starting and switching tasks is
//! implemented here.
//!
//! A single global instance of [`TaskManager`] called `TASK_MANAGER` controls
//! all the tasks in the whole operating system.
//!
//! A single global instance of [`Processor`] called `PROCESSOR` monitors running
//! task(s) for each core.
//!
//! A single global instance of `PID_ALLOCATOR` allocates pid for user apps.
//!
//! Be careful when you see `__switch` ASM function in `switch.S`. Control flow around this function
//! might not be what you expect.
mod context;
mod id;
mod manager;
mod processor;
mod switch;
#[allow(clippy::module_inception)]
#[allow(rustdoc::private_intra_doc_links)]
mod task;

use crate::{fs::{open_file, OpenFlags}, config::MAX_SYSCALL_NUM, mm::{self, MapPermission}};
use alloc::sync::Arc;
pub use context::TaskContext;
use lazy_static::*;
pub use manager::{fetch_task, TaskManager};
use switch::__switch;
pub use task::{TaskControlBlock, TaskStatus};

pub use id::{kstack_alloc, pid_alloc, KernelStack, PidHandle};
pub use manager::add_task;
pub use processor::{
    current_task, current_trap_cx, current_user_token, run_tasks, schedule, take_current_task,
    Processor,
};
/// Suspend the current 'Running' task and run the next task in task list.
pub fn suspend_current_and_run_next() {
    // There must be an application running.
    let task = take_current_task().unwrap();

    // ---- access current TCB exclusively
    let mut task_inner = task.inner_exclusive_access();
    let task_cx_ptr = &mut task_inner.task_cx as *mut TaskContext;
    // Change status to Ready
    task_inner.task_status = TaskStatus::Ready;

    drop(task_inner);
    // ---- release current PCB

    // push back to ready queue.
    add_task(task);
    // jump to scheduling cycle
    schedule(task_cx_ptr);
}

/// pid of usertests app in make run TEST=1
pub const IDLE_PID: usize = 0;

/// Exit the current 'Running' task and run the next task in task list.
pub fn exit_current_and_run_next(exit_code: i32) {
    // take from Processor
    let task = take_current_task().unwrap();

    let pid = task.getpid();
    if pid == IDLE_PID {
        println!(
            "[kernel] Idle process exit with exit_code {} ...",
            exit_code
        );
        panic!("All applications completed!");
    }

    // **** access current TCB exclusively
    let mut inner = task.inner_exclusive_access();
    // Change status to Zombie
    inner.task_status = TaskStatus::Zombie;
    // Record exit code
    inner.exit_code = exit_code;
    // do not move to its parent but under initproc

    // ++++++ access initproc TCB exclusively
    {
        let mut initproc_inner = INITPROC.inner_exclusive_access();
        for child in inner.children.iter() {
            child.inner_exclusive_access().parent = Some(Arc::downgrade(&INITPROC));
            initproc_inner.children.push(child.clone());
        }
    }
    // ++++++ release parent PCB

    inner.children.clear();
    // deallocate user space
    inner.memory_set.recycle_data_pages();
    // drop file descriptors
    inner.fd_table.clear();
    drop(inner);
    // **** release current PCB
    // drop task manually to maintain rc correctly
    drop(task);
    // we do not have to save task context
    let mut _unused = TaskContext::zero_init();
    schedule(&mut _unused as *mut _);
}

lazy_static! {
    /// Creation of initial process
    ///
    /// the name "initproc" may be changed to any other app name like "usertests",
    /// but we have user_shell, so we don't need to change it.
    pub static ref INITPROC: Arc<TaskControlBlock> = Arc::new({
        let inode = open_file("ch6b_initproc", OpenFlags::RDONLY).unwrap();
        let v = inode.read_all();
        TaskControlBlock::new(v.as_slice())
    });
}

///Add init process to the manager
pub fn add_initproc() {
    add_task(INITPROC.clone());
}

/// get current task
pub fn get_current_task()-> (TaskStatus, [u32; MAX_SYSCALL_NUM],usize){
    let binding = current_task().unwrap();
    let current = binding.inner_exclusive_access();
    let mut syscall_times_clone:[u32;MAX_SYSCALL_NUM]=[0;MAX_SYSCALL_NUM];
    for i in 0..current.syscall_times.len(){
        syscall_times_clone[i] = current.syscall_times[i];
    }
    
    (
        TaskStatus::Running,
        syscall_times_clone,
        current.start_time,
    )
}

/// increase the syscall times
pub fn inc_syscall_times(syscall_id:usize){
    let binding = current_task().unwrap();
    let mut inner = binding.inner_exclusive_access();
    inner.syscall_times[syscall_id] += 1;
}

/// apply memory
pub fn mmap(start: usize, len: usize, port: usize)->isize{
    let binding = current_task().unwrap();
    let mut inner = binding.inner_exclusive_access();

    let start_va=mm::VirtAddr::from(start);
    let end_va=mm::VirtAddr::from(start+len);

    for vpn in mm::VPNRange::new(start_va.floor(),end_va.ceil()){
        if let Some(pte) =  inner.memory_set.translate(vpn){
            if pte.is_valid(){
                return -1;
            }
        }
    }
    let map_permission: mm::MapPermission = MapPermission::from_bits((port as u8) << 1).unwrap() | MapPermission::U;
    inner.memory_set.insert_framed_area(start_va, end_va, map_permission);

    for vpn in mm::VPNRange::new(start_va.floor(),end_va.ceil()){
        match inner.memory_set.translate(vpn) {
            Some(pte)=>{
                if pte.is_valid()==false{
                    return -1;
                }
            }
            None => {
                return -1;
            }
        }
    }
    0
}

/// cancel memory mapping
pub fn munmap(start:usize,len:usize)->isize{
    let binding = current_task().unwrap();
    let mut inner = binding.inner_exclusive_access();

    let start_va=mm::VirtAddr::from(start);
    let end_va=mm::VirtAddr::from(start+len);
    //检查从起始地址到结束地址中是否有未被映射的内存
    for vpn in mm::VPNRange::new(start_va.floor(),end_va.ceil()){
        match inner.memory_set.translate(vpn) {
            Some(pte)=>{
                if pte.is_valid()==false{
                    return -1;
                }
            }
            None => {
                return -1;
            }
        }
    }
    inner.memory_set.delete_frame_area(start_va, end_va);//按照虚拟地址从物理内存中删除页框
    0
}
