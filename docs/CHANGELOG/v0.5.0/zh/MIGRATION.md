# 迁移到 Ectropy v0.5.0

## 命名过大的组合模型

升级后请重新运行 Ectropy。已有 `ectropy.toml` 不需要修改 schema：省略
`[limit] combination` 时默认值为三。若仓库维护完整的本地 limit 清单，也可以
显式记录这个值。

处理每一项新的 `combination` finding 时，应命名缺位的模型。随时间变化的事实
应成为显式 state 与 transition；彼此正交的事实应成为 classification、pattern、
decision table 或 domain type。把原表达式不变地移入 helper 并不能解决 finding。

扫描范围较宽的仓库还应检查普通 `.css` 文件暴露的新 finding；它们现在与
`.scss` 一样被识别。

## 移除已退役选项

将 `ectropy --strict <root>` 改为 `ectropy <root>`。这个选项在 v0.4 中没有效果，
并在 v0.5 中变为无效。`--debt` 仍然无效。

无需迁移存储数据或受管 skill。stable 提升后可用 stable binary 升级受管 skill；
评估候选版时，则应把精确 beta skill stage 到隔离路径。
