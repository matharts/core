# 当前公共 API 与编码变更

2026-10-04，按用户要求全库取消向后兼容。以下是当前契约，旧接口、旧字段名与兼容 feature 不再提供。本次未发布，消费方需自行升级调用与存量数据；库内不提供旧数据自动迁移或双格式解码。

## 领域查询入口

| 移除或改变的入口 | 当前调用 |
| --- | --- |
| `Branch::triangle_combination()`、`three_combination_partners()` | `branch.three_combination()` 返回 `ThreeCombination`；完整成员用 `members()` |
| `SexagenaryCycle::nayin_identity()` | `value.nayin()` 返回 `Nayin` |
| 原 `value.nayin() -> Element` | `value.nayin().element()` |
| `value.xun_start_branch()` | `value.xun().leader().branch()` |
| `value.void_branches()` | `value.xun().void_branches()` |
| `value.is_void_branch(branch)` | `value.xun().is_void_branch(branch)` |
| `Stem::polarity()`、`Branch::polarity()` | `primitive()` |
| `wuhu_dun`、`wushu_dun`、`dun` 模块 | `derive_month_stem`、`derive_hour_stem`、`stem_derivation` 模块 |
| `HiddenStems::residual` | `HiddenStems::tertiary` |
| `Stem::combination_stem()` | `stem.five_combination().partner_of(stem)` 返回 `Option<Stem>`；归属组中必为 `Some` |
| `Stem::transformed_element()`、`combination_element()` | `stem.five_combination().element()` |
| `Stem::ten_god_to(target)`、`clashing_with(target)` | `ten_god_of(target)`、`is_clashing_with(target)` |
| 原 `Branch::six_combination() -> Branch`、`six_combination_partner()` | `branch.six_combination() -> SixCombination`；伙伴用 `group.partner_of(branch)` |
| `SexagenaryCycle::leader_branch()`、`is_void(target)` | 通过 `xun()` 查询旬首与缺位关系 |

```rust
use matharts_core::{Branch, Element, Nayin, SexagenaryCycle, Xun};

let value: SexagenaryCycle = "甲子".parse().unwrap();
assert_eq!(value.xun(), Xun::JiaZi);
assert_eq!(value.xun().leader(), value);
assert!(value.xun().is_void_branch(Branch::Xu));
assert_eq!(value.nayin(), Nayin::HaiZhongJin);
assert_eq!(value.nayin().element(), Element::Metal);
assert_eq!(Branch::Zi.three_combination().members(), [Branch::Zi, Branch::Chen, Branch::Shen]);
```

旬、纳音、五合、六合、三合、三会的身份值与成员集合各由所属类型维护。`FiveCombination` 与 `SixCombination` 分别归入 `stem` 与 `branch` 模块并在 crate 根导出；两成员数组按各自领域索引升序。`from_stems()`／`from_branches()` 识别无序完整配对，拒绝重复与混组；`partner_of()` 对非本组成员返回 `None`。五合五行从组身份查询，六合不提供五行映射。原伙伴及五合五行方法移除，不设别名。

三支完整组的 `members()` 按地支索引升序，不提供旧四步／八步伙伴元组顺序。静态查询仍不判断空亡效应、实际成局、成化或吉凶。来源与范围见[规则边界](rules.md)、[完整关系组](branch-groups.md)及[五合与六合配对](pair-groups.md)。

文件按领域职责组织：`stem::{Stem, FiveCombination}` 定义天干值及五合；`branch::{Branch, HiddenStems, SixCombination, SixClash, SixHarm, SixBreak, ThreeCombination, ThreeMeeting}` 定义地支值、藏干记录及完整固定关系。各类型的表、查询与 Serde 都在所属领域文件内维护，不使用独立 Serde 模块或共享生产宏。

原 `five_combination`、`six_combination`、`six_clash`、`six_harm`、`six_break`、`three_combination`、`three_meeting` 七个独立类型模块移除；更早的 `combination`、`branch_relations` 及私有 `serde_support` 也不恢复。不添加旧路径别名，调用方改用 `stem`／`branch` 模块路径或 crate 根导入。类型名称、根导出、方法签名、固定成员、索引及编码保持不变。

### 六冲、六害及采用表六破身份

