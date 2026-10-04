# 爻、八卦与六十四卦：领域设计与准入评估

> 历史记录：本文反映兼容层清理之前的设计或检查状态，不代表当前接口、编码、特性矩阵或验证结果。当前契约见 [API 与编码变更](migration.md)，本轮结果见 [全库清理记录](breaking-cleanup.md)。

日期：2026-10-04。状态：G1 已实现并完成本地验收，见[实现与验证记录](gua-implementation.md)。本文保留模型边界、来源对照和验收契约；第9节是实施前的设计及修订实验历史，不能替代生产验收记录。

## 1. 建议结论

G1 向 Core 加入八卦与六爻卦的**纯结构层**：复用现有阴阳值，表达三个／六个有序爻、上下卦组合、有界爻位及明确的结构变换。无需起卦时间、流派或命盘上下文即可确定这些结果。

新增类型为 `Trigram`、`Hexagram`、`TrigramPosition`、`HexagramPosition`；不另造与 `Primitive` 重复的二值 `Yao`。静态结构与起卦所得老少阴阳、动爻集合分别归属。不会因为都含“爻”字就把两类信息合并。

本轮核对支持八卦形状、上下卦和六爻结构作为共同数据的建议；这仍是有证据范围的准入提案，不声称所有数术都使用卦，也不声称已校勘全部流派。将纯结构留在 Core，可以避免梅花、六爻等未来消费库各自定义不兼容的阴阳与上下卦；实际下游调用尚未接入。

暂不创建新 crate、泛型代数框架、规则注册器或卦序 trait。若后续应用出现同名但结构定义不同的对象，应先区分语义，不通过增加隐含配置改变这里的值。

## 2. 来源与准入单位

以下均为原典的在线转录；没有完成刻本影印校勘。资料用来核对术语与结构，不把占断解释视为本库计算保证。

