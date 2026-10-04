# MathArts Core 整体与逐文件设计审查

> 历史记录：本文反映兼容层清理之前的设计或检查状态，不代表当前接口、编码、特性矩阵或验证结果。当前契约见 [API 与编码变更](migration.md)，本轮结果见 [全库清理记录](breaking-cleanup.md)。

后续状态：下文保留修复前的审查快照。F1、F2、F3 已完成优化，修改与当前验证结果见[修复记录](design-review-optimization.md)。

审查日期：2026-10-04。对象：当前工作树，而非某个已提交版本的 diff。此次只新增本报告，未修改生产代码、测试、配置或既有文档。

## 结论

**保留当前架构，先补稳定编码验收和本地测试触发，再修正文档。** 发现 2 个 P2、1 个 P3；没有确认新的计算错误或 P0/P1 问题。通过现有测试不代表没有维护风险：本轮用变异实验复现了现有测试漏掉的 Serde 模型变化。

单 crate、按领域值拆文件、不可变有限类型、严格构造与周期构造分开、可选 Serde、无堆分配的基础计算，这些选择与当前规模匹配。没有必要拆多 crate、引入规则插件、泛型代数框架或统一序列化宏。

审查覆盖 16 个生产源码文件、8 个集成测试文件、3 个测试样本文件，以及示例、工程配置、设计文档和仓库静态资产。生成目录、Git 内部数据、CodeGraph 数据库和工具运行日志不属于产品设计审查范围；历史审查文档按其历史定位核对，不将其中的旧快照当作现行缺陷。

## 确认的问题

### F1 · P2：旧类型的 Serde 模型依赖声明顺序，现有验收未覆盖

位置：[stem.rs:15](/Users/lzm0x219/Code/github.com/matharts/core/src/stem.rs:15)、[encoding.rs:13](/Users/lzm0x219/Code/github.com/matharts/core/tests/encoding.rs:13)、[core-plan.md:124](/Users/lzm0x219/Code/github.com/matharts/core/docs/core-plan.md:124)。

设计要求明确写了“不依赖枚举声明位置”及“Serde 数据模型也需固定”。目前 `Primitive`、`Element`、`ElementRelation`、`Stem`、`Branch`、`God`、`Growth`、`Xun`、`Nayin` 使用派生 Serde。显式 `rename` 固定了字符串代码，`repr(u8)` 和显式 discriminant 固定了领域数值，但没有固定 Serde 的 variant index；后者仍随声明位置变化。

旧类型的冻结样本测试主要通过 JSON 检查代码和对象形状，没有像爻卦测试一样观察模型类别、类型名、variant index 和结构体字段序列。其后果是：维护者即使保持所有显式数值、字符串代码和冻结样本不动，也可能在测试全绿时改变已承诺的模型。

**实际复现：** 在 `/tmp/matharts-core-design-mutant-20261004/` 建立隔离 crate，仅将 `Stem::Jia` 和 `Stem::Yi` 两段声明交换，保留 `Jia = 0`、`Yi = 1`、所有 `rename`、`ALL` 和算法不变；其他生产模块及原仓库全部 8 个测试文件直接引用原文件。结果：

- 原有测试和文档测试全部通过。
- `Stem::Jia.index()` 仍为 `0`，JSON 仍为 `"Jia"`。
- 独立 Serializer 观察到 `("Stem", 1, "Jia")`，原来的 Serde index `0` 现在反序列化成 `Stem::Yi`。

这证明的是**当前实现对内部重排不稳定，且验收存在缺口**，并非声称当前发布数据已经损坏，也不等于已验证某种二进制格式。

建议：把既有类型全部纳入模型契约检查，固定每个 variant 的名称、index、代码及输入路径；结构体也锁定名称、字段序列和模型类别。若继续履行“不依赖声明位置”的承诺，枚举应显式编码索引；若决定保留 derive 并禁止重排，则需要明确修订设计约束并用测试拦截顺序变化，不能把它描述为已经与声明顺序解耦。实现继续放在所属领域文件中，无须恢复独立 `gua_serde` 模块。

验收：上述变异必须被契约测试拒绝；独立冻结预期不能从当前枚举声明或实现自动生成。补充结构体模型检查时，应保留合法性校验及已有 JSON 行为，不顺带收紧未约定的输入。

