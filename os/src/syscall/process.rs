//! Process management syscalls

use crate::{
    config::PAGE_SIZE,
    mm::{translated_byte_buffer, MapPermission},
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

/// YOUR JOB: get time with second and microsecond
/// HINT: You might reimplement it with virtual memory management.
/// HINT: What if [`TimeVal`] is splitted by two pages ?
pub fn sys_get_time(ts: *mut TimeVal, _tz: usize) -> isize {
    trace!("kernel: sys_get_time");
    let len = core::mem::size_of::<TimeVal>();

    let us = get_time_us();
    let time_val = TimeVal {
        sec: us / 1_000_000,
        usec: us % 1_000_000,
    };
    let time_val_bytes =
        unsafe { core::slice::from_raw_parts(&time_val as *const TimeVal as *const u8, len) };

    let mut buffers = translated_byte_buffer(current_user_token(), ts as *const u8, len); // 用户空间虚拟地址转换为内核空间物理地址

    // 数据拷贝
    let mut offset = 0;
    for buffer in buffers.iter_mut() {
        let copy_len = buffer.len();
        buffer.copy_from_slice(&time_val_bytes[offset..offset + copy_len]);
        offset += copy_len;
    }
    0
}

/// TODO: Finish sys_trace to pass testcases
/// HINT: You might reimplement it with virtual memory management.
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

// YOUR JOB: Implement mmap.
pub fn sys_mmap(start: usize, len: usize, port: usize) -> isize {
    trace!("kernel: sys_mmap");

    // 检查起始地址是否页对齐
    if start & (PAGE_SIZE - 1) != 0 {
        return -1;
    }

    // 检查 port 参数是否有效
    if port & !0x7 != 0 {
        return -1;
    }

    // 检查是否至少有一个权限位
    if port & 0x7 == 0 {
        return -1;
    }

    // 计算需要映射的页数（向上取整）
    let len = if len == 0 {
        0
    } else {
        (len - 1) / PAGE_SIZE + 1
    };

    // 转换权限
    let mut permission = MapPermission::U; // 用户态可访问
    if (port & 1) != 0 {
        permission |= MapPermission::R;
    } // 可读
    if (port & 2) != 0 {
        permission |= MapPermission::W;
    } // 可写
    if (port & 4) != 0 {
        permission |= MapPermission::X;
    } // 可执行

    // 执行映射
    let end = start + len * PAGE_SIZE;

    // 创建一个新的内存映射区域
    let result = task::mmap(start, end, permission);

    if result {
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
