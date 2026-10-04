# 爻卦 G1 实现与验收

> 历史记录：本文反映兼容层清理之前的设计或检查状态，不代表当前接口、编码、特性矩阵或验证结果。当前契约见 [API 与编码变更](migration.md)，本轮结果见 [全库清理记录](breaking-cleanup.md)。

日期：2026-10-04。状态：本地技术交付完成；未提交、推送或发布。依据为修订后的 [G1 方案](gua-design.md)，前序设计审查见 [R1／R2 记录](gua-design-review.md)。

## 实现范围

- [trigram.rs](../src/trigram.rs)：八卦身份、中文名、三爻构造与拆分、严格位编码、三爻位及查询／赋值／翻转／倒序。
- [hexagram.rs](../src/hexagram.rs)：64种六爻结构、上下卦构造与拆分、严格位编码、六爻位及对应结构操作。两个字段私有；所有8×8组合均合法。
- 三种枚举的 Serde 实现直接放在各类型所在文件中，无独立 Serde 模块或共享宏。类型名、代码、variant index 显式固定，既不借用已有阴阳编码，也不依赖 Rust 声明顺序。
- 根导出 `Trigram`、`TrigramPosition`、`Hexagram`、`HexagramPosition`。纯操作为 `const fn`，值可复制、无堆分配；未增加生产依赖。`serde_test` 仅为开发依赖。

JSON 枚举只接受代码字符串，六爻卦只接受对象；长度正确的数组也拒绝。非人类可读输入保留双元素结构体序列；两模式均检查缺失、重复、未知字段。输出固定为 unit variant／struct 数据模型，不承诺具体二进制格式的字节兼容。

## 独立预期与实际测试

[结构测试](../tests/gua.rs)覆盖8个独立固定三爻形状、全部64个上下卦组合、四类构造器的全部256个输入、408个爻位查询及翻转、816个爻位赋值、72种结构的逐爻取反和完整倒序，以及不同爻位修改的交换性。参考结果通过阴阳数组构造，不调用生产变换函数生成预期；另在编译期调用代表性 const API。

[Serde 测试](../tests/gua_serde.rs)直接调用生产类型，已从隔离探针迁入：

- 64条冻结组合编码分别检查当前输出和固定旧输入，样本数量、bits 覆盖及解码结果唯一性均断言。
- 6个语义案例、16个非法 JSON 值的 `from_str`／`from_value` 双入口、2个原始重复键和相反键顺序。
- 17个 variant 的名称／索引／代码、compact 索引输入；结构体模型、名称、字段顺序、人类可读及 compact 输入分支。
- string 替代 unit variant、枚举重命名、索引变化、struct 改为 map、结构体重命名、字段顺序变化等6类反向检查。

新增 [gua-v1.json](../tests/fixtures/gua-v1.json) 是固定验收文件，64组合预期由静态表列出，不从生产实现导出；设计样本继续保留为历史设计材料。现有 `encoding-v1.json` 和 `identities-v1.json` 均未修改，SHA-256 与实施前一致。新样本 SHA-256：`5332ab6d4f6eb85146085161db102a063a092e11005dfbc0abf29d9db143bc7f`。

## 工程检查

实际工具链：Rust 1.99.0；Serde 1.0.229、serde_json 1.0.151、serde_test 1.0.177。下列检查全部退出码0：

| 检查 | 结果 |
| --- | --- |
| debug／release × 默认、无默认、无默认加 Serde、全特性 | 八组测试通过，含原回归和 rustdoc 示例；未启用 Serde 时仅编码测试按特性不编译 |
| 新增结构测试／生产 Serde 测试 | 4组／5组通过；无 ignored 或 filtered out |
| 原有 `ganzhi` 示例，同样八组运行配置 | 全部通过 |
| `thumbv7m-none-eabi`，无默认／无默认加 Serde | 两组库编译通过 |
| 全目标全特性 Clippy，`-D warnings` | 通过 |
| rustdoc，`RUSTDOCFLAGS=-D warnings` | 通过 |
| `cargo fmt --check` | 通过 |

在仓库根目录复跑的命令与配置见 [CI 工作流](../.github/workflows/ci.yml)。本地用 `rtk proxy mise exec -- cargo` 代替工作流中的 `cargo +1.99.0`。单独复跑新增测试：

```sh
rtk proxy mise exec -- cargo test --locked --all-features --test gua --test gua_serde
```

开发中修正了由隔离测试迁入引起的私有字段访问，以及严格 lint 提示，之后完成上述全套验证。

## 范围与后续

此次完成 G1 纯结构层；六十四卦中文名称、文王卦序、先后天编号和方位仍属 G2 的独立核验任务。起卦、动爻选择、互卦特判、纳甲、世应、六亲、体用和占断仍在领域层；未声称已经验证实际下游接入或全部流派。

Jev 依据上述生产测试、工程矩阵及兼容证据判定“G1 本地技术交付完成”（置信度0.86）；该结论不包含 G2 或发布，也不替代实际测试。

## 文件组织调整（2026-10-04）

按用户要求去掉独立的 Serde 模块和共享宏，将实现直接收回各类型所在文件。公共 API、固定类型名／代码／索引和两种输入模式保持不变，三个冻结编码样本 SHA-256 均未改变。本次重跑全特性测试、两组裸机库编译、严格 Clippy／rustdoc 和 fmt，全部通过；源码及文档已无旧模块引用。此处记录本次重构检查，前述八组矩阵属于 G1 初次实现验收。
