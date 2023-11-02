//! Process management syscalls
use core::mem::size_of;

use crate::{
    config::MAX_SYSCALL_NUM,
    task::{
        change_program_brk, exit_current_and_run_next, suspend_current_and_run_next, TaskStatus, current_user_token,
        get_current_task_information, change_current_user_space_mmap, remove_current_user_space_mmap
    }, timer::get_time_us,
    mm::translated_byte_buffer,
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
pub fn sys_exit(_exit_code: i32) -> ! {
    trace!("kernel: sys_exit");
    exit_current_and_run_next();
    panic!("Unreachable in sys_exit!");
}

/// current task gives up resources for other tasks
pub fn sys_yield() -> isize {
    trace!("kernel: sys_yield");
    suspend_current_and_run_next();
    0
}

/// YOUR JOB: get time with second and microsecond
/// HINT: You might reimplement it with virtual memory management.
/// HINT: What if [`TimeVal`] is splitted by two pages ?
pub fn sys_get_time(_ts: *mut TimeVal, _tz: usize) -> isize {
    trace!("kernel: sys_get_time");
    let us = get_time_us();
    let __ts = TimeVal {
        sec: us / 1_000_000,
        usec: us % 1_000_000,
    };
    let mut ptr = translated_byte_buffer(current_user_token(), _ts as *const u8, size_of::<TimeVal>());
    // Medthod 1
    let mut bytes_written = 0;
    for buffer in &mut ptr {
        let buffer_len = buffer.len();
        trace!("kernel: buffer_len {}", buffer_len);
        if buffer_len >= bytes_written + size_of::<TimeVal>() {
            let data_bytes: &[u8] = unsafe {
                core::slice::from_raw_parts(
                    &__ts as *const TimeVal as *const u8, 
                    size_of::<TimeVal>(),    
                )
            };

            buffer[bytes_written..bytes_written + data_bytes.len()].copy_from_slice(data_bytes);
            bytes_written += data_bytes.len();
        } else {
            panic!("Should not reachable here!");
        }
    }
    0
}

/// YOUR JOB: Finish sys_task_info to pass testcases
/// HINT: You might reimplement it with virtual memory management.
/// HINT: What if [`TaskInfo`] is splitted by two pages ?
pub fn sys_task_info(_ti: *mut TaskInfo) -> isize {
    trace!("kernel: sys_task_info NOT IMPLEMENTED YET!");
    let tuple_result = get_current_task_information();
    let task_information = TaskInfo {
        status: tuple_result.0,
        time: tuple_result.1,
        syscall_times: tuple_result.2,
    };
    let mut ptr = translated_byte_buffer(current_user_token(), 
        _ti as *const u8, size_of::<TaskInfo>());
    let bytes_written = 0;
    for buffer in &mut ptr {
        let buffer_len = buffer.len();
        if buffer_len >= bytes_written + size_of::<TaskInfo>() {
            let data_bytes: &[u8] = unsafe {
                core::slice::from_raw_parts(
                    &task_information as *const TaskInfo as *const u8,
                    size_of::<TaskInfo>(),
                )
            };

            buffer[bytes_written..bytes_written + data_bytes.len()].copy_from_slice(data_bytes);
        } else {
            panic!("Should not reachable here!");
        }
    }
    0
}

// YOUR JOB: Implement mmap.
pub fn sys_mmap(_start: usize, _len: usize, _port: usize) -> isize {
    trace!("kernel: sys_mmap");
    if let Some(_resutl) = change_current_user_space_mmap(_start, _len, _port) {
        0
    } else {
        -1
    }
}

// YOUR JOB: Implement munmap.
pub fn sys_munmap(_start: usize, _len: usize) -> isize {
    trace!("kernel: sys_munmap");
    if let Some(_result) = remove_current_user_space_mmap(_start, _len) {
        0
    } else {
        -1
    }
}
/// change data segment size
pub fn sys_sbrk(size: i32) -> isize {
    trace!("kernel: sys_sbrk");
    if let Some(old_brk) = change_program_brk(size) {
        old_brk as isize
    } else {
        -1
    }
}
