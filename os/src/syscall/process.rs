//! Process management syscalls
use alloc::sync::Arc;

use crate::{
    config::{BIG_STRIDE, PAGE_SIZE},
    loader::get_app_data_by_name,
    mm::{translated_byte_buffer, translated_refmut, translated_str, MapPermission, VirtAddr},
    task::{
        add_task, current_task, current_user_token, exit_current_and_run_next,
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
pub fn sys_exit(exit_code: i32) -> ! {
    trace!("kernel:pid[{}] sys_exit", current_task().unwrap().pid.0);
    exit_current_and_run_next(exit_code);
    panic!("Unreachable in sys_exit!");
}

/// current task gives up resources for other tasks
pub fn sys_yield() -> isize {
    trace!("kernel:pid[{}] sys_yield", current_task().unwrap().pid.0);
    suspend_current_and_run_next();
    0
}

pub fn sys_getpid() -> isize {
    trace!("kernel: sys_getpid pid:{}", current_task().unwrap().pid.0);
    current_task().unwrap().pid.0 as isize
}

pub fn sys_fork() -> isize {
    trace!("kernel:pid[{}] sys_fork", current_task().unwrap().pid.0);
    let current_task = current_task().unwrap();
    let new_task = current_task.fork();
    let new_pid = new_task.pid.0;
    // modify trap context of new_task, because it returns immediately after switching
    let trap_cx = new_task.inner_exclusive_access().get_trap_cx();
    // we do not have to move to next instruction since we have done it before
    // for child process, fork returns 0
    trap_cx.x[10] = 0; // update ra - child process return 0
    add_task(new_task); // add new task to scheduler - 将生成的子进程通过 add_task 加入到任务管理器中
    new_pid as isize // current parent process return child pid
}

pub fn sys_exec(path: *const u8) -> isize {
    trace!("kernel:pid[{}] sys_exec", current_task().unwrap().pid.0);
    let token = current_user_token();

    // 在内核中具体获得字符串的话就需要手动查页表
    // 从内核地址空间之外的某个应用的用户态地址空间中拿到一个字符串
    let path = translated_str(token, path);

    // 找到对应的 ELF 格式的数据
    if let Some(data) = get_app_data_by_name(path.as_str()) {
        let task = current_task().unwrap();
        task.exec(data);
        0
    } else {
        -1
    }
}

// 如果当前的进程不存在一个进程 ID 为 pid（pid==-1 或 pid > 0）的子进程，则返回 -1；
// 如果存在一个进程 ID 为 pid 的僵尸子进程，则正常回收并返回子进程的 pid，并更新系统调用的退出码参数为 exit_code
// 这里还有一个 -2 的返回值，它的含义是子进程还没退出，通知用户库 user_lib （是实际发出系统调用的地方），这样用户库看到是 -2 后，就进一步调用 sys_yield 系统调用（第46行），让当前父进程进入等待状态
/// If there is not a child process whose pid is same as given, return -1.
/// Else if there is a child process but it is still running, return -2.
pub fn sys_waitpid(pid: isize, exit_code_ptr: *mut i32) -> isize {
    trace!(
        "kernel::pid[{}] sys_waitpid [{}]",
        current_task().unwrap().pid.0,
        pid
    );
    let task = current_task().unwrap();
    // find a child process

    // ---- access current PCB exclusively
    let mut inner = task.inner_exclusive_access();
    if !inner
        .children
        .iter()
        .any(|p| pid == -1 || pid as usize == p.getpid())
    {
        return -1;
        // ---- release current PCB
    }

    // 判断符合要求的子进程中是否有僵尸进程，如果有的话还需要同时找出它在当前进程控制块子进程向量中的下标。如果找不到的话直接返回 -2
    let pair = inner.children.iter().enumerate().find(|(_, p)| {
        // ++++ temporarily access child PCB exclusively
        p.inner_exclusive_access().is_zombie() && (pid == -1 || pid as usize == p.getpid())
        // ++++ release child PCB
    });
    if let Some((idx, _)) = pair {
        let child = inner.children.remove(idx);

        // 确认这是对于该子进程控制块的唯一一次强引用，即它不会出现在某个进程的子进程向量中，更不会出现在处理器监控器或者任务管理器中。当它所在的代码块结束，这次引用变量的生命周期结束，将导致该子进程进程控制块的引用计数变为 0 ，彻底回收掉它占用的所有资源
        // 内核栈和它的 PID 还有它的应用地址空间存放页表的那些物理页帧等等
        // confirm that child will be deallocated after being removed from children list
        assert_eq!(Arc::strong_count(&child), 1);

        // 得到子进程的 PID 并会在最终返回
        let found_pid = child.getpid();

        // 得到了子进程的退出码
        // ++++ temporarily access child PCB exclusively
        let exit_code = child.inner_exclusive_access().exit_code;
        // ++++ release child PCB

        // 写入到当前进程的应用地址空间中
        *translated_refmut(inner.memory_set.token(), exit_code_ptr) = exit_code;
        found_pid as isize
    } else {
        -2
    }
    // ---- release current PCB automatically
}

/// YOUR JOB: get time with second and microsecond
/// HINT: You might reimplement it with virtual memory management.
/// HINT: What if [`TimeVal`] is splitted by two pages ?
pub fn sys_get_time(ts: *mut TimeVal, _tz: usize) -> isize {
    let len = core::mem::size_of::<TimeVal>();
    let mut translated_ts_buffers =
        translated_byte_buffer(current_user_token(), ts as *const u8, len);

    if translated_ts_buffers.is_empty() {
        return -1;
    }

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

/// YOUR JOB: Implement mmap.
pub fn sys_mmap(start: usize, len: usize, prot: usize) -> isize {
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

    if let Some(task) = current_task() {
        if task.mmap(start_va, end_va, map_perm) {
            return 0;
        } else {
            return -1;
        }
    }

    -1
}

/// YOUR JOB: Implement munmap.
pub fn sys_munmap(start: usize, len: usize) -> isize {
    if start % PAGE_SIZE != 0 {
        return -1;
    }

    if len == 0 {
        return 0;
    }

    let start_va = VirtAddr::from(start);
    let end_va = VirtAddr::from(start + len);

    if let Some(task) = current_task() {
        if task.munmap(start_va, end_va) {
            return 0;
        } else {
            return -1;
        }
    }

    -1
}

/// change data segment size
pub fn sys_sbrk(size: i32) -> isize {
    trace!("kernel:pid[{}] sys_sbrk", current_task().unwrap().pid.0);
    if let Some(old_brk) = current_task().unwrap().change_program_brk(size) {
        old_brk as isize
    } else {
        -1
    }
}

/// YOUR JOB: Implement spawn.
/// HINT: fork + exec =/= spawn
pub fn sys_spawn(path: *const u8) -> isize {
    if let Some(current_task) = current_task() {
        if let Some(spawn_task) = current_task.spawn(path) {
            let pid = spawn_task.getpid();
            add_task(spawn_task);
            return pid as isize;
        }
    }
    -1
}

// YOUR JOB: Set task priority.
pub fn sys_set_priority(prio: isize) -> isize {
    if prio < 2 {
        -1
    } else {
        current_task().unwrap().set_pass(BIG_STRIDE / prio as usize);
        prio
    }
}