### F2 · P2：仅修改冻结样本时，本地 hooks 跳过测试

位置：[hk.pkl:5](/Users/lzm0x219/Code/github.com/matharts/core/hk.pkl:5)、[hk.pkl:12](/Users/lzm0x219/Code/github.com/matharts/core/hk.pkl:12)。

`rustTests` 复用 `rustInputs`，其中没有 `tests/fixtures/**`。这些 JSON 是测试读取的验收输入，但单独修改它们时，测试步骤不会触发。`pre-push` 也复用同一个测试定义。

实际执行：

```sh
rtk proxy mise exec -- hk check --slow --plan --json \
  tests/fixtures/encoding-v1.json \
  tests/fixtures/identities-v1.json \
  tests/fixtures/gua-v1.json
```

三个步骤均返回 `skipped`，原因是 `filter_no_match`，进程退出码为 `0`。本次只查看计划，没有修改暂存区或运行提交。

影响限定在**本地质量门**：远端 CI 没有路径过滤，仍会测试，不能据此声称远端也漏检。

建议：分开定义源码输入与测试输入，让测试输入包含 `tests/fixtures/**`。JSON 变更没有必要强制运行 Rust 格式化，但必须触发读取这些数据的测试。

验收：对任一冻结样本单独做计划检查，`cargo-test` 必须匹配并计划执行；在隔离副本中将样本改成错误预期，实际测试必须失败。不能只验证 `.rs` 文件变更路径。

### F3 · P3：迁移说明仍要求移除当前存在的纳音模块

位置：[migration.md:55](/Users/lzm0x219/Code/github.com/matharts/core/docs/migration.md:55)。

这里写着纳音表已合入 `sexagenary.rs`、公开 `matharts_core::nayin` 模块已删除，并指导调用方移除导入。当前 [lib.rs:18](/Users/lzm0x219/Code/github.com/matharts/core/src/lib.rs:18) 和 [lib.rs:34](/Users/lzm0x219/Code/github.com/matharts/core/src/lib.rs:34) 已公开 `nayin` 模块和 `Nayin` 类型；实际身份、名称及五行映射位于独立 `nayin.rs`，干支上的 `nayin()` 委托给身份查询。

建议：将旧删除过程标明为历史阶段，并更新当前迁移目标：`nayin()` 仍返回五行；`nayin_identity()` 返回新增身份；当前允许模块路径及根导出路径。无需改动代码 API。

验收：迁移说明中的当前导入示例可编译，且不再要求删除现有有效入口。

## 整体设计

### 领域边界

Core 的边界应是跨领域共享的具体语义，而不是按文件名或术数名称整块接纳、整块排除。阴阳、五行、合法干支、旬身份、纳音身份和卦形都适合有限值表示；关系查询应明确参照方向和采用规则。当前代码没有把固定关系自动解释为成局、成化、吉凶或排盘结果，这个边界应继续保持。

`God`、`Growth`、藏干、刑和天干推导已明确规则及使用限制。规则表在当前采用定义下通过独立预期测试，不等于已经完成所有体系的文献验证；它们跨体系准入证据的补充仍是领域研究工作，不是本次发现的算法错误，也不是应立即迁出整个文件的依据。

### 类型、不变量与错误

`SexagenaryCycle` 用私有字段排除非法配对，构造与反序列化都经校验；`Hexagram` 用两个合法三爻卦表示六爻值，不留下非法位模式。严格 `TryFrom` 与回绕 `from_index` 分工明确。`TrigramPosition`、`HexagramPosition` 避免裸整数爻位和越界查询。

`Option` 用于简单可选构造，`Result` 为外部输入提供错误分类；文本解析按格式、天干、地支、配对顺序报告，保留底层配对错误。现在没有必要引入一个覆盖所有领域的巨大错误枚举。

### 文件组织与公共接口

16 个源码文件规模可控。`lib.rs` 提供统一导出；按类型维护固定表和 Serde 实现，符合用户要求。`god.rs`、`growth.rs` 内为 `Stem` 增加方法，使规则实现与规则类型相邻；这是可以接受的组织方式，注意文档能从 `Stem` 找到入口即可。

`CyclicRing` 是有限周期操作契约，不是完整代数环；文档已有范围说明。`Nayin`、卦形没有为了统一接口而强行实现周期操作，避免了不自然的语义。保留现有名称及公开路径，不因审查追求形式统一而破坏调用方。

