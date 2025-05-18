# 编程作业

## 实现 spawn

- 这里 spawn 实现起来会更简单，不需要像 fork 那样复制一份地址空间，直接创建新的 TCB 产生一个新的地址空间
- 用 spawn 创建好 task，添加到调度队列里即可

针对直接启动新进程的场景，spawn 更合适，避免了笔不要的开销

## stride 调度算法实现

- 为每个 TCB 添加一个 stride (运行长度)、pass (权重) 属性字段
- 在 Processor 中每次调度的时候，借助 TaskManager 中查找 stride 最小的 TCB 的方法，找到需要运行的 task
- 把当前需要运行的 task 的 stride 属性值加上 pass
- 定义 BIG_STRIDE，在 sys_set_priority 系统调用的时候，用 BIG_STRIDE / priority 生成 pass 赋值给当前 TCB

# 问答作业

## 实际情况是轮到 p1 执行吗？为什么？

不轮到 p1 执行还是 p2 执行

- 由于使用8位无符号整型存储，p2 执行完后 stride 是260，会溢出变成 260 % 256 = 4
- 此时 p1.stride = 255，p2.stride = 4
- 由于 p2 stride 还是比 p1 小，所以还是 p2 执行

## STRIDE_MAX – STRIDE_MIN <= BigStride / 2

想象两个小朋友A和B在玩轮流吃糖的游戏：

1. 初始设置：
- 每人有一个"吃糖次数计数器"（stride）
- 规定：A每次可以加10（pass=10），B每次可以加5（pass=5）
- 计数器超过100就从头开始（BigStride=100）

2. 游戏过程：
```
初始状态：
A的计数器：90
B的计数器：80

第1轮：
选择计数器小的B先吃糖 → B的计数器变成80+5=85

第2轮：
A:90, B:85 → 还是B吃糖 → B=85+5=90

第3轮： 
A:90, B:90 → 随便选一个，假设选A → A=90+10=100（超过100变成0）

第4轮：
A:0, B:90 → 选A → A=0+10=10

第5轮：
A:10, B:90 → 选A → A=10+10=20
```
3. 结论
- 虽然B的pass值小，但A也会得到吃糖机会
- 两人的计数器差距最大时是90-0=90，确实小于BigStride(100)/2=50
- 长期来看，B吃糖次数会是A的2倍（因为10:5=2:1）

每次选择stride最小的进程执行，执行后该进程的stride += pass，由于pass ≤ BigStride/2，单个进程的stride增量有限，因为每次增加的步长(pass)不超过BigStride的一半，所以任何两个进程的stride差距不会超过BigStride的一半。这保证了调度的公平性，防止某个进程被"饿死"

## 考虑溢出的情况下，Stride 比较器

```rs
use core::cmp::Ordering;

struct Stride(u64);

impl PartialOrd for Stride {
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        let threshold = 255 / 2;
        let a = self.0;
        let b = other.0;

        if (a.wrapping_sub(b) as u8) <= threshold {
            Some(a.cmp(&b))
        } else {
            Some(b.cmp(&a))
        }
    }
}

impl PartialEq for Stride {
    fn eq(&self, other: &Self) -> bool {
        false
    }
}
```

# 荣誉准则

1. 在完成本次实验的过程（含此前学习的过程）中，我曾分别与 以下各位 就（与本次实验相关的）以下方面做过交流，还在代码中对应的位置以注释形式记录了具体的交流对象及内容：

- 部分参考 DeepSeek V3 的回答

2. 此外，我也参考了 以下资料 ，还在代码中对应的位置以注释形式记录了具体的参考来源及内容：

- [rCore-Tutorial-Book](https://rcore-os.cn/rCore-Tutorial-Book-v3/chapter5/index.html)
- [rCore-Tutorial-Guide](https://learningos.cn/rCore-Tutorial-Guide-2025S/chapter5/index.html)
- [Slides](https://learningos.cn/os-lectures/lec7/p4-labs.html)

3. 我独立完成了本次实验除以上方面之外的所有工作，包括代码与文档。 我清楚地知道，从以上方面获得的信息在一定程度上降低了实验难度，可能会影响起评分。
4. 我从未使用过他人的代码，不管是原封不动地复制，还是经过了某些等价转换。 我未曾也不会向他人（含此后各届同学）复制或公开我的实验代码，我有义务妥善保管好它们。 我提交至本实验的评测系统的代码，均无意于破坏或妨碍任何计算机系统的正常运转。 我清楚地知道，以上情况均为本课程纪律所禁止，若违反，对应的实验成绩将按“-100”分计。