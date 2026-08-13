# Ectropy v0.7.0

Ectropy 现在把 `.svelte` 文件作为一等源码地形。内嵌 grammar 将 TypeScript
脚本、响应式语句、rune、模板表达式、控制块、snippet、元素、注释与样式块映射
到既有共享结构树，不增加框架专属 kind。

Svelte 组件文件名是路径词汇原子。因此 `UserCard.svelte` 这样的复合组件名，
会和复合 Rust、TypeScript 或 TSX 文件名一样触发 `word` 法则。

法则目录与配置 schema 没有变化。grammar 覆盖面已经扩展；尚未支持的已接纳
语法仍会产生 `coverage`，不会给出虚假的 clean。