`branch::{SixClash, SixHarm, SixBreak}` 定义三类地支配对，同时在 crate 根导出。`Branch::six_clash()`、`six_harm()`、`six_break()` 返回单支所属身份；每类提供 `ALL`、`index()`、严格 `TryFrom<u8>`、`members()`、`contains()`、`partner_of()`、`from_branch()` 及 `from_branches()`。完整识别拒绝重复和混组，成员按地支索引升序。身份没有周期、大小排序或五行接口。

原 `Branch::is_clashing_with()`、`is_harming()`、`is_breaking()` 的签名与固定结果不变，委托对应类型统一识别。它们是两个支的谓词查询；身份入口另用于传递、保存和枚举关系。`opposite()` 仍表示对宫，`is_punishing()` 仍使用原有向表。六破限定为《六壬大全》采用表，无序六害身份不承诺作用对称，见[地支配对边界](branch-pairs.md)。

## 周期类型与特性

`CyclicRing::MODULUS` 改为 `core::num::NonZeroU8`，自定义实现直接提供非零周期。索引运算需要整数时调用 `.get()`；不再以私有泛型常量补验旧 `u8` 周期。

严格外部索引用 `TryFrom<u8>` 或 `try_from_index`；`from_index` 明确按周期回绕。`forward_distance` 接受 `NonZeroU8`，`checked_forward_distance` 接受外部 `u8` 并对零返回 `None`。极端 `i32` 位移仍在 debug/release 下安全约减。

库始终 `#![no_std]`、禁止 unsafe。删除无实际能力的 `std` feature，默认特性为空；唯一可选生产特性为 `serde`。测试、示例及 CI 只运行 Serde 关闭／开启两种配置，每种分别执行 debug 和 release。

## 当前 Serde 协议

- 可读格式中的枚举只接受精确 Rust 变体代码字符串，例如 `"Jia"`、`"HaiZhongJin"`。不接受数字、unit 对象、中文显示名或旧别名。
- `SexagenaryCycle` 只接受含 `stem`、`branch` 的对象，并校验阴阳配对。
- `HiddenStems` 只接受含 `primary`、`secondary`、`tertiary` 的对象。三个字段都必须出现，可选槽位用 `null` 表示为空；允许自定义重复成员及第三槽单独存在。
- `Hexagram` 只接受含 `lower`、`upper` 的对象。
- 所有可读记录拒绝数组、未知字段、重复字段与缺失字段。`residual` 不作为 `tertiary` 的别名。
- 紧凑格式使用 Serde 原生 unit variant／struct 模型，包括格式提供的合法 variant 标识和结构体序列。这是当前格式能力，不承诺旧版本或任意第三方格式的字节兼容。

例如藏干当前编码为：

```json
{"primary":"Ji","secondary":"Gui","tertiary":"Xin"}
```

枚举序列化采用领域类型上的 `Serialize` 派生，代码、类型名和 variant index 对应当前声明；不保留为旧名称与旧索引准备的手写输出映射。反序列化的输入限制由领域实现维护；五合在 `stem.rs`、六合及各地支关系在 `branch.rs` 内按类型显式维护代码表与 Serde 实现，无独立 Serde 模块或共享生产宏，干支合法值校验不能被绕过。

当前协议样本为 [encoding-v2.json](../tests/fixtures/encoding-v2.json)，取代旧藏干字段样本；[身份样本](../tests/fixtures/identities-v1.json)、[爻卦样本](../tests/fixtures/gua-v1.json)、[关系组样本](../tests/fixtures/branch-groups-v1.json)继续定义其当前值表。新增[配对样本](../tests/fixtures/pair-groups-v1.json)定义五合与六合的11个身份、代码、索引、成员及五合对应五行。版本化样本用于独立验收当前协议，不构成跨版本兼容承诺。

新增[地支配对样本](../tests/fixtures/branch-pairs-v1.json)定义六冲、六害及六破18个身份、代码、索引与成员，分别采用原生 unit variant 模型。可读编码只接受例如 `"ZiWu"` 的代码字符串，拒绝数字、对象、数组及中文名称。紧凑格式的索引必须在 `0..6`，不接受越界值或非 unit 变体。

`Display`／`FromStr` 仍只处理精确中文干支文本，拒绝空白、拼音、别名和非法配对，与 Serde 机器代码分开。错误对象不持有输入、不分配堆内存。

实际改动与验证结果见[兼容层清理记录](breaking-cleanup.md)。此前审查与实施记录仅代表其记录时点。