### 编码与验证

当前最需要改进的是“承诺、实现、测试”三者对齐。爻卦已有 JSON、模型 token、独立 index recorder、非法输入和反向变异检查；旧类型仅锁 JSON，不足以支撑相同强度的模型稳定承诺，见 F1。

有限域采用穷举和独立表作为主证据，配合极端偏移性质测试是合适的。现在无需为微小常数规模引入缓存、复杂索引或性能框架。若以后关注布局或序列化成本，应先用真实调用测量。

## 生产源码逐文件审查

“保留”表示本轮没有确认需要修复的该文件独立缺陷，不表示形式化证明或全部历史文化映射已经考据完毕。F1 是跨文件问题，在受影响行重复标注以免遗漏。

| 文件 | 细节设计审查 | 结论与建议 |
| --- | --- | --- |
| [lib.rs](/Users/lzm0x219/Code/github.com/matharts/core/src/lib.rs) | 无条件 `no_std`、禁用 unsafe；模块声明与根导出一致，错误及新身份均可访问。 | 保留。总览文字可随功能扩充提及旬、纳音和卦形，但不影响行为。 |
| [math.rs](/Users/lzm0x219/Code/github.com/matharts/core/src/math.rs) | 只组织周期基础设施及重导出，不夹带领域映射。 | 保留公开路径；无需为减少一个文件制造迁移。 |
| [math/ring.rs](/Users/lzm0x219/Code/github.com/matharts/core/src/math/ring.rs) | 非零周期、严格构造、回绕和有向距离职责分明；宽整数运算处理极端 `i32`；外部 trait 实现契约有示例及零周期检查。 | 保留。外部实现仍需履行 `index/from_index` 的互逆与范围契约，不能保证任意错误实现的数学性质。 |
| [primitive.rs](/Users/lzm0x219/Code/github.com/matharts/core/src/primitive.rs) | 双值类型、反转及谓词足够直接；当前判别值不能直接当爻位编码使用，卦模块已有显式转换。 | F1。`ALL` 可按确切遍历需求补充；不必为了外观一致增加周期 trait。 |
| [element.rs](/Users/lzm0x219/Code/github.com/matharts/core/src/element.rs) | 五行次序支持生克偏移，`relation_to` 从接收者指向目标；严格与取模构造分开，全部 25 对关系有独立预期。 | 两个枚举均受 F1 影响。算法与方法方向保留。 |
| [stem.rs](/Users/lzm0x219/Code/github.com/matharts/core/src/stem.rs) | 阴阳、五行、五合、冲及距离语义明确；加法处理有符号极值；中文解析严格。五合返回对应五行，不判成化。 | F1 的实际变异复现位置。现有 `Ord` 只表示工程次序，不能解释成强弱。 |
| [branch.rs](/Users/lzm0x219/Code/github.com/matharts/core/src/branch.rs) | 支属性、藏干与各关系有明确入口；刑不被错误当作全部对称关系；三合伙伴返回次序固定。`HiddenStems` 是允许自定义、重复和独立可选槽位的记录容器，不保证记录来自某地支的合法藏干表，也不承担权重规则。 | F1。体量较大但仍围绕地支；暂不拆成大量小文件。不要新增旺衰、合化判断混入固定查询。 |
| [god.rs](/Users/lzm0x219/Code/github.com/matharts/core/src/god.rs) | 十神由五行关系及同异阴阳推导；接收者是参照、参数是目标；100 个有序对有预期。 | F1。保留相对关系接口，不将枚举身份本身当独立绝对属性。 |
| [growth.rs](/Users/lzm0x219/Code/github.com/matharts/core/src/growth.rs) | 十二阶段身份与当前采用的起点、阴阳顺逆规则分明；120 个干支位置测试锁定规则。 | F1。需继续保留采用定义和流派限制，不能用测试通过代替规则普适性证明。 |
| [sexagenary.rs](/Users/lzm0x219/Code/github.com/matharts/core/src/sexagenary.rs) | 同奇偶检查构成合法性入口；60 周期和同余索引互逆；`RawGanzhi` 将解码汇入合法构造；文本不隐式 trim。纳音五行已委托身份。 | 保留。Serde 结构体模型也应补契约测试；旬首、缺位查询可委托 `Xun` 减少重复公式，属可选改进。 |
| [xun.rs](/Users/lzm0x219/Code/github.com/matharts/core/src/xun.rs) | 六个身份、固定全集；每旬十成员按周期产生；旬首和缺位仅解释集合，不推断实际空亡作用。 | F1。保留六旬周期及 `contains`。与干支侧查询有全量一致性测试。 |
| [nayin.rs](/Users/lzm0x219/Code/github.com/matharts/core/src/nayin.rs) | 三十身份、中文名、所属五行和两个干支的映射集中；严格索引拒绝越界；没有强行提供纳音循环算术。 | F1。文件边界合理；修正 F3 文档即可，无须再次合回干支文件。 |
| [stem_derivation.rs](/Users/lzm0x219/Code/github.com/matharts/core/src/stem_derivation.rs) | 年干/月支、日干/时支两个纯函数；输入已是领域值，不承担节气、日期或真太阳时换算。 | 保留用途命名；调用方负责先确定目标月支/时支。 |
| [trigram.rs](/Users/lzm0x219/Code/github.com/matharts/core/src/trigram.rs) | 八卦身份、三爻位置、底爻为低位的编码；数组、位、阴阳间显式转换；反转与变爻在有限域内；枚举 Serde 固定名称、index 和代码。 | 保留。内联 Serde 代码虽增加行数，但符合已确认的组织选择；不抽通用宏。 |
| [hexagram.rs](/Users/lzm0x219/Code/github.com/matharts/core/src/hexagram.rs) | 下卦占低三位、上卦占高三位；六爻位置强类型；互卦、反转、变爻由数组语义验证。Serde 区分可读对象与非可读序列输入，并检查缺失、重复、未知字段。 | 保留。暂不加入卦序、卦辞、先后天方位或占断策略。 |
| [error.rs](/Users/lzm0x219/Code/github.com/matharts/core/src/error.rs) | 索引、非法干支、文本格式/名称错误分层；`ParseError` 可扩展并保留 `source()`；不存储输入，不依赖堆分配。 | 保留。错误是输入失败事实，不带展示层修正或自动纠错策略。 |

