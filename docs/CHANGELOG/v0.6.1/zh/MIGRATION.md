# 迁移到 Ectropy v0.6.1

调用方和 `ectropy.toml` 无需迁移。

## 受管 skill

stable 提升后用 stable 二进制升级受管 skill。评估候选版时，把精确
beta skill stage 到隔离路径，不要替换受管 seat。存储数据无需迁移。

已有的 Claude、Codex 席位除这次升级外不必另做处理。

若本机已有 `~/.grok`，下一次 `ectropy skill install` 会尝试
`~/.grok/skills/ectropy`。目录已在、但不在 Ectropy ledger 里的，一律
拒绝。先移走该目录，再 install。`--force` 不能认领。

没有 Grok home 的安装面不受影响。
