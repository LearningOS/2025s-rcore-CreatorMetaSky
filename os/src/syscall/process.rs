//! Process management syscalls

use crate::{
    config::PAGE_SIZE,
    mm::{translated_byte_buffer, MapPermission, PageTable, VirtAddr},
    task::{
        self, change_program_brk, current_user_token, exit_current_and_run_next, get_syscall_count,
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
    trace!("try again");

    let pagetable = PageTable::from_token(current_user_token());
    let vaddr = VirtAddr::from(id);
    let vpn = VirtAddr::from(id).floor();

    match trace_request {
        0 => match pagetable.translate(vpn) {
            Some(pte) if pte.is_valid() && pte.readable() && pte.user() => {
                let ppn = pte.ppn();
                let offset = vaddr.page_offset();
                let page_ptr = ppn.get_bytes_array().as_ptr();
                let byte = unsafe { *page_ptr.add(offset) };
                byte as isize
            }
            _ => -1,
        },
        1 => match pagetable.translate(vpn) {
            Some(pte) if pte.is_valid() && pte.writable() && pte.user() => {
                let ppn = pte.ppn();
                let offset = vaddr.page_offset();
                let page_ptr = ppn.get_bytes_array().as_mut_ptr();
                unsafe {
                    *page_ptr.add(offset) = data as u8;
                }
                0
            }
            _ => -1,
        },
        2 => get_syscall_count(id) as isize,
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

pub fn sys_munmap(start: usize, len: usize) -> isize {
    trace!("kernel: sys_munmap"); // todo: - the trace not display ?

    if start % PAGE_SIZE != 0 {
        return -1;
    }

    if len == 0 {
        return 0;
    }

    let start_va = VirtAddr::from(start);
    let end_va = VirtAddr::from(start + len);

    let result = task::munmap(start_va, end_va);

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
