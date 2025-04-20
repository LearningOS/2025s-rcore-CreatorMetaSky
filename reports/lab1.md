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

# 简单总结

通过在 TaskManager 记录系统调用的次数，实现系统调用次数的更新和读取。

每个 app 对应一个 task，在当前 task 执行系统调用的时候，就会触发更新系统调用次数的函数，再根据当前 task 的编号，记录当前 task 调用了哪个系统调用，更新调用次数。

需要获取系统调用次数的时候，找到当前是在哪个 task 下，再根据系统调用编号找到调用次数即可。


本次编程作业学到的知识点：

- rcore
    - 了解和熟悉如何构造调试环境
    - link_app.S 是通过 build.rs 自动生成的
- rust 语言相关
    - default trait 用法
    - 所有权机制与集合类数据结构的使用

# 简答作业

## 1. 进入用户态访问内核态指令寄存器报错

版本信息：[rustsbi] RustSBI version 0.3.0-alpha.2, adapting to RISC-V SBI v1.0.0

### ch2b_bad_address

这个错误是因为程序尝试访问地址 0x0，这是一个无效的内存地址。在操作系统中，地址 0x0 通常被保留，不允许用户程序访问。0x0通常这是为了捕获空指针引用的错误。当程序尝试写入地址 0x0 时，特权级处理会检测到异常，打印报错日志

```sh
[kernel] PageFault in application, bad addr = 0x0, bad instruction = 0x804003a4, kernel killed it.
```

### ch2b_bad_instruction

由于特权级别限制，sret (Supervisor Return) 是一个特权指令，它只能在 S 模式（Supervisor Mode，即内核态）下执行。在 RISC-V 架构中，用户程序运行在 U 模式（User Mode，即用户态），没有权限执行这类特权指令，所以会报错：

```sh
[kernel] IllegalInstruction in application, kernel killed it.
```

### ch2b_bad_register

sstatus 也是 RISC-V 架构中的一个特权寄存器，属于 S 模式（Supervisor Mode，即内核态）。用户程序运行在 U 模式（User Mode，即用户态）下，没有权限直接访问这类特权寄存器，所有会报错：

```sh
[kernel] IllegalInstruction in application, kernel killed it.
```

## 深入理解 trap.S 中两个函数 __alltraps 和 __restore 的作用

### 1. L40：刚进入 __restore 时，sp 代表了什么值。请指出 __restore 的两种使用情景

当进入 __restore 函数时， sp 寄存器指向的是内核栈上分配的 TrapContext 结构体的起始地址。这个结构体包含了从用户态陷入内核态时保存的所有寄存器值，包括用户程序的栈指针、程序计数器和处理器状态等信息

- __restore 的两种使用场景
    - 从内核态返回用户态（特权级下降）
    - 从异常处理返回到内核态（特权级不变）

### 2. L43-L48：这几行汇编代码特殊处理了哪些寄存器？这些寄存器的的值对于进入用户态有何意义？请分别解释

处理了三个关键寄存器：

- sstatus 寄存器 (通过 t0 加载) - 从内核态切换用户态时，需要恢复之前保存的 sstatus 值，以确保用户程序在正确的状态下运行
- sepc 寄存器 (通过 t1 加载) - 在从内核态切换用户态时，sepc 被设置为用户程序应该继续执行的地址
- sscratch 寄存器 (通过 t2 加载) - 当从内核态切换到用户态时，需要恢复用户栈指针，以便用户程序能够正确访问其栈

这三个寄存器的正确设置是特权级切换的核心，确保了用户程序能够在正确的位置、正确的特权级别下，使用正确的栈继续执行。

### 3. L50-L56：为何跳过了 x2 和 x4？

1. x2 (sp) 寄存器 ：
   x2 是栈指针寄存器。它不是直接从 TrapContext 中恢复的，而是通过特殊的方式处理。在代码的后面部分，通过 csrrw sp, sscratch, sp 指令来恢复用户态的栈指针。这是因为栈指针的切换需要特别小心，以确保正确地从内核栈切换回用户栈。
2. x4 (tp) 寄存器 ：
   tp 是线程指针寄存器。在 RISC-V 架构中，tp 寄存器通常用于线程本地存储（Thread Local Storage），但在这个简单的操作系统中，用户程序不使用这个功能，所以不需要保存和恢复它。
