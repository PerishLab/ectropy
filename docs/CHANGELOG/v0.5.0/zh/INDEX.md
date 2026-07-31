# Ectropy v0.5.0

## KISS 拒绝匿名模型

KISS 现在是 Ectropy 的顶层原则：简单的代码会显式表达自己的模型。每条具体 law
只命名一个机械可见的投影；`kiss` 本身永远不是 finding 或 suppression 标签。

新的 `combination` law 会拒绝组合第四个 decision atom 的布尔表达式。相连的
`&&` 与 `||` 会跨越括号和取反统一计数，调用内部的组合则保持独立。Rust closure
前缀、TypeScript regex 的词法位置以及 TSX prose 边界都由各自 adapter 校正。

`[limit] combination` 的默认值为三。finding 会指向 `combination` cookbook；
它区分时序状态与正交分类，也拒绝仅用 helper 隐藏同一个表达式。

## 更小、更清楚的诊断

`receiver` law 现在为每个拥挤的 receiver group 只报告一个 finding，并按源码
顺序列出全部成员。finding 也会指向 receiver cookbook。

普通 `.css` 文件现在与 `.scss` 一样进入 source 集，并携带相同的整文件 style
标记。

## 兼容入口退役

仅为 v0.4 系列保留的隐藏、无效果 `--strict` 拼写已经移除。`--strict` 与
`--debt` 现在都无效；唯一的正常扫描模式本就会阻塞每一项 finding。
