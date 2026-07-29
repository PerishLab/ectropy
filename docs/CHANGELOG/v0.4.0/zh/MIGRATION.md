# 迁移到 Ectropy v0.4.0

## 处理每一项 finding

此前依赖“仅有 debt 仍成功”的仓库，现在必须处理每一项 finding，Ectropy 才会
以零退出。请修复对应结构；只有在该 law 确实不适用时，才声明范围窄且理由明确的
boundary。

请从脚本中移除 `--debt`。`--strict` 在 v0.4 中可以暂时保留，但不会改变结果，
并将在 v0.5.0 移除。

Windows 用户升级后应重新运行 Ectropy。此前因 Windows 路径分隔符而没有匹配任何
文件的 include 配置，现在会扫描预期目录，因此可能暴露真实 finding。

## 选择是否保留已安装版本

manager install 现在默认只在安装根目录留下一个版本。需要离线保留旧版本目录时，
请传入 `--retain`；否则回滚使用 `install --version <older>`，重新下载不可变资产。

`ectropy.toml` schema 和已有数据都不需要迁移。安装受管 Ectropy skill 是可选项。
