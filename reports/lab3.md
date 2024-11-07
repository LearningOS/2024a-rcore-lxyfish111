实现的功能:
实现sys_spawn函数并merge之前完成的system call。

问答题：
答：p1 的步长为 255，p2 的步长为 250。在 p2 执行一个时间片后，它的步长减少到 249，而 p1 的步长仍然是 255。理论上，下一个时间片应该轮到 p1 执行，但实际上，由于 p1 和 p2 的步长差距只有 6，p2 很快就会再次获得CPU，这可能导致 p1 长时间得不到执行。
当所有进程的优先级都 >= 2 时，意味着它们的步长值都相对较小。在这种情况下，步长的最大值（STRIDE_MAX）和最小值（STRIDE_MIN）之间的差距不会超过 BigStride 的一半。这是因为，如果 STRIDE_MAX - STRIDE_MIN 大于 BigStride 的一半，那么至少有一个进程的步长会小于 STRIDE_MIN，这与所有进程优先级都 >= 2 的假设矛盾。
在 Rust 中，我们需要为 Stride 结构体实现 PartialOrd trait，以便在 BinaryHeap 中正确比较 Stride 值。由于 Stride 可能溢出，我们需要特别设计比较逻辑。以下是 partial_cmp 函数的实现：

use core::cmp::Ordering;

struct Stride(u8); // 使用 u8 来存储 stride

impl PartialOrd for Stride {
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        let diff_self = (u16::from(self.0)).wrapping_sub(u16::from(Stride::max_value().0));
        let diff_other = (u16::from(other.0)).wrapping_sub(u16::from(Stride::max_value().0));
        diff_self.partial_cmp(&diff_other)
    }
}

impl PartialEq for Stride {
    fn eq(&self, other: &Self) -> bool {
        false // 根据题目要求，两个 Stride 永远不会相等
    }
}

impl Stride {
    fn max_value() -> Stride {
        Stride(u8::MAX) // STRIDE_MAX 的值
    }
}

在完成本次实验的过程（含此前学习的过程）中，我曾分别与 以下各位 就（与本次实验相关的）以下方面做过交流，还在代码中对应的位置以注释形式记录了具体的交流对象及内容：
群和网络资料

此外，我也参考了 以下资料 ，还在代码中对应的位置以注释形式记录了具体的参考来源及内容：
百度搜索结果

3. 我独立完成了本次实验除以上方面之外的所有工作，包括代码与文档。 我清楚地知道，从以上方面获得的信息在一定程度上降低了实验难度，可能会影响起评分。

4. 我从未使用过他人的代码，不管是原封不动地复制，还是经过了某些等价转换。 我未曾也不会向他人（含此后各届同学）复制或公开我的实验代码，我有义务妥善保管好它们。 我提交至本实验的评测系统的代码，均无意于破坏或妨碍任何计算机系统的正常运转。 我清楚地知道，以上情况均为本课程纪律所禁止，若违反，对应的实验成绩将按“-100”分计。