## 测试与样本逐文件审查

| 文件 | 证据与覆盖 | 结论与建议 |
| --- | --- | --- |
| [contracts.rs](/Users/lzm0x219/Code/github.com/matharts/core/tests/contracts.rs) | 严格/回绕构造、周期边界、属性、25 对五行关系、合冲、伙伴顺序、极端加法、有向减法、旬及外部 trait 实现。 | 有限域公共契约覆盖合理。保留明确预期与泛型边界测试的分工。 |
| [rules.rs](/Users/lzm0x219/Code/github.com/matharts/core/tests/rules.rs) | 100 对十神、120 个长生位置、天干推导、30 对纳音、支关系表、藏干与五合。 | 独立表能捕捉实现方向错误；表的文化来源与适用范围仍由规则文档承担。 |
| [regressions.rs](/Users/lzm0x219/Code/github.com/matharts/core/tests/regressions.rs) | 极端偏移；120 个天干地支组合恰好接受 60 个；启用 Serde 时复核同一合法性。 | 保留，不仅做正向 roundtrip，拒绝路径也有覆盖。 |
| [encoding.rs](/Users/lzm0x219/Code/github.com/matharts/core/tests/encoding.rs) | 旧枚举 JSON、60 干支对象、藏干可空字段及非法输入。 | F1：JSON 无法观察 variant index、Serde 名称及全部模型类别；应补独立模型验收。 |
| [identities.rs](/Users/lzm0x219/Code/github.com/matharts/core/tests/identities.rs) | 30 纳音身份的名称/五行/干支对、六旬分区、越界、极端偏移、JSON 冻结编码。 | 计算与集合覆盖合理；`Xun/Nayin` 同样需要 F1 的模型检查。 |
| [text.rs](/Users/lzm0x219/Code/github.com/matharts/core/tests/text.rs) | 独立中文名、60 干支和全部120配对、拒绝别名空白、错误优先级/source、栈缓冲区和 writer 失败。 | 直接测公共文本行为，足以支持当前严格解析约定。 |
| [gua.rs](/Users/lzm0x219/Code/github.com/matharts/core/tests/gua.rs) | 全部64形态、全部u8输入、所有爻位查询/更新、变换及 const 构造；数组语义与位运算分开。 | 适合小有限域的穷举方法，未发现需新增另一套同构测试的理由。 |
| [gua_serde.rs](/Users/lzm0x219/Code/github.com/matharts/core/tests/gua_serde.rs) | 64 编码分别检查输出/旧输入；17 个 variant 的独立模型观察；对象/序列、非法输入、重复键及反向模型变异。 | 当前模型验收最完整，可借鉴测试方法覆盖旧类型。测试文件按跨类型协议组织，不与生产 Serde 内联要求冲突。 |
| [encoding-v2.json（当前样本）](/Users/lzm0x219/Code/github.com/matharts/core/tests/fixtures/encoding-v2.json) | 阴阳2、五行5、关系5、干10、支12、十神10、长生12、干支60、非法干支4及两种藏干对象。数组项无重复。 | 保持独立冻结数据，不随 Rust 重命名自动重生成；触发问题见 F2。 |
| [identities-v1.json](/Users/lzm0x219/Code/github.com/matharts/core/tests/fixtures/identities-v1.json) | 六旬6、纳音30，条目完整且无重复。 | 保留旧输入测试地位；不能只验证当前输出再写回此文件。 |
| [gua-v1.json](/Users/lzm0x219/Code/github.com/matharts/core/tests/fixtures/gua-v1.json) | 八卦8、六爻卦64、示例6、非法位输入各2、非法JSON值16、重复键文本2；各数组内部无重复。 | 重复键必须保留原始文本才能验证，当前分开存储正确。 |

