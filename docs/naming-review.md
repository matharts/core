# 当前代码命名审查

> 历史记录：本文反映兼容层清理之前的设计或检查状态，不代表当前接口、编码、特性矩阵或验证结果。当前契约见 [API 与编码变更](migration.md)，本轮结果见 [全库清理记录](breaking-cleanup.md)。

执行状态：用户确认后，N1–N6 六组公开名称已按本报告建议调整，旧方法和字段名不再保留为源码入口；核心类型名及 Serde 编码不变。当前名称与迁移对照见[迁移说明](migration.md)。下文保留改名前的审查证据和行号，“未修改”“建议”等表述均指审查时的状态；内部与测试名称的可选整理未纳入本次执行。

日期：2026-10-04。范围：当前工作区 `src/` 的公开类型、方法、字段、模块和主要内部变量，以及测试名称；不是相对于某个提交的差异审查。

结论：发现六组值得讨论的命名问题，其中前三组容易引起领域语义或调用方向误读，后三组主要影响一致性和可发现性。它们不是已确认的计算错误。仅新增本报告，未修改源码或公开接口。

## 优先讨论：名称是否准确表达契约

### N1：`HiddenStems.residual` 暗示了实现不承诺的“余气”语义

位置：[branch.rs:42](../src/branch.rs#L42)、[branch.rs:45](../src/branch.rs#L45)。

`secondary` 的注释说明它是第二槽，不表示统一中气或强弱；`residual` 的注释说明它是第三槽，不等同于季节承接意义的余气。当前字段名却混用了顺序词与领域解释词，调用方容易把第三槽直接用于“余气”计算。

建议使用 `primary / secondary / tertiary`，其中 `primary` 继续表示本气，后两项只表示附加成员的固定槽位。无需为改名顺便重设计结构。若采纳，应保留第三字段的 `#[serde(rename = "residual")]`，以维持现有序列化表示；Rust 字段访问和结构体字面量仍需迁移。

### N2：`transformed_element()` 容易让人以为已经判断合化

位置：[stem.rs:104](../src/stem.rs#L104)。

实现只按天干查询五合对应五行，既不接收另一干，也不接收合化条件。`transformed` 表示已经发生的转化，比实际返回契约更强。例如 `Stem::Jia.transformed_element()` 总返回土，不证明甲己合化土成立。

建议 `combination_element()`，与现有 `combination_stem()` 配套；保留“仅返回固定对应，不判断成化”的说明。此方法曾被明确要求保留原名，所以这是重新讨论项，不应作为一般清理直接改掉。

### N3：`ten_god_to(target)` 的命名没有充分表达谁相对于谁

位置：[god.rs:49](../src/god.rs#L49)，对照 [element.rs:96](../src/element.rs#L96)。

`Element::relation_to(target)` 返回自身指向目标的生克关系；`Stem::ten_god_to(target)` 返回目标相对于自身的十神身份。两者都有 `_to`，但返回值的叙述视角不同。

由当前实现可得：`Stem::Jia.ten_god_to(Stem::Bing)` 返回 `ShiShen`，表示丙是甲的食神；反向调用返回 `PianYin`。误把结果理解为“甲对于丙是什么十神”会读反关系。

建议 `ten_god_of(target)`，明确接收者为参照，查询目标的十神；配套保留正反两个例子。该名字仍需文档说明参照方向，不能仅靠换一个介词解决全部歧义。无需强制把接收者叫 `day_master`，当前契约允许任意参照天干。

## 次要改进：一致性与返回形状

### N4：天干和地支的相冲谓词命名不一致

位置：[stem.rs:113](../src/stem.rs#L113)、[branch.rs:316](../src/branch.rs#L316)。

两者都是 `self, target -> bool`，但分别叫 `clashing_with` 和 `is_clashing_with`。使用者从地支切换到天干时需要记忆一个没有语义理由的差别。

建议把天干方法统一为 `is_clashing_with()`。这是本轮最明确、改动范围最小的一项一致性改进。不要因此机械地给全部关系方法加 `_with`：`is_punishing(target)` 有方向，不能把它写得像对称关系。

### N5：组合查询名称没有表达返回的是伙伴

位置：[branch.rs:301](../src/branch.rs#L301)、[branch.rs:308](../src/branch.rs#L308)。

`six_combination()` 返回六合中的另一支；`triangle_combination()` 返回三合中的另外两支，均不包含自身，也不判断实际成局。`triangle` 还把这里的三合名称翻译成了几何图形，与 `six_combination` 的命名方式不一致。

建议：

| 当前名字 | 建议名字 | 返回内容 |
| --- | --- | --- |
| `six_combination()` | `six_combination_partner()` | 一个六合伙伴 |
| `triangle_combination()` | `three_combination_partners()` | 两个三合伙伴 |

这些是项目内命名建议，不宣称是统一的传统术语英译。继续明确两个伙伴按偏移 4、8 的顺序返回。`Stem::combination_stem()` 已带返回对象，可暂时保留，不必为整齐再改一轮。

### N6：旬查询可以更明确地标出领域和被判断对象

位置：[sexagenary.rs:84](../src/sexagenary.rs#L84)、[sexagenary.rs:106](../src/sexagenary.rs#L106)。

`leader_branch()` 没有说明是“旬”的首支；`is_void(target)` 判断的是参数地支是否属于本旬缺位集合，名字容易被读成判断当前干支自身。

建议 `leader_branch()` → `xun_start_branch()`，`is_void(target)` → `is_void_branch(target)`；保留返回两支的 `void_branches()`。这是可读性改进，已有注释足以解释当前行为，优先级低于前面的语义风险。

## 不建议顺手修改的名称

[迁移说明](migration.md)记录了此前明确保留的命名决定。本次审查不把重新翻译核心词汇当作必须执行的整改。

| 名称 | 当前评价 |
| --- | --- |
| `Primitive`、`primitive()` | 单看名字较泛，靠阴阳枚举及领域文档补全；属于已确认的项目词汇。 |
| `God` | 比 `ten_god_to` 的领域范围更宽，但已确认为十神类型；不要借此轮局部清理改类型名。 |
| `Growth` | 单独看不如完整的阶段名称明确；`growth_phase_at()` 已补足“阶段查询”语义。 |
| `SexagenaryCycle` | 实际是周期中的一个值，存在“整体周期／单个值”的可读性取舍；当前保留。 |
| `CyclicRing`、`MODULUS` | 当前契约是有限周期序列，不提供数学环运算；保留名称，相关描述必须准确。 |
| `from_index` | 明确采用回绕语义，与 `try_from_index`、`TryFrom` 的严格构造区分；不再次要求改名。 |
| `nayin()` | 只返回纳音五行，范围比完整纳音身份窄；此前及本轮合并均确认保留原名。 |
| `derive_month_stem`、`derive_hour_stem` | 用途明确，当前模块名 `stem_derivation` 也匹配职责。 |
| `ElementRelation`、`generated_by`、`overcome_by` | 主被动关系与源码方向吻合，无需为了统一前后缀改名。 |
| `Stem`、`Branch`、各拼音枚举项 | 类型提供足够上下文；`Stem::Wu` 与 `Branch::Wu` 不需要人为制造不同拼音。 |

## 内部与测试名称的低成本整理

- [stem_derivation.rs:34](../src/stem_derivation.rs#L34)、[stem_derivation.rs:66](../src/stem_derivation.rs#L66)：两个 `target_branch` 可分别叫 `month_branch`、`hour_branch`，让函数签名直接表达参数用途。
- [stem_derivation.rs:36](../src/stem_derivation.rs#L36)：`start_idx` 可写成 `start_stem_index`，`delta` 可写成 `branch_distance`。短公式内的 `s`、`b` 与数学注释直接对应，不必全部展开。
- [math.rs:4](../src/math.rs#L4) 的“环代数”描述与 `ring.rs` 明确声明的“不定义数学环的加法与乘法”不一致；即使保留 `CyclicRing`，这里也应写“周期运算”。这是注释术语问题。
- [tests/rules.rs:38](../tests/rules.rs#L38) 的 `named_growth_rule_covers_all_hundred_twenty_positions` 保留了“具名规则”的旧说法，当前调用的是 `growth_phase_at()`；可改为 `growth_phase_at_covers_all_120_stem_branch_pairs`。
- [tests/encoding.rs:13](../tests/encoding.rs#L13) 的辅助函数 `check` 可改为 `assert_frozen_encoding`，提示它检查冻结编码，而不只是泛化验证。

这些项目应随相关模块改动顺便处理，不值得单独制造大量 diff。

## 采用与验证边界

若采纳，先处理 N4 这种明确的一致性问题，再决定 N1–N3 的公开契约用词，最后考虑 N5–N6。公开改名会影响源码调用，不能以 JSON 字段保持不变为由声称完全兼容；显式 Serde 名称与冻结编码样本应维持原样。是否保留旧入口，需要依据实际下游使用情况决定，本轮未调查外部消费者。

本轮核对了当前源码、调用语义和仓库命名约定，没有修改行为，未重跑测试，也未开展新的古籍术语考证。Jev 支持上述“语义误读风险优先、风格建议次之”的分类，并支持十神方向歧义属于命名可读性问题；该判断不证明用户实际会误用，也不代表新名字已获采纳。
