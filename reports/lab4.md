# 编程作业

- 在系统调用里需要虚拟地址和物理地址的转换
- 最终都是会用到 vfs.rs 中新增 link、 unlink、stat 方法实现，其他都是这些方法的调用

## 建立硬链接 link 的实现思路 - 为同一个 inode 创建多个目录项

1. 在 DiskInode 中添加 link_num 的属性，标记硬链接的个数
2. 根据文件名找到 inode_id
3. 更新当前目录项的大小并创建新目录项
4. 根据 inode_id 找到 block_id 方便修改 inode
5. 获取到 block_id 增加硬连接次数

## 取消建立硬连接 unlink 实现思路

1. 修改目录inode，查找并删除目录项
    - 查找目标目录项索引
    - 如果找到目录项，执行删除操作
2. 如果没找目录项则直接返回失败
3. 修改目标inode的链接计数
4. 如果是最后一个链接则释放inode
5. 同步块缓存并返回成功

## stat 实现思路

- 根据 Stat 结构体的设计，需要得到 inode_id, mode, link_num 信息
- 通过 read_disk_inode 将信息读取出来

# 问答作业

## 在我们的 easy-fs 中，root inode 起着什么作用？如果 root inode 中的内容损坏了，会发生什么？

1. **root inode的作用**：

- root inode是文件系统的根目录inode，固定编号为0，在文件系统创建时初始化
- 它作为整个文件系统的入口点，所有其他文件和目录都通过它来访问
- 通过 efs.rs 中的 root_inode() 方法可以获取根inode
- 它维护了文件系统的目录结构，通过 find() 方法可以查找子文件/目录

2. **root inode损坏的影响**：

- 如果root inode内容损坏，会导致整个文件系统无法访问，因为无法定位其他文件和目录
- 具体表现包括：
  - 无法通过`find()`查找任何文件
  - 无法执行`ls()`列出目录内容
  - 文件系统操作会返回错误或panic

# 荣誉准则

1. 在完成本次实验的过程（含此前学习的过程）中，我曾分别与 以下各位 就（与本次实验相关的）以下方面做过交流，还在代码中对应的位置以注释形式记录了具体的交流对象及内容：

- 部分参考 DeepSeek V3 的回答

2. 此外，我也参考了 以下资料 ，还在代码中对应的位置以注释形式记录了具体的参考来源及内容：

- [rCore-Tutorial-Book](https://rcore-os.cn/rCore-Tutorial-Book-v3/chapter6/index.html)
- [rCore-Tutorial-Guide](https://learningos.cn/rCore-Tutorial-Guide-2025S/chapter6/index.html)
- [Slides](https://learningos.cn/os-lectures/lec9/p4-fs-lab.html)

3. 我独立完成了本次实验除以上方面之外的所有工作，包括代码与文档。 我清楚地知道，从以上方面获得的信息在一定程度上降低了实验难度，可能会影响起评分。
4. 我从未使用过他人的代码，不管是原封不动地复制，还是经过了某些等价转换。 我未曾也不会向他人（含此后各届同学）复制或公开我的实验代码，我有义务妥善保管好它们。 我提交至本实验的评测系统的代码，均无意于破坏或妨碍任何计算机系统的正常运转。 我清楚地知道，以上情况均为本课程纪律所禁止，若违反，对应的实验成绩将按“-100”分计。