## 工程、示例与静态文件逐文件审查

| 文件 | 审查结论 |
| --- | --- |
| [Cargo.toml](/Users/lzm0x219/Code/github.com/matharts/core/Cargo.toml) | Rust 2024 / 1.99、可选且关闭默认特性的 Serde 与实际 no_std 设计一致；JSON、性质测试和模型测试仅为开发依赖。`std = []` 已说明兼容用途，不是虚假的功能实现。 |
| [Cargo.lock](/Users/lzm0x219/Code/github.com/matharts/core/Cargo.lock) | 生成的依赖锁定文件，`--locked` 基线通过；本轮未做第三方依赖漏洞或许可证审计，不能扩大为供应链安全结论。 |
| [mise.toml](/Users/lzm0x219/Code/github.com/matharts/core/mise.toml) | Rust 1.99.0、hk 2.4.0 与本地命令和 CI 基线一致。 |
| [AGENTS.md](/Users/lzm0x219/Code/github.com/matharts/core/AGENTS.md) | 最终清单检查时出现并纳入审查；项目准入、不变量、历史记录边界和分级验证要求与当前设计一致。明确 `HiddenStems` 为可自定义容器，避免错误收紧输入；文档修改要求链接与空白检查。本轮未改动此文件。 |
| [hk.pkl](/Users/lzm0x219/Code/github.com/matharts/core/hk.pkl) | F2。其他设计合理：格式修复显式执行，pre-commit 不自动 stage；测试为 slow profile，并有 pre-push 测试入口。 |
| [ci.yml](/Users/lzm0x219/Code/github.com/matharts/core/.github/workflows/ci.yml) | debug/release × 四种 feature 组合、示例、裸机目标与文档检查齐全；只读仓库权限合理。任务串行可能增加耗时，但没有测量依据要求拆 jobs 或加缓存。 |
| [ganzhi.rs](/Users/lzm0x219/Code/github.com/matharts/core/examples/ganzhi.rs) | 展示输入→领域属性→周期→旬→纳音→错误的完整调用路径；普通断言在 release 仍执行。库 no_std 与示例用 std 输出的边界写明。局部中文展示函数是当前 API 范围的合理调用方适配。 |
| [.gitignore](/Users/lzm0x219/Code/github.com/matharts/core/.gitignore) | 忽略构建产物；保留了一些当前 Rust 工程用不到的模板规则，可择机精简，不构成运行缺陷。新增工具元数据是否入库应单独确定，不把当前未跟踪状态当代码问题。 |
| [.codegraph/.gitignore](/Users/lzm0x219/Code/github.com/matharts/core/.codegraph/.gitignore) | 本地索引资产的忽略配置，不进入库 API；数据库、socket 和日志不作为交付源码评价。审查中使用现有索引查询旬相关调用关系，没有创建或重建索引。 |
| [LICENSE](/Users/lzm0x219/Code/github.com/matharts/core/LICENSE) | MIT 声明与 Cargo 元数据一致；仅核对仓库声明一致性，不构成法律意见。 |
| [hero.svg](/Users/lzm0x219/Code/github.com/matharts/core/assets/readme/hero.svg) | 静态 README 图，源码未发现脚本或外部资源依赖；具有可访问名称。未做本轮像素渲染或视觉验收。 |
| [hero-mobile.svg](/Users/lzm0x219/Code/github.com/matharts/core/assets/readme/hero-mobile.svg) | 移动版静态图与桌面图分开，作为展示资源保留；同样只检查源码结构，不声称已验证全部客户端的响应式效果。 |

