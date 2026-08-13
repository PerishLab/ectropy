# 迁移到 Ectropy v0.7.0

扫描 pattern 已经包含 `.svelte` 文件的仓库，现在会真正检查这些文件。请把复合
组件文件名改成一个词汇原子、移除注释，并只在仓库 style grant 或 ban 允许的
地形中放置组件样式块。

无需迁移 `ectropy.toml` schema。本版 TypeScript 与 TSX adapter 行为保持不变。
