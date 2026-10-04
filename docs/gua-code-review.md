# 爻卦实现代码审查

> 历史记录：本文反映兼容层清理之前的设计或检查状态，不代表当前接口、编码、特性矩阵或验证结果。当前契约见 [API 与编码变更](migration.md)，本轮结果见 [全库清理记录](breaking-cleanup.md)。

日期：2026-10-04。范围：G1 生产实现，以及删除独立 Serde 模块后内联到类型文件的代码。

结论：本次范围内未发现可确认的、需要修复的功能或兼容性缺陷。

## 审查依据

当前源码尚未纳入 Git 跟踪，因此本次按 [G1 契约](gua-design.md)核对工作树中的 [trigram.rs](../src/trigram.rs)、[hexagram.rs](../src/hexagram.rs)、根导出、Cargo 特性以及对应测试；不将其描述为基于某次提交的 diff 审查。

核查了八卦位表、上下卦方向、既有阴阳编码的显式转换、全部构造边界、爻位操作、完整六爻倒序，以及固定 Serde 名称／代码／索引、字符串输入、compact 枚举输入和严格 map／sequence 路径。删除共享宏后的三个枚举实现仍保持各自固定编码和输入约束。按照用户要求保留类型文件内的实现，不将这一组织选择重新列为缺陷。

## 本轮实际验证

- 重跑 `cargo test --locked --all-features --test gua --test gua_serde`：4组结构测试及5组编码／数据模型测试全部通过。
- 临时审查探针直接依赖当前生产 crate，另有3组测试全部通过：两种爻位的 compact 越界索引、非 unit payload、全部9个字符串标识；嵌套六爻序列不会吞掉外层后续字段；compact map 接受相反字段顺序并拒绝缺失／重复 `upper`。

临时探针保存在 `/tmp/matharts-gua-review-20261004/`，未写入生产源码或正式回归测试。复跑命令：

```sh
rtk proxy mise exec -- cargo test --offline --manifest-path /tmp/matharts-gua-review-20261004/Cargo.toml
```

## 限制

本轮没有重跑全部工程矩阵；此前工程验证见 [实现记录](gua-implementation.md)。额外 compact 验证使用 Serde token 模拟，未对具体二进制格式做兼容性认证。额外探针覆盖的两个爻位非法输入及嵌套场景可在后续维护中纳入正式回归测试，但本轮没有据此发现生产缺陷。
