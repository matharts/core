# 六冲、六害与采用表六破的身份模型

2026-10-04，新增 `SixClash`、`SixHarm`、`SixBreak`，分别表示六种完整两支配对。三类均归入地支 `branch.rs`，用于独立传递、比较、保存及枚举固定关系身份。原布尔谓词委托相应完整配对识别，固定结果不变。

## 准入范围与来源

本轮核对电子转录，未完成刻本影印校勘。采用版本和可定位位置如下：

| 类型 | 定义来源与采用版本 | 具体消费场景与限制 |
| --- | --- | --- |
| `SixClash` | [《三命通会》四库全书本卷二《论冲击》](https://zh.wikisource.org/wiki/三命通會_(四庫全書本)/卷02)；[《选择纪要》上编《地支相冲》](https://zh.wikisource.org/zh-hant/選擇紀要/上編)的电子转录 | 命理对宫与择日对宫成员一致。只查询固定集合；冲动、强弱与吉凶条件不在 Core |
| `SixHarm` | 同卷《论六害》与同上编《地支六害一名穿心六害》 | 命理六害与择日六害的六对成员一致。年日等使用位置由调用方确定；无序身份不表示受害效果对称 |
| `SixBreak` | [《六壬大全》卷三《鬼·破》原文电子转录](https://libokang.com/zh-hant/guji/liuren/六壬大全/3/)；页面未明确底本版次 | 与本库已有 `is_breaking` 六对完全一致，适用于该采用表的关系记录。跨体系复用证据待补，不将其认定为所有体系的“破” |

《选择纪要》六害段“卯辰六亥”的转录字样与段标题不一致，配对成员仍为卯辰；通过《三命通会》六害段交叉核对，不把该字样另建身份。《三命通会》分别解释酉见戌及戌见酉时的作用，因此不能由无序集合推断应用效果对称。页面现代译文或评语不作为新增表的依据。

### 相同输入的逐项对照

以下仅比较两支是否为该固定配对，交换输入顺序返回同一身份；不比较吉凶。六冲、六害分别逐项核对上述命理及择日文本的相同成员。

| 零基索引 | 六冲：命理／择日同表 | 六害：命理／择日同表 | 六破：《六壬大全》／本库原表同表 |
| --- | --- | --- | --- |
| 0 | 子午 `ZiWu` | 子未 `ZiWei` | 子酉 `ZiYou` |
| 1 | 丑未 `ChouWei` | 丑午 `ChouWu` | 丑辰 `ChouChen` |
| 2 | 寅申 `YinShen` | 寅巳 `YinSi` | 寅亥 `YinHai` |
| 3 | 卯酉 `MaoYou` | 卯辰 `MaoChen` | 卯午 `MaoWu` |
| 4 | 辰戌 `ChenXu` | 申亥 `ShenHai` | 巳申 `SiShen` |
| 5 | 巳亥 `SiHai` | 酉戌 `YouXu` | 未戌 `WeiXu` |

这项对照支持六冲、六害固定成员的限定共用，不代表命理与择日有相同作用条件。未运行外部排盘或择日消费方；仓内集成场景为 `examples/ganzhi.rs` 的关系查询与记录。

六破仍保留[规则边界](rules.md#六破及偏印)记录的异义：艺术典第601卷的破杀排除四孟，因此巳申、寅亥与本库采用表不同；《五行大义》的冲破表示对宫关系，子午也与本库六破不同。本轮未重新校勘这两篇底本，沿用现有审查记录。身份提升不弥补跨体系消费证据，也不引入任何替代表。

## 查询契约

每类均提供 `ALL`、`index()`、严格 `TryFrom<u8>`、`members()`、`contains()`、`partner_of()`、`from_branch()`、`from_branches()`。成员按地支索引升序；索引 `0..6` 表示固定身份顺序，没有周期或大小强弱含义。重复成员与混组输入返回 `None`，非成员伙伴查询返回 `None`。

`Branch::six_clash()`、`six_harm()`、`six_break()` 是单支的唯一所属身份。单支查询不意味着另一支已出现在应用场景中。原 `is_clashing_with()`、`is_harming()`、`is_breaking()` 仍是两支关系的便利谓词，统一使用上述成员定义；不是旧路径别名。`opposite()` 继续表示对宫，刑的方向、自刑与类别规则不在本轮改动。

关系不互斥：巳申及寅亥同时属于六合与采用表六破；巳申还具有巳到申的有向刑，反向并不成立。三种新类型没有五行、实际冲动、受害程度、破坏或吉凶接口。

## 编码与独立验收

Serde 可读格式只接受精确代码字符串，例如 `"ZiWu"`；紧凑格式使用各类型的原生 unit variant 模型。未知代码、越界索引、中文显示名、数字可读编码、对象、数组和非 unit 变体均拒绝。五合的代码表与 Serde 实现放在天干文件 `stem.rs` 内，六合及三类地支配对的实现分别维护在地支文件 `branch.rs` 内，不使用共享生产宏，无 `std` 或分配依赖。

`tests/fixtures/branch-pairs-v1.json` 冻结18行身份、索引、代码、成员与10个必需负例。`tests/branch_pairs.rs` 独立列出三张成员表，检查每类对12支的不重不漏分割、每个成员及非成员的伙伴查询、144个有序输入（12个接受）、256个严格索引、模块及根导出、const 查询和多关系共存。启用 Serde 后分别验证每类全部六行 JSON 输出、输入及模型 token；数量或负例缩水会失败。

文件组织调整的回归同时运行原五合与六合样本，测试不能替代来源校勘或第三方二进制格式的字节验收。

## 本轮实际验证

2026-10-04，在固定 Rust 1.99.0 工具链下完成以下检查，均退出成功：

- `rtk proxy mise exec -- hk check --all --slow`：格式、严格 Clippy、无特性及 Serde 两组 debug 全库测试。
- `cargo test --profile release --locked` 及加 `--features serde`：分别72项及96项通过，包含文档测试。新增地支配对文件分别执行5项及8项测试，原五合六合文件分别执行5项及7项测试。
- `cargo build --lib --target thumbv7m-none-eabi --no-default-features --locked`，以及加 `--features serde` 的两组裸机库构建。
- `RUSTDOCFLAGS='-D warnings' cargo doc --no-deps --all-features --locked`。
- `cargo run --example ganzhi --profile dev/release --locked`，分别关闭及启用 Serde 的四组实际运行。固定断言通过，输出子午、子未、子酉所属配对。

以上 `cargo` 命令均经 `rtk proxy mise exec --` 执行。本地通过不代表远端 CI、硬件运行、外部消费方接入或领域来源校勘已完成；本轮没有提交或发布。

## 前一轮文件拆分记录（历史阶段）

> 下文记录前一轮按类型拆分的检查，随后用户确认改按领域归属组织。当前布局见下节，历史通过结果不替代当前验证。

按当时对规范的理解，曾将本次五类配对分别组织为 `five_combination.rs`、`six_combination.rs`、`six_clash.rs`、`six_harm.rs`、`six_break.rs`。每个文件直接定义其领域枚举、固定成员、查询、严格构造及 Serde 实现；移除 `combination.rs`、`branch_relations.rs`、`serde_support.rs` 和共享生产宏。对应模块路径及 crate 根导出已同步，原模块路径没有别名，见[迁移说明](migration.md)。规范同时补入仓库 `AGENTS.md`。

修复后重新执行全量 hooks、两组 release 测试（72／96项）、两组裸机库构建、严格 rustdoc 和四组示例实际运行，全部成功。全部六份冻结 JSON 样本的 SHA-256 与修复前相同；新模块及根路径均由集成测试实际调用。源码不再包含移除模块的引用或共享宏，根类型、成员、方法签名和编码不变。此次验证只覆盖本地文件组织与公共契约，来源及外部验收限制仍如上所述。

## 按领域归属合并（2026-10-04）

按用户确认的方案，以领域职责决定文件边界：天干五合归入 [stem.rs](../src/stem.rs)，六合、六冲、六害、采用表六破、三合和三会归入 [branch.rs](../src/branch.rs)。七个独立类型文件及其模块声明已移除，各身份的固定表、查询与 Serde 实现仍分别显式维护；没有增加共享宏或独立 Serde 模块。仓库 `AGENTS.md` 改为按领域组织，不再要求一个身份类型对应一个文件。

模块导入改为 `stem::FiveCombination` 或 `branch::{SixCombination, SixClash, SixHarm, SixBreak, ThreeCombination, ThreeMeeting}`；crate 根导出保持不变，不保留旧模块路径别名。迁移说明、README 和当前领域文档已同步。三类相关集成测试分别验证新领域模块与根导出路径；七个类型的文档示例也在所属领域文件中实际执行。

合并后重新执行 `rtk proxy mise exec -- hk check --all --slow`、两组 release 全库测试（无 Serde 72项、启用 Serde 96项，包含文档测试）、两组 `thumbv7m-none-eabi` 库构建、严格 rustdoc 和 debug／release × Serde 开关的四组示例运行，全部通过。六份冻结 JSON 样本 SHA-256 未变，源码、测试及示例中无旧模块路径引用；类型成员、构造及编码契约不变。此处是本次合并的验证，前节结果仅作为历史记录；未提交或发布，外部消费方、远端 CI 与硬件运行不在本地验证范围内。
