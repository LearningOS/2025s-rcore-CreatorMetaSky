//! Process management syscalls

use crate::{
    config::PAGE_SIZE,
    mm::{translated_byte_buffer, MapPermission, VirtAddr},
    task::{
        self, change_program_brk, current_user_token, exit_current_and_run_next,
        suspend_current_and_run_next,
    },
    timer::get_time_us,
};

#[repr(C)]
#[derive(Debug)]
pub struct TimeVal {
    pub sec: usize,
    pub usec: usize,
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

pub fn sys_get_time(ts: *mut TimeVal, _tz: usize) -> isize {
    trace!("kernel: sys_get_time");

    let len = core::mem::size_of::<TimeVal>();
    let mut translated_ts_buffers =
        translated_byte_buffer(current_user_token(), ts as *const u8, len);

    let us = get_time_us();
    let time_val = TimeVal {
        sec: us / 1_000_000,
        usec: us % 1_000_000,
    };
    let time_val_bytes =
        unsafe { core::slice::from_raw_parts(&time_val as *const TimeVal as *const u8, len) };

    let mut offset = 0;
    for buf in translated_ts_buffers.iter_mut() {
        let copy_len = buf.len();
        buf.copy_from_slice(&time_val_bytes[offset..offset + copy_len]);
        offset += copy_len;
    }
    0
}

pub fn sys_trace(trace_request: usize, id: usize, data: usize) -> isize {
    trace!("kernel: sys_trace");
    let token = current_user_token();

    let addr = id as *const u8;
    let len = 1;

    let buffers = translated_byte_buffer(token, addr, len);

    if buffers.is_empty() {
        return -1;
    }

    match trace_request {
        0 => {
            // read
            let buffer = &buffers[0];
            buffer[0] as isize
        }
        1 => {
            // write
            let mut buffers = translated_byte_buffer(token, addr, len);
            if let Some(buffer) = buffers.iter_mut().next() {
                buffer[0] = data as u8;
                0
            } else {
                -1
            }
        }
        _ => -1,
    }
}

pub fn sys_mmap(start: usize, len: usize, prot: usize) -> isize {
    trace!("kernel: sys_mmap");

    if start & (PAGE_SIZE - 1) != 0 || prot & !0x7 != 0 || prot & 0x7 == 0 {
        return -1;
    }

    if len == 0 {
        return 0;
    }

    let start_va = VirtAddr::from(start);
    let end_va = VirtAddr::from(start + len);
    let mut map_perm = MapPermission::from_bits((prot << 1) as u8).unwrap();
    map_perm |= MapPermission::U;

    let res = task::mmap(start_va, end_va, map_perm);
    if res {
        0
    } else {
        -1
    }
}

// YOUR JOB: Implement munmap.
pub fn sys_munmap(start: usize, len: usize) -> isize {
    trace!("kernel: sys_munmap");

    // 检查起始地址是否页对齐
    if start & (PAGE_SIZE - 1) != 0 {
        return -1;
    }

    // 计算需要取消映射的页数（向上取整）
    let len = if len == 0 {
        0
    } else {
        (len - 1) / PAGE_SIZE + 1
    };

    // 执行取消映射
    let end = start + len * PAGE_SIZE;

    // 使用公共接口取消映射
    let result = task::munmap(start, end);

    if result {
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
