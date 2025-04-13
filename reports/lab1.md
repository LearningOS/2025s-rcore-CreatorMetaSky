# 编程作业

## 处理 process.rs 中系统调用函数的逻辑

```rs
pub fn sys_trace(trace_request: usize, id: usize, data: usize) -> isize {
    trace!("kernel: sys_trace");
    match trace_request {
        0 => {
            let cur_task_addr = id as *const u8;
            unsafe { *cur_task_addr as isize }
        }
        1 => {
            let cur_task_addr = id as *mut u8;
            unsafe {
                *cur_task_addr = data as u8;
            }
            0
        }
        2 => get_syscall_count(id) as isize,
        _ => -1,
    }
}
```

## 处理 get_syscall_count 获取系统调用次数的逻辑

```rs
// os/src/task/mod.rs
/// Get syscall count
pub fn get_syscall_count(syscall_id: usize) -> usize {
}
```

## 修改测试用例，确保代码执行

- 调试 user/Makefile，查看筛选 App 的逻辑

```Makefile
show-build-var:
	@echo "-------"
	@echo "TEST=$(TEST), BASE=$(BASE), CHAPTER=$(CHAPTER)"
	@echo "TESTS=$(shell seq $(BASE) $(TEST))"
	@echo "APPS=$(APPS)"
	@echo "ELFS=$(ELFS)"
	@echo "-------"
```

- 将 os/Makefile 中的 BASE 从 1 改为 0，就可以测试到 user/src/bin/ch3_trace.rs 中的代码了

## 实现 syscall_count 存储相关逻辑

```rs
// os/src/task/task.rs
#[derive(Default)]
pub struct SysCallInfo {
    pub count_map: BTreeMap<usize, usize>,
}
```

```rs
pub struct TaskManagerInner {
    ...
    // syscall information
    syscall_infos: [SysCallInfo; MAX_APP_NUM], // 添加系统调用的信息存储
}
```

```rs
// os/src/task/mod.rs
fn update_syscall_count(&self, syscall_id: usize) {
    let mut inner = self.inner.exclusive_access();
    let cur_task_id = inner.current_task;
    let syscall_info = &mut inner.syscall_infos[cur_task_id];

    *syscall_info.count_map.entry(syscall_id).or_insert(0) += 1;
}

fn get_syscall_count(&self, syscall_id: usize) -> usize {
    let inner = self.inner.exclusive_access();
    let cur_task_id = inner.current_task;
    let syscall_info = &inner.syscall_infos[cur_task_id];
    *syscall_info.count_map.get(&syscall_id).unwrap_or(&0)
}
```

```rs
// os/src/task/mod.rs
/// Update syscall count
pub fn update_syscall_count(syscall_id: usize) {
    TASK_MANAGER.update_syscall_count(syscall_id);
}

/// Get syscall count
pub fn get_syscall_count(syscall_id: usize) -> usize {
    TASK_MANAGER.get_syscall_count(syscall_id)
}
```

## 找到更新系统调用次数的关键位置

```rs
// os/src/syscall/mod.rs
pub fn syscall(syscall_id: usize, args: [usize; 3]) -> isize {
    update_syscall_count(syscall_id); // 更新系统调用次数
    ...
}
```

## 自测跑通

```sh
cd os
make run
```

```sh
[rustsbi] RustSBI version 0.3.0-alpha.2, adapting to RISC-V SBI v1.0.0
.______       __    __      _______.___________.  _______..______   __
|   _  \     |  |  |  |    /       |           | /       ||   _  \ |  |
|  |_)  |    |  |  |  |   |   (----`---|  |----`|   (----`|  |_)  ||  |
|      /     |  |  |  |    \   \       |  |      \   \    |   _  < |  |
|  |\  \----.|  `--'  |.----)   |      |  |  .----)   |   |  |_)  ||  |
| _| `._____| \______/ |_______/       |__|  |_______/    |______/ |__|
[rustsbi] Implementation     : RustSBI-QEMU Version 0.2.0-alpha.2
[rustsbi] Platform Name      : riscv-virtio,qemu
[rustsbi] Platform SMP       : 1
[rustsbi] Platform Memory    : 0x80000000..0x88000000
[rustsbi] Boot HART          : 0
[rustsbi] Device Tree Region : 0x87000000..0x87000ef2
[rustsbi] Firmware Address   : 0x80000000
[rustsbi] Supervisor Address : 0x80200000
[rustsbi] pmp01: 0x00000000..0x80000000 (-wr)
[rustsbi] pmp02: 0x80000000..0x80200000 (---)
[rustsbi] pmp03: 0x80200000..0x88000000 (xwr)
[rustsbi] pmp04: 0x88000000..0x00000000 (-wr)
[kernel] Hello, world!
get_time OK! 5
current time_msec = 6
time_msec = 106 after sleeping 100 ticks, delta = 100ms!
Test sleep1 passed!
string from task trace test

Test trace OK!
Test sleep OK!
[kernel] Panicked at src/task/mod.rs:139 All applications completed!
```

## 简单总结

通过在 TaskManager 记录系统调用的次数，实现系统调用次数的更新和读取。

每个 app 对应一个 task，在当前 task 执行系统调用的时候，就会触发更新系统调用次数的函数，再根据当前 task 的编号，记录当前 task 调用了哪个系统调用，更新调用次数。

需要获取系统调用次数的时候，找到当前是在哪个 task 下，再根据系统调用编号找到调用次数即可。


本次编程作业学到的知识点：

- rcore
    - 了解和熟悉如何构造调试环境
    - link_app.S 是通过 build.rs 自动生成的
- rust 语言相关
    - default trait 用法
    - 所有权机制结合集合类数据结构的使用

## 简答作业