## 文档与设计探针逐文件审查

| 文件 | 定位、结论与维护建议 |
| --- | --- |
| [README.md](/Users/lzm0x219/Code/github.com/matharts/core/README.md) | 当前入口、示例与边界说明，包含新身份和卦形。应继续链接现行契约，避免把所有历史过程复制到首页。 |
| [CONTEXT.md](/Users/lzm0x219/Code/github.com/matharts/core/CONTEXT.md) | 领域词汇和边界导航；固定关系与应用判断的区别合理。当前只审查，不另行创造术语或扩大领域模型。 |
| [core-plan.md](/Users/lzm0x219/Code/github.com/matharts/core/docs/core-plan.md) | 当前准入及接口原则。F1 是实现/验收未完全兑现其编码要求；“全集”等规划条目应区分目标与现有能力，例如 `Primitive` 尚无 `ALL`。 |
| [rules.md](/Users/lzm0x219/Code/github.com/matharts/core/docs/rules.md) | 明确采用定义、应用限制与待补跨体系证据，是防止固定映射被误当普适推断的关键文档。保持逐规则补证，不笼统宣称全库文化正确性。 |
| [migration.md](/Users/lzm0x219/Code/github.com/matharts/core/docs/migration.md) | F3；同时应与 core-plan 对齐“JSON承诺”和“Serde模型承诺”的层次。不保证所有二进制字节格式，不等于模型可以任意改变。 |
| [current-code-review.md](/Users/lzm0x219/Code/github.com/matharts/core/docs/current-code-review.md) | 历史代码审查快照，已标历史；不以其旧行号、旧问题状态覆盖当前源码。本报告作为新一轮结论，不回写历史。 |
| [domain-model-review.md](/Users/lzm0x219/Code/github.com/matharts/core/docs/domain-model-review.md) | 历史领域审查和边界论证；当前决策以 core-plan、rules 和已确认实现为准。历史观点不自动构成本轮迁出代码的授权。 |
| [naming-review.md](/Users/lzm0x219/Code/github.com/matharts/core/docs/naming-review.md) | 已确认命名调整的历史依据；实际迁移信息由 migration 维护，避免再出现两个现行名称清单。 |
| [fix-status.md](/Users/lzm0x219/Code/github.com/matharts/core/docs/fix-status.md) | 阶段修复及验证记录；其中的历史测试数量应保留时间语境，不冒充当前测试总量。 |
| [gua-design.md](/Users/lzm0x219/Code/github.com/matharts/core/docs/gua-design.md) | 形态域范围、位约定、typed位置及编码契约与实现相符。保持“卦形”与“卦序/经文/解释”的边界。 |
| [gua-design-review.md](/Users/lzm0x219/Code/github.com/matharts/core/docs/gua-design-review.md) | 实施前的方案修订和验收依据；已转为历史记录，不将隔离探针当生产实现验证。 |
| [gua-implementation.md](/Users/lzm0x219/Code/github.com/matharts/core/docs/gua-implementation.md) | G1 实现与矩阵记录，生产验收入口清晰；内联 Serde 的当前组织应持续与记录一致。 |
| [gua-code-review.md](/Users/lzm0x219/Code/github.com/matharts/core/docs/gua-code-review.md) | 上一轮局部审查记录。其“未发现问题”仅针对该轮爻卦范围，不能覆盖本轮发现的旧类型与工程门禁问题。 |
| [gua-design-v1.json](/Users/lzm0x219/Code/github.com/matharts/core/docs/fixtures/gua-design-v1.json) | 实施前设计样本，供隔离探针；生产64组合冻结样本在 tests 中。保留两者角色区别，不把旧设计样本更新成第二份生产事实源。 |
| [探针 README.md](/Users/lzm0x219/Code/github.com/matharts/core/docs/checks/gua-serde/README.md) | 顶部已说明 G1 完成及生产测试位置，避免旧的“G1仍须”被误读为未实现。隔离 probe 的复跑命令明确。 |
| [探针 Cargo.toml](/Users/lzm0x219/Code/github.com/matharts/core/docs/checks/gua-serde/Cargo.toml) | 独立开发实验，不作为生产依赖或公开 crate 接口；保留实验隔离即可。 |
| [探针 Cargo.lock](/Users/lzm0x219/Code/github.com/matharts/core/docs/checks/gua-serde/Cargo.lock) | 锁定历史实验依赖，用于复现而非为根 crate 制造第二套版本决策。 |
| [探针 contract.rs](/Users/lzm0x219/Code/github.com/matharts/core/docs/checks/gua-serde/contract.rs) | 最小复制类型验证 Serde 策略；现在真实生产类型已有对应测试。保留为历史实验，不用它替代生产测试，也不要求同步复制每次业务修改。 |
| [探针 .gitignore](/Users/lzm0x219/Code/github.com/matharts/core/docs/checks/gua-serde/.gitignore) | 独立构建产物的忽略配置，职责合理。 |

