# 迁移到 Ectropy v0.6.0

## 检查 `[[boundary]] allow` 里有没有 grant 或 coverage

这是本版唯一的破坏性变更，升级前请先扫一遍每个仓库的 `ectropy.toml`：

```sh
grep -n 'allow' ectropy.toml
```

若某条 `[[boundary]]` 的 `allow` 含有 `"grant"` 或 `"coverage"`，v0.6.0 会让整个
配置加载失败并以退出码 2 拒绝，错误形如：

```
ectropy: boundary law `grant` admits no exemption: reserved syntax outside granted paths; widen the grant itself
```

在 v0.5 及更早版本，这两个名字会被静默接受且完全不起作用——finding 照常报出。
所以移除它们不会改变扫描结果，只会让配置重新可加载。真正需要的是：

- 本来想放宽 `grant`：改 `[[grant]]` 的 `paths`，把该territory 纳入声明。
- 本来想收窄 `ban`：改 `[[ban]]` 的 `paths`。
- 本来想豁免 `coverage`：没有这条路。让解析器读懂那段语法，或把该文件移出
  `[scan] include`。声称 clean 的前提是真的读过。

`allow = ["ban"]` 此前就会报错，只是措辞是笼统的 unknown boundary law，现在会说明
原因。其余十三条法的 boundary 行为不变。

## 不再抄法条清单

若你的仓库文档或 agent brief 复制了十六条法的枚举，改为指向 `ectropy law`。目录
是唯一真相，抄本会漂移——本版修的正是这种漂移。

## 受管 skill

stable 提升后可用 stable binary 升级受管 skill；评估候选版时，把精确 beta skill
stage 到隔离路径，不要替换受管 seat。存储数据无需迁移。
