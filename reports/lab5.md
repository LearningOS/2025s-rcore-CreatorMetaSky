# 编程作业

## TaskControlBlockInner 中添加数据属性

- 互斥锁
    - hold_mutex_list - 持有的互斥锁
    - wait_mutex - 等待的互斥锁
- 信号量的记录
    - hold_semaphore_list - 持有的信号量
    - wait_semaphore - 等待的信号量

在 syscall 调用 sys_mutex_lock、sys_mutex_unlock、sys_semaphore_create、sys_semaphore_up、sys_semaphore_down 的时候填充数据

## 添加死锁检测的方法

mutex_deadlock_detect 和 semaphore_deadlock_detect 实现了基于银行家算法的死锁检测机制，主要步骤如下：

1. **初始化**：
   - 获取当前任务数量(`task_num`)和互斥锁数量(`mutex_num`)
   - 初始化三个关键矩阵：
     - `available`: 表示每个互斥锁是否可用
     - `allocation`: 记录每个任务当前持有的互斥锁
     - `need`: 记录每个任务正在等待的互斥锁

2. **状态收集**：
   - 遍历所有任务，根据每个任务中持有的互斥锁(`hold_mutex_list`) 和 等待的互斥锁(`wait_mutex`)来填充上面定义的向量结构

3. **检测**：
    - 检测每个任务没有依赖的资源或者需要的资源是可用的，表明这个任务可以完成
    - 释放当前任务的资源
    - 标记该任务为完成
    - 最后检测所有任务都可以完成，则不存在死锁

# 简答作业

## 在我们的多线程实现中，当主线程 (即 0 号线程) 退出时，视为整个进程退出， 此时需要结束该进程管理的所有线程并回收其资源。 - 需要回收的资源有哪些？ - 其他线程的 TaskControlBlock 可能在哪些位置被引用，分别是否需要回收，为什么？

## 对比以下两种 Mutex 中的实现，二者有什么区别？这些区别可能会导致什么问题？


# 荣誉准则

1. 在完成本次实验的过程（含此前学习的过程）中，我曾分别与 以下各位 就（与本次实验相关的）以下方面做过交流，还在代码中对应的位置以注释形式记录了具体的交流对象及内容：

- 部分参考 DeepSeek V3 的回答

2. 此外，我也参考了 以下资料 ，还在代码中对应的位置以注释形式记录了具体的参考来源及内容：

https://learningos.cn/rCore-Tutorial-Guide-2025S/chapter8/1thread-kernel.html
- [rCore-Tutorial-Book](https://rcore-os.cn/rCore-Tutorial-Book-v3/chapter6/index.html)
- [rCore-Tutorial-Guide](https://learningos.cn/rCore-Tutorial-Guide-2025S/chapter6/index.html)
- [Slides](https://learningos.cn/os-lectures/lec9/p4-fs-lab.html)

3. 我独立完成了本次实验除以上方面之外的所有工作，包括代码与文档。 我清楚地知道，从以上方面获得的信息在一定程度上降低了实验难度，可能会影响起评分。
4. 我从未使用过他人的代码，不管是原封不动地复制，还是经过了某些等价转换。 我未曾也不会向他人（含此后各届同学）复制或公开我的实验代码，我有义务妥善保管好它们。 我提交至本实验的评测系统的代码，均无意于破坏或妨碍任何计算机系统的正常运转。 我清楚地知道，以上情况均为本课程纪律所禁止，若违反，对应的实验成绩将按“-100”分计。