## 可选改进，不作为缺陷

1. **旬查询归一处实现。** `SexagenaryCycle::xun_start_branch/void_branches` 与 `Xun::leader/void_branches` 目前结果一致，但维护两个公式。可让干支便利入口委托 `Xun`，保留所有公共 API；注意调用方向，不形成循环递归。现有六旬/六十干支穷举足以验收。
2. **按实际需求补展示接口。** 部分类型用 `Display/FromStr`，部分只有 `name()`，阴阳五行在示例里转换中文。这不是当前行为错误；有真实统一展示需求时再决定 API，不为了整齐一次性增加全部解析器。
3. **文档区分当前规范与过程记录。** 现有历史标记已有作用；未来增加一处简短的当前文档导航即可，不需要重写历史报告或持续同步其旧测试数字。

不建议的改动：恢复独立生产 `gua_serde.rs`、为消除少量重复建立通用序列化框架、把所有有限类型强行塞入 `CyclicRing`、重命名已明确保留的核心类型、在 Core 混入日期上下文或应用效果判断。

## 本轮验证与限制

以下命令在原仓库执行，均退出 `0`；通过 `rtk proxy mise exec --` 调用：

| 检查 | 结果 |
| --- | --- |
| `cargo test --all-features --locked` | 47 个集成测试 + 14 个文档测试通过，其中 1 个文档测试为 compile-fail。 |
| `cargo test --no-default-features --locked` | 36 个集成测试 + 14 个文档测试通过。Serde 测试按 feature 正常关闭。 |
| `cargo clippy --all-targets --all-features --locked -- -D warnings` | 通过。 |
| `cargo check --target thumbv7m-none-eabi --no-default-features --locked` | 通过。 |
| `cargo check --target thumbv7m-none-eabi --no-default-features --features serde --locked` | 通过。 |

另执行 F1 隔离变异实验及 F2 hook 计划检查。JSON 样本已解析并核对各数组的条目数与重复情况；实际语义由对应测试验证。CodeGraph 调用关系仅作定位辅助，源码与实际测试是结论依据。

本轮没有重新运行完整八组合 debug/release CI 矩阵、远端 CI、发布流程或全部历史探针；不把此前验证记录当作本轮新执行结果。没有验证所有第三方序列化格式的字节兼容、古籍原文的逐条校勘、其他语言实现一致性或 SVG 的实际视觉表现。

Jev 对同一批已收集证据进行了有界复核：接受 F1/F2 为 P2、F3 为 P3（置信度 0.94），支持保留现有架构并补验收（置信度 1.00）。它没有独立运行测试或审阅仓库，结论不能替代上述复现证据。

建议执行顺序：F1 编码模型与验收 → F2 本地测试触发 → F3 迁移文档；可选改进另行安排。