这种选择性地恢复寄存器的方式可以提高效率，避免不必要的操作。特别是对于有特殊用途的寄存器（如栈指针），需要使用专门的方法来处理，以确保特权级切换的正确性和安全性。

### 4. L60：该指令之后，sp 和 sscratch 中的值分别有什么意义？

执行 `csrrw sp, sscratch, sp` 指令后：

- sp 寄存器：包含了用户态的栈指针值。这个值之前存储在 sscratch 寄存器中，现在被加载到 sp 中，使处理器可以使用用户栈。
- sscratch 寄存器：包含了内核态的栈指针值。这个值之前在 sp 寄存器中，现在被保存到 sscratch 中。

### 5. __restore：中发生状态切换在哪一条指令？为何该指令执行之后会进入用户态？

在 61 行 sret 指令

执行该指令后会进入用户态的原因如下：

- 特权级别切换：sret 指令会根据 sstatus 寄存器中来决定返回到哪个特权级别
- 程序计数器恢复：sret 指令会将 sepc 寄存器的值加载到 pc 寄存器中，使程序从用户程序的指定位置继续执行
- 状态恢复：sret 指令还会恢复 sstatus 寄存器中的其他状态位

### 6. L13：该指令之后，sp 和 sscratch 中的值分别有什么意义？

执行 `csrrw sp, sscratch, sp` 指令后：

- sp 寄存器：包含了内核栈的地址。在进入 trap 处理前，sscratch 寄存器中存储的是内核栈地址，执行这条指令后，这个值被加载到 sp 中，使处理器可以使用内核栈
- sscratch 寄存器：包含了用户态的栈指针值。这个值之前在 sp 寄存器中，现在被保存到 sscratch 中，以便后续恢复

这条指令实现了栈指针的交换，是特权级切换的第一步。当从用户态陷入内核态时，需要立即切换到内核栈，这样内核代码才能安全地执行，不会破坏用户栈。
同时，将用户栈指针保存在 sscratch 中，为后续恢复用户态做准备。这种设计使得特权级切换既高效又安全，确保了内核和用户程序各自使用正确的栈。

### 7. 从 U 态进入 S 态是哪一条指令发生的？

一般情况下是通过执行 ecall 指令触发的

## 荣誉准则

1. 在完成本次实验的过程（含此前学习的过程）中，我曾分别与 以下各位 就（与本次实验相关的）以下方面做过交流，还在代码中对应的位置以注释形式记录了具体的交流对象及内容：

无

2. 此外，我也参考了 以下资料 ，还在代码中对应的位置以注释形式记录了具体的参考来源及内容：

- [rust syscall](https://github.com/rust-lang/libc/blob/main/src/unix/linux_like/linux/gnu/b64/riscv64/mod.rs)
- [RISCV 手册](http://riscvbook.com/chinese/RISC-V-Reader-Chinese-v2p1.pdf)
- [RISC-V Linux syscall table](https://jborza.com/post/2021-05-11-riscv-linux-syscalls/)
- [RISC-V通用寄存器及函数调用规范](https://lgl88911.github.io/2021/02/15/RISC-V%E9%80%9A%E7%94%A8%E5%AF%84%E5%AD%98%E5%99%A8%E5%8F%8A%E5%87%BD%E6%95%B0%E8%B0%83%E7%94%A8%E8%A7%84%E8%8C%83/)
- [rustsbi-qemu](https://github.com/rustsbi/rustsbi-qemu)
- [rCore-Tutorial-Book](https://rcore-os.cn/rCore-Tutorial-Book-v3)

3. 我独立完成了本次实验除以上方面之外的所有工作，包括代码与文档。 我清楚地知道，从以上方面获得的信息在一定程度上降低了实验难度，可能会影响起评分。
4. 我从未使用过他人的代码，不管是原封不动地复制，还是经过了某些等价转换。 我未曾也不会向他人（含此后各届同学）复制或公开我的实验代码，我有义务妥善保管好它们。 我提交至本实验的评测系统的代码，均无意于破坏或妨碍任何计算机系统的正常运转。 我清楚地知道，以上情况均为本课程纪律所禁止，若违反，对应的实验成绩将按“-100”分计。