| 审查单位 | 本轮核对 | 建议归属与证据限制 |
| --- | --- | --- |
| 阴阳与六爻结构 | [《说卦》第一、二章](https://zh.wikisource.org/wiki/易傳/說卦#第二章)描述阴阳及六画、六位 | 支持结构层；不据此引入蓍法或爻辞解释 |
| 八卦的三爻形状 | [《梅花易数》卷一《八卦象例》](https://zh.wikisource.org/wiki/梅花易數/卷一#八卦象例)逐一描述八个形状 | 记录为8项固定预期；属于本轮准入建议 |
| 上下卦与爻位 | [《周易·泰》](https://zh.wikisource.org/wiki/周易/泰)为乾下坤上，[《周易·否》](https://zh.wikisource.org/wiki/周易/否)为坤下乾上；两篇列出完整六爻 | 确认参数方向与非对称性，不能交换两卦而保持身份 |
| 六爻应用中的结构 | [《卜筮正宗》装卦表](https://www.quanxue.cn/qt_mingxiang/boshi/boshi06.html)列乾、姤、屯等六爻形状，并附加纳支、六亲、世应 | 与结构层可对接；本轮只交叉核对所选样本，不称64卦逐项跨书对照完成 |
| 某套八卦编号 | [《梅花易数》卷一《周易卦数》](https://zh.wikisource.org/wiki/梅花易數/卷一#周易卦數)采用乾一至坤八 | 保留为显式具名映射候选，不作为卦身份或存储编号 |
| 单爻阴阳交换 | [同卷《爻以六除》](https://zh.wikisource.org/wiki/梅花易數/卷一#爻以六除)区分选动爻的方法及选定后的阴阳变化 | 交换操作可按显式爻位定义；如何选择爻位属于应用规则 |
| 互卦 | [同卷《互卦起例》](https://zh.wikisource.org/wiki/梅花易數/卷一#互卦起例)除中间四爻取卦，还列有针对乾坤、取其变卦的处理 | 不新增一个无条件的 `nuclear()` 并声称覆盖该规则；结构取爻与应用特判须另分契约 |
| 方位、五行、卦序及类象 | 《说卦》及《梅花易数》分别存在方位、五行与类象文本 | 固定映射逐项候选，不因可查表就自动认定为卦的无条件属性 |
| 纳甲、世应、六亲、体用、吉凶 | 装卦表和梅花应用章节含各自附加信息 | 放对应领域库，不能由静态卦类型暗中给出 |

两个具体消费场景均只接收已确定的卦：梅花侧传入上下卦及选定爻位，Core 返回改变该位后的结构；六爻侧传入六个爻的阴阳，Core 返回相同结构及上下卦。起卦方式、时间、世应、体用和解释不进入这些输入。

## 3. 值模型

| 类型 | 有效值与不变量 | 拟提供能力 |
| --- | --- | --- |
| 已有 `Primitive` | 只有阴阳，不更改名称、数值或 Serde 代码 | 直接作为爻数组的元素 |
| `Trigram` | 8种不同三爻结构；具名 `Qian/Dui/Li/Zhen/Xun/Kan/Gen/Kun` | 名称、三爻构造与分解、位编码、有界位置查询和修改、结构变换 |
| `Hexagram` | 两个合法八卦组成64种不同六爻结构 | 上下卦构造与分解、六爻构造与分解、位编码、有界位置查询和修改、结构变换 |
| `TrigramPosition` | `First/Second/Third`，从下向上 | `index()` 返回0至2；`TryFrom<u8>` 严格拒绝3至255 |
| `HexagramPosition` | `First/Second/Third/Fourth/Fifth/Sixth`，从初爻向上爻 | `index()` 返回0至5；`TryFrom<u8>` 严格拒绝6至255 |

用不同的爻位类型，让第四爻不能误传给三爻卦；它们也不自动循环回绕。显示层可以把位置转换为传统爻位名称，但不把零基索引直接呈现为传统序数。

`Hexagram` 建议采用两个私有 `Trigram` 字段 `lower`、`upper`。所有8×8种组合都合法，不照搬干支配对的阴阳校验。两个小枚举足以表达不变量，暂不为节省一个字节引入另一套私有位域表示。

类型均为小型、不可变、可复制值，支持 `Debug/Clone/Copy/PartialEq/Eq/Hash`；Serde 继续可选。没有自然默认卦，不实现 `Default`；没有本轮选定的语义全序，不实现 `Ord` 或 `CyclicRing`。

`Trigram::Xun` 表示巽，已有根类型 `Xun` 表示旬；保留枚举路径限定，不把八个变体平铺导出到 crate 根。

## 4. 位序、数组顺序与身份编码

爻数组统一**从下到上**。位编码约定最低位代表最下爻，阳为1、阴为0。普通二进制字面量仍从高位写到低位，不能按其书写方向误读爻数组。

这是本库建议采用的工程编码，不是古籍中的卦序。

| 三爻卦 | 自下而上的爻数组 | bits（二进制） | bits（十进制） |
| --- | --- | --- | --- |
| 坤 | 阴、阴、阴 | `000` | 0 |
| 震 | 阳、阴、阴 | `001` | 1 |
| 坎 | 阴、阳、阴 | `010` | 2 |
| 兑 | 阳、阳、阴 | `011` | 3 |
| 艮 | 阴、阴、阳 | `100` | 4 |
| 离 | 阳、阴、阳 | `101` | 5 |
| 巽 | 阴、阳、阳 | `110` | 6 |
| 乾 | 阳、阳、阳 | `111` | 7 |

当前 `Primitive` 为 `Yang = 0`、`Yin = 1`。实现转换必须显式匹配阴阳；不能用 `primitive as u8` 写入卦位，也不能改变已有阴阳编码来适应新模型。

六爻编码为 `lower.bits() | (upper.bits() << 3)`。因此泰为7、否为56；这两个数字均不是它们的文王卦序。公开入口称 `bits()` / `try_from_bits()`，不提供含糊的 `index()` 或 `from_index()`；两个卦类型也不实现含义不明的 `TryFrom<u8>`。

`try_from_bits` 对三爻只接受0至7，对六爻只接受0至63；越界返回现有 `InvalidIndex { index: bits, upper_bound: 8或64 }`，文档说明这里的索引是结构编码。禁止通过掩码静默接受高位。`ALL` 按 bits 升序枚举，名称须在文档中注明这是技术遍历顺序。

## 5. G1 公共 API

下面的接口已实现。`from_lines` 的固定长度数组和 `from_trigrams` 的合法枚举输入保证构造不会失败；纯操作实现为 `const fn`。两个爻位类型另有按自下而上顺序排列的 `ALL`。

| 类型 | 拟定入口 |
| --- | --- |
| `Trigram` | `ALL: [Self; 8]`、`name() -> &'static str`、`from_lines([Primitive; 3]) -> Self`、`lines() -> [Primitive; 3]` |
| `Trigram` | `bits() -> u8`、`try_from_bits(u8) -> Result<Self, InvalidIndex>` |
| `Trigram` | `line(TrigramPosition) -> Primitive`、`with_line(TrigramPosition, Primitive) -> Self`、`toggle_line(TrigramPosition) -> Self` |
| `Trigram` | `complement() -> Self`、`reverse_lines() -> Self` |
| `Hexagram` | `ALL: [Self; 64]`、`from_trigrams(lower: Trigram, upper: Trigram) -> Self`、`lower() -> Trigram`、`upper() -> Trigram` |
| `Hexagram` | `from_lines([Primitive; 6]) -> Self`、`lines() -> [Primitive; 6]` |
| `Hexagram` | `bits() -> u8`、`try_from_bits(u8) -> Result<Self, InvalidIndex>` |
| `Hexagram` | `line(HexagramPosition) -> Primitive`、`with_line(HexagramPosition, Primitive) -> Self`、`toggle_line(HexagramPosition) -> Self` |
| `Hexagram` | `complement() -> Self`、`reverse_lines() -> Self` |

`complement()` 逐爻取反；`reverse_lines()` 倒置完整爻序，两者都是本方案明确给定的结构操作，不自动推导占断意义。六爻倒序不能只交换 `lower` 与 `upper`，还必须分别倒置两卦内部的三爻顺序。`toggle_line()` 只改变调用方明确提供的一个位置；没有批量接口，也没有隐含选动爻规则。

调用示例（已纳入生产 rustdoc 验证）：

```rust
use matharts_core::{Hexagram, HexagramPosition, Primitive, Trigram};

let tai = Hexagram::from_trigrams(Trigram::Qian, Trigram::Kun);
assert_eq!(tai.lines(), [
    Primitive::Yang, Primitive::Yang, Primitive::Yang,
    Primitive::Yin, Primitive::Yin, Primitive::Yin,
]);
assert_eq!(tai.bits(), 7);
let pi = Hexagram::from_trigrams(Trigram::Kun, Trigram::Qian);
assert_eq!(tai.reverse_lines(), pi);

let qian = Hexagram::from_trigrams(Trigram::Qian, Trigram::Qian);
let gou = qian.toggle_line(HexagramPosition::First);
assert_eq!(gou.lower(), Trigram::Xun);
assert_eq!(gou.upper(), Trigram::Qian);
assert_eq!(gou.bits(), 62);
```

边界反例：泰初爻与上爻不同；`reverse_lines()` 对震三爻得到艮，而阴阳取反得到巽。起卦所得6、7、8、9不能直接传入爻位或当作结构编码，必须由上层解释。

第一批 `Hexagram` 已可表示全部64个结构，但不提供六十四卦中文名称、文王序号或64个命名常量。名称与序号进入下一批独立表核验，避免用少量样本冒充完整名称库。方案样本中的“泰、否、姤、屯”是验收标签，不是已承诺的 `name()` API。

## 6. 序列化与兼容

### 6.1 稳定 Serde 数据模型

四个新增类型的输出模型固定如下。表中名称是 Serde 名称，不能随 Rust 重命名而变化；variant index 是显式编码契约，不能依赖未经约束的源码声明顺序。序列化可以派生，但必须用模型测试锁定这些值；反序列化按6.2及下文的输入入口显式处理。

| 类型 | 模型类别及稳定名称 | variant 代码与索引／字段序列 |
| --- | --- | --- |
| `Trigram` | `unit_variant`，枚举名 `Trigram` | `Kun=0, Zhen=1, Kan=2, Dui=3, Gen=4, Li=5, Xun=6, Qian=7` |
| `TrigramPosition` | `unit_variant`，枚举名 `TrigramPosition` | `First=0, Second=1, Third=2` |
| `HexagramPosition` | `unit_variant`，枚举名 `HexagramPosition` | `First=0, Second=1, Third=2, Fourth=3, Fifth=4, Sixth=5` |
| `Hexagram` | `struct`，结构体名 `Hexagram`，长度2 | 依次序列化 `lower: Trigram`、`upper: Trigram` |

枚举在人类可读输入中通过 `deserialize_str` 严格匹配代码；非人类可读输入通过 `deserialize_enum` 使用表中名称及 variant 表，接受合法代码或索引标识，拒绝越界标识和非 unit payload。JSON 中枚举只接受上述字符串代码，不接受中文显示名、别名、数字或 `{"Qian":null}` 形式；后者是常规枚举派生可能额外接受的形状，因此输入也需显式实现。`Hexagram` 使用 `deserialize_struct` 和固定名称、字段表。两模式的输出模型始终按上表固定，输入入口的差异不改变输出模型承诺。

不能把 `unit_variant` 换成 `string`，也不能把 `struct` 换成 `map`，即使 JSON 输出相同。模型契约与 JSON 表示均须检查；不据此承诺第三方二进制格式的字节兼容。依据为 [Serde 数据模型](https://serde.rs/data-model.html)与本项目总体规划的 S2 约束。

### 6.2 JSON 与非人类可读输入

`Hexagram` 使用两个明确的卦身份字段：

```json
{"lower":"Qian","upper":"Kun"}
```

这表示泰。JSON 缺字段、未知卦代码、重复字段、非对象、数字及额外字段均拒绝；包括元素合法且长度正好为2的 `["Qian","Kun"]`。不能悄悄忽略 `moving_line` 等应用信息，再将输入当成已经完整保存的占卦记录。严格字段规则只适用于新增类型，不修改旧类型反序列化约定。

`Hexagram` 需要显式 Visitor，不能仅用 `derive(Deserialize)` 加 `deny_unknown_fields`。调用 `deserialize_struct("Hexagram", &["lower", "upper"], visitor)` 前读取 `Deserializer::is_human_readable()`，并采用以下规则：

- 人类可读模式（包括 serde_json 的 `from_str` 和 `from_value`）：`visit_seq` 一律报错，只接受字段映射。
- 非人类可读模式：允许结构体以恰好两个元素的序列传给 `visit_seq`，顺序固定为 `lower`、`upper`；缺项或多项报错。额外元素探测使用 `Trigram`，不依赖非自描述格式未必支持的 `IgnoredAny`。
- 两种模式的 `visit_map` 均检查未知、重复和缺失字段；输入键顺序可以颠倒。是否支持 map 由具体格式决定。序列化在两种模式下均使用6.1规定的同一模型。

这是一项按 Serde 人类可读标志区分的策略，适用于所有遵守该标志的格式，不尝试识别格式品牌。JSON 对象键的书写顺序不作字节级承诺；Serde 序列化调用的字段顺序固定。格式是否适配及其版本迁移须另行验收。

不更改已有 `Primitive`、`Xun`、`Nayin`、干支值及现有两个编码样本文件。G1 已建立独立固定验收样本 [gua-v1.json](../tests/fixtures/gua-v1.json)，尚未发布；原 [gua-design-v1.json](fixtures/gua-design-v1.json) 继续明确标为历史设计样本。

## 7. 实施位置与顺序

| 阶段 | 交付 | 进入下一阶段所需证据 |
| --- | --- | --- |
| G0，已完成 | 本设计、术语表、8卦形状与6个六爻案例设计样本；公式穷举演算 | 检查位序、上下卦、Primitive转换及规则分界；已补齐两项 Serde 设计缺口 |
| G1，已实现并本地验证 | `src/trigram.rs` 与三爻位、`src/hexagram.rs` 与六爻位；根导出四个类型；`tests/gua.rs`、`tests/gua_serde.rs` 及独立编码样本 | 结构全集、非法输入、编码与工程验证全部通过，见[验收记录](gua-implementation.md) |
| G2，固定映射 | 64卦名称、文王卦序；先后天编号与方位另列具名映射方案 | 各表独立核验、全量样本、明确名称异写与采用版本；分别判断 Core 或易学基础层归属 |
| G3，领域消费 | 梅花／六爻等应用接入，定义起卦、动爻记录、互卦特判、体用或装卦规则 | 有真实调用、版本依据和独立验收；不塞入静态卦值 |

模块内放对应爻位类型，避免新增只有几行内容的通用位置框架。`Primitive` 保持原文件和 API。第一批不增加生产依赖，保持 `no_std`、无堆分配与可选 Serde；模型测试工具仅进入开发依赖。本轮隔离探针的依赖不会加入生产 crate。

## 8. G1 验收契约

1. **固定表**：8卦逐项断言名称、代码、三爻数组及 bits；独立列出64个上下卦组合的编码样本。生产实现不得生成这些固定预期。
2. **构造与拆分**：8组三爻、64组六爻及全部8×8上下卦组合往返；`ALL` 无缺项、无重复。泰与否必须用相反上下卦分别断言。
3. **严格输入**：两个 `try_from_bits` 分别枚举所有256个输入，明确合法数为8和64；两种爻位各枚举全部256个输入。不要用错误回绕把8变成坤、把64变成坤卦。
4. **查询与修改**：8×3和64×6个爻位分别核对查询、设置为阴和设置为阳；未指定的其他位置保持。反转同一爻两次还原，修改不同爻的顺序不影响结果。
5. **两类结构反转**：每个卦的逐爻取反、完整倒序均与数组参考计算比较；两次操作各自还原。含震／艮／巽的非对称案例，避免只拿乾坤、泰否测试而漏掉内部三爻方向错误。
6. **JSON**：8个三爻代码、64个上下卦对象、3+6个位置代码分别检查编码和解码。全部非法值分别执行 `from_str` 与 `from_value`，必须包含长度为2且元素合法的数组。原始重复键单列字符串样本并实际执行 `from_str`，不能先装进会丢失重复键的 JSON 对象。正例同时接受相反键顺序。样本长度必须断言，禁止静默跳过。
7. **Serde 模型**：独立断言6.1中的类别、名称、全部17个 variant 的代码及索引、结构体长度与字段序列；分别检查输出和固定输入，不能只做往返。对 `unit_variant → string`、类型名变化、variant 索引变化、`struct → map`、字段顺序变化做反向验证，模型检查必须报错，即使 JSON 相同。token 工具若不观察 variant index，须补专用 serializer 断言。
8. **输入模式**：人类可读序列拒绝，非人类可读的双元素结构体序列成功，缺项、多项失败；两模式均检查 map 的缺失、重复、未知字段。检查枚举非 unit payload 与越界标识。使用模型 token／测试适配器验证分支，不将结果宣称为所有二进制格式兼容。
9. **兼容与工程**：原测试和编码样本继续通过；执行仓库的8组 debug/release 特性矩阵、两组裸机编译、严格 Clippy/rustdoc、fmt。此时才把目标调用示例写进可执行 rustdoc。

本轮设计演算不能替代上述 Rust 实现验收。互卦、文王序、方位、纳甲等未进入 G1，因此不能由 G1 全绿推出它们已实现或已通过准入。

## 9. 本轮检查记录

### 修订前的 G0 演算

本轮实际运行设计样本与公式的独立数组参考演算，结果通过：8个三爻固定形状、6个六爻案例、全部64个上下卦组合、72种结构的构造与反转、408个单爻翻转、816个爻位赋值，以及9个非法 JSON 对象／形状案例。8位整数输入的编码合法数量分别核对为8和64；这不是未来 Rust 构造器的执行结果。另检查11个本地文档链接均可解析，`git diff --check` 通过。

这些是设计公式与样本的一致性检查，未编译任何新增 Rust API，没有把它们写成生产测试通过记录。G1 的独立64组合编码样本、反序列化重复键与完整特性矩阵仍须在实现时完成。

Jev 对已收集的源码与文献证据支持“复用阴阳、提出 Core 纯结构层、映射与应用规则分开”的建议，并根据上述实测结果判定设计阶段可交付；该判断不是文献权威裁定，也不代替实现测试。

### R1／R2 修订后的隔离验证

按 [审查记录](gua-design-review.md) 补全第6节数据模型和第8节验收契约；设计样本现有16个非法 JSON 值及2个原始重复键字符串。新增 [可重跑探针与命令](checks/gua-serde/README.md)，使用隔离开发依赖验证输入策略，未引入生产爻卦 API。

实测四组探针全部通过：6个正例、16个非法值的双入口拒绝、2个原始重复键；17个 variant 的名称／索引／代码和固定输入；结构体模型、两模式输入分支；6类模型变化的反向检查。探针的 `cargo fmt --check`、严格 Clippy 通过。JSON 枚举输入也拒绝对象式 unit variant，避免常规派生额外接受 `{"Qian":null}`。

这些结果证明修订策略在隔离实验中可行，不是 G1 生产实现通过。完整64组合编码样本、生产类型的上述检查、实际 no_std 与完整特性矩阵仍留在 G1；未承诺具体二进制格式兼容。修订期间根 `Cargo.toml`、`Cargo.lock`、`src/lib.rs` 和现有两个冻结编码样本保持不变。
