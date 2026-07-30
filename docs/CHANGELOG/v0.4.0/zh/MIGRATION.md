# 迁移到 Ectropy v0.4.0

## 处理每一项 finding

此前依赖“仅有 debt 仍成功”的仓库，现在必须处理每一项 finding，Ectropy 才会
以零退出。请修复对应结构；只有在该 law 确实不适用时，才声明范围窄且理由明确的
boundary。

请从脚本中移除 `--debt`。`--strict` 在 v0.4 中可以暂时保留，但不会改变结果，
并将在 v0.5.0 移除。

Windows 用户升级后应重新运行 Ectropy。此前因 Windows 路径分隔符而没有匹配任何
文件的 include 配置，现在会扫描预期目录，因此可能暴露真实 finding。

## 采用 stable 交付

CI 应将 `setup-ectropy` 替换为
`PerishLab/actions/setup-binary@main`，并设置
`PERISH_SETUP_PRODUCT=ectropy`。

manager 的默认安装现在跟随 stable。安装 non-stable 时，必须指定精确版本，
并显式提供与默认值互不重叠的安装路径和 binary 路径。

`ectropy.toml` schema 和已有数据都不需要迁移。安装受管 Ectropy skill 仍为
可选项。
