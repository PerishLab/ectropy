# Ectropy v0.4.0

## 所有 finding 都是错误

Ectropy 现在只有一种拒绝模型：退出码为零就意味着严格的 `clean`，任何已报告的
finding 都以退出码一结束。word、dispatch、receiver、parameter、burr、shadow
和 coverage 不再作为可容忍的 debt 或 blindspot。

旧的 `--strict` 拼写在 v0.4 系列中保留为隐藏、无效果的兼容选项，并将在 v0.5.0
移除。旧的 `--debt` 选项现已无效。

## 带版本的操作 skill

每个 release 现在都携带 `ectropy-skill.tar.gz` 资产。新的 `ectropy skill`
命令可以安装、检查、升级、列出和卸载受管的 Ectropy 操作简报。stable 与 beta
release smoke 会在 Linux、macOS 和 Windows 上验证完整的 skill 生命周期。

## 可信的原生 Windows 扫描

在应用 `ectropy.toml` 的 scan、module、boundary、grant 与 ban glob 前，
仓库相对路径会先规范为配置使用的斜杠方言。带 include 配置的扫描不再在 Windows
上静默匹配零文件。

Rust adapter 也能识别零 `#` 的 raw string 与 byte raw string，包括以反斜杠
结尾的 Windows 路径字面量。新的原生 Windows guard 会为每次变更运行 workspace
测试与 PowerShell manager smoke。

## 默认只保留一个安装版本

Unix 与 PowerShell manager 会在新 binary 完成链接并通过 `--version` 后移除
旧版本。使用 `--retain` 可以保留已有版本。已发布资产保持不可变，也可以通过指定
版本重新获取。

stable release 现在还会在发布开始前要求完整的中英文变更与迁移说明。
