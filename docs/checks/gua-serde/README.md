# 爻卦 Serde 设计探针

后续状态：G1 已实现，生产验收位于 [tests/gua_serde.rs](../../../tests/gua_serde.rs)，完整记录见 [G1 实现记录](../../gua-implementation.md)。本目录保留实施前的隔离实验，下文的“G1 仍须”描述是当时的交接要求。

这是 [爻卦方案](../../gua-design.md) R1/R2 修订的隔离实验，不引用或导出生产 `matharts-core` 类型，也不修改根 crate 的依赖。锁定 Rust 1.99.0 环境中的 Serde 1.0.229、serde_json 1.0.151、serde_test 1.0.177；首次运行需获取独立 Cargo.lock 中的开发依赖。

在仓库根目录重跑：

```sh
rtk proxy mise exec -- cargo test --locked --manifest-path docs/checks/gua-serde/Cargo.toml
rtk proxy mise exec -- cargo fmt --manifest-path docs/checks/gua-serde/Cargo.toml --check
rtk proxy mise exec -- cargo clippy --locked --manifest-path docs/checks/gua-serde/Cargo.toml --all-targets -- -D warnings
```

四组测试分别验证：

- JSON：6个案例，16个非法值分别走 `from_str`／`from_value`，2个原始重复键，倒序对象键；保留常规结构体派生接受长度正确数组的对照。
- 输入模式与结构体模型：人类可读模式拒绝序列，非人类可读模式接受双元素序列、拒绝缺项／多项；两模式检查 map 的缺失、重复、未知字段；固定结构体名称、字段序列及输出类别。
- 全部17个枚举 variant：独立观察名称／index／代码，验证固定代码输入、compact 数字标识输入、拒绝 JSON 对象式 unit variant；另检查非法索引和非 unit payload。`serde_test` 的输出 token 不观察 index，因此另有小型 serializer，不能删成单纯 token 往返。
- 反向检查：string 替代 unit variant、枚举重命名、索引变化、struct 改成 map、结构体重命名、字段顺序变化，均被对应模型检查识别。预期失败的模型断言用 `catch_unwind` 捕获；测试成功表示它们确实失败。

输入使用 [设计样本](../../fixtures/gua-design-v1.json)，显式断言样本数量。非人类可读路径通过模型 token 模拟，不代表任何具体二进制格式的字节兼容。这里的类型是验证策略的最小复制品；G1 仍须把相同契约接到真正生产类型，补齐独立64组合样本、完整工程矩阵与 no_std 验收。G0 位运算的旧演算不由本探针覆盖。
