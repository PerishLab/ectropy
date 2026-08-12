# Ectropy v0.6.0

## 法条有了目录

十六条法此前以字符串字面量散落在各自的判定点，boundary 校验另有一份手写清单。
两份已经漂移。现在它们收敛为一份目录，记录每条法的名字、它拒绝什么、以及它是否
接受 boundary 豁免；boundary 校验由该目录派生。

新的 `ectropy law` 打印全表，`ectropy law <name>` 读一条。查目录，不要再抄清单。

## 三条法不接受 boundary 豁免

`coverage`、`grant` 和 `ban` 现在明确拒绝出现在 `[[boundary]] allow` 中。

`coverage` 拒绝是因为 clean 不能覆盖没有解析过的地盘；`grant` 和 `ban` 拒绝是
因为它们各自的 `paths` 已经承载了territory —— 要放宽就改 grant，要收窄就改 ban，
不必再走第二套路径机制。

这修正了一处静默失效：此前 `allow = ["grant"]` 与 `allow = ["coverage"]` 会被
配置校验接受，但判定点从不查询豁免，finding 照常报出。声明存在、行为不存在。
`allow = ["ban"]` 此前报的是笼统的 unknown boundary law。

现在两种错误是分开的：未知法名仍报 `unknown boundary law`，已知但封闭的法报
`boundary law X admits no exemption`，并带上该法为何封闭的理由。

## 成文法补齐

`docs/principles.md` 原本只写了 ban 不可豁免，现在三条都写明，并把 grant 一节
改述为「grant 是放宽，不是豁免」。`docs/stable.md` 的 surface 增列 `ectropy law`。

## 操作 brief 改为指针

skill 的 SKILL.md 不再复制十六条法的枚举，改为指向 `ectropy law`。brief 与二进制
之间因此少了一份会漂移的拷贝。
