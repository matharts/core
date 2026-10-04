<picture>
  <source media="(max-width: 600px)" srcset="assets/readme/hero-mobile.svg" />
  <img src="assets/readme/hero.svg" width="100%" alt="MathArts Core：阴阳、五行、干支的 Rust 基础库。十天干与十二地支同步步进，六十步回到甲子。" />
</picture>

# MathArts Core

阴阳、五行与干支的 Rust 基础库。用可校验的类型、循环步进和固定关系查询，构建数术项目中共用的基础模型。

[运行示例](#运行完整示例) · [本地接入](#本地接入) · [规则边界](#规则边界) · [开发验证](#开发验证)

## 一个甲子的循环

从甲子走到乙丑，六十步回到起点。构造干支时，库会检查天干与地支的阴阳配对：

```rust
use matharts_core::{Branch, CyclicRing, SexagenaryCycle, Stem};

fn main() {
    let jia_zi = SexagenaryCycle::new(Stem::Jia, Branch::Zi).unwrap();
    let next = jia_zi.offset(1);

    assert_eq!(
        (next.stem(), next.branch()),
        (Stem::Yi, Branch::Chou),
    );
    assert_eq!(jia_zi.offset(60), jia_zi);
    assert!(SexagenaryCycle::new(Stem::Jia, Branch::Chou).is_none());
}
```

## 旬与纳音身份

已确定的干支通过 `xun()`、`nayin()` 查询所属旬及纳音身份，再由身份值查询成员和五行：

```rust
use matharts_core::{Branch, Element, Nayin, SexagenaryCycle, Stem, Xun};

let value = SexagenaryCycle::new(Stem::Jia, Branch::Zi).unwrap();
assert_eq!(value.xun(), Xun::JiaZi);
assert_eq!(value.xun().members().len(), 10);
assert_eq!(value.xun().void_branches(), (Branch::Xu, Branch::Hai));
assert_eq!(value.nayin(), Nayin::HaiZhongJin);
assert_eq!(value.nayin().name(), "海中金");
assert_eq!(value.nayin().element(), Element::Metal);
```

`Xun` 是六十甲子的六旬，不是民用月份的上中下旬。`Nayin` 区分海中金、剑锋金等身份，其索引不提供循环操作。两类都支持严格 `TryFrom<u8>` 和可选 Serde；名称异写、稳定编码与证据边界见[规则文档](docs/rules.md#旬与纳音身份)。

## 五合与六合配对

`FiveCombination` 表示五种天干五合组，`SixCombination` 表示六种地支六合组。单值查询归属，完整配对识别则拒绝重复和混组：

```rust
use matharts_core::{Branch, Element, FiveCombination, SixCombination, Stem};

let group = Stem::Jia.five_combination();
assert_eq!(group, FiveCombination::JiaJi);
assert_eq!(group.members(), [Stem::Jia, Stem::Ji]);
assert_eq!(group.partner_of(Stem::Jia), Some(Stem::Ji));
assert_eq!(group.partner_of(Stem::Yi), None);
assert_eq!(group.element(), Element::Earth);
assert_eq!(FiveCombination::from_stems([Stem::Ji, Stem::Jia]), Some(group));
assert_eq!(FiveCombination::from_stems([Stem::Jia; 2]), None);
assert_eq!(Branch::Hai.six_combination(), SixCombination::YinHai);
assert_eq!(SixCombination::from_branches([Branch::Hai, Branch::Yin]), Some(SixCombination::YinHai));
```

两种类型分别归入天干 `stem` 与地支 `branch` 模块，也在 crate 根导出。成员按各自索引升序，身份索引严格校验且不回绕；五合五行只是固定对应，六合不提供合化五行。旧伙伴方法移除，迁移路径见[迁移说明](docs/migration.md)，来源、消费对照及限制见[配对关系](docs/pair-groups.md)。

## 六冲、六害与六破配对

`SixClash`、`SixHarm`、`SixBreak` 均归入地支 `branch` 模块并在 crate 根导出。各类型独立维护六种配对，允许同一输入同时具有多种关系：

```rust
use matharts_core::{Branch, SixBreak, SixClash, SixCombination, SixHarm};

assert_eq!(Branch::Zi.six_clash(), SixClash::ZiWu);
assert_eq!(Branch::Zi.six_harm().partner_of(Branch::Zi), Some(Branch::Wei));
assert_eq!(SixHarm::from_branches([Branch::Wei, Branch::Zi]), Some(SixHarm::ZiWei));
assert_eq!(SixClash::from_branches([Branch::Zi; 2]), None);
assert_eq!(SixBreak::from_branches([Branch::Si, Branch::Shen]), Some(SixBreak::SiShen));
assert_eq!(SixCombination::from_branches([Branch::Si, Branch::Shen]), Some(SixCombination::SiShen));
```

成员按地支索引升序，身份索引严格校验。原 `is_clashing_with`、`is_harming`、`is_breaking` 委托类型识别；刑仍保留方向。六破限定采用《六壬大全》表，跨体系复用证据待补；来源、差异及应用边界见[地支配对](docs/branch-pairs.md)。

## 地支完整关系组

`ThreeCombination` 与 `ThreeMeeting` 同样归入 `branch` 模块，成员表、查询及 Serde 均在 `branch.rs` 内按各类型维护。

`ThreeCombination` 与 `ThreeMeeting` 分别表示四种三合、四种三会（方合）身份。单支可查询归属；三个支可识别完整集合：

```rust
use matharts_core::{Branch, Element, ThreeCombination, ThreeMeeting};

let group = Branch::Shen.three_combination();
assert_eq!(group, ThreeCombination::ShenZiChen);
assert_eq!(group.members(), [Branch::Zi, Branch::Chen, Branch::Shen]);
assert_eq!(group.element(), Element::Water);
assert_eq!(
    ThreeCombination::from_branches([Branch::Shen, Branch::Chen, Branch::Zi]),
    Some(group),
);
assert_eq!(ThreeCombination::from_branches([Branch::Zi; 3]), None);
assert_eq!(Branch::Hai.three_meeting(), ThreeMeeting::HaiZiChou);
```

成员数组按地支索引升序；冬方返回子、丑、亥。`from_branches()` 忽略输入排列并拒绝重复、混组，仅识别集合；应用层保留三传等原始顺序并判断具体条件。完整组替代旧伙伴元组入口。来源、编码和验收见[完整关系组](docs/branch-groups.md)。

## 八卦与六爻结构

`Trigram` 表示三爻卦，`Hexagram` 表示上下卦组成的六爻结构。数组从下到上；位编码最低位代表最下爻，阳为1、阴为0：

```rust
use matharts_core::{Hexagram, HexagramPosition, Trigram};

let tai = Hexagram::from_trigrams(Trigram::Qian, Trigram::Kun);
assert_eq!(tai.bits(), 7);
assert_eq!(tai.reverse_lines().bits(), 56);
let qian = Hexagram::from_trigrams(Trigram::Qian, Trigram::Qian);
let gou = qian.toggle_line(HexagramPosition::First);
assert_eq!((gou.lower(), gou.upper()), (Trigram::Xun, Trigram::Qian));
```

`try_from_bits()` 严格拒绝越界编码；两种爻位类型各自提供严格 `TryFrom<u8>`。`complement()` 交换阴阳，`reverse_lines()` 倒置完整爻序。位编码不代表传统卦序，`Trigram::Xun` 是巽卦，根类型 `Xun` 是旬。

可选 Serde 使用固定枚举模型和 `{"lower":"Qian","upper":"Kun"}` 结构体表示。JSON 只接受对象，拒绝数组、未知／重复／缺失字段；枚举只接受固定代码字符串。模型契约见[爻卦方案](docs/gua-design.md#6-序列化与兼容)，固定样本见 [gua-v1.json](tests/fixtures/gua-v1.json)。本批不提供六十四卦名称、文王序、动爻选择或占断规则。

## 中文文本解析与显示

`Stem`、`Branch`、`SexagenaryCycle` 支持中文 `Display` 和严格 `FromStr`：

```rust
use matharts_core::{Branch, InvalidGanzhi, ParseError, SexagenaryCycle, Stem};

assert_eq!("甲".parse::<Stem>(), Ok(Stem::Jia));
assert_eq!("子".parse::<Branch>(), Ok(Branch::Zi));
let value: SexagenaryCycle = "甲子".parse().unwrap();
assert_eq!(value.to_string(), "甲子");
assert_eq!(
    "甲丑".parse::<SexagenaryCycle>(),
    Err(ParseError::InvalidGanzhi(InvalidGanzhi)),
);
```

只接受精确中文名，不裁剪空白，不接受拼音、数字或其他别名。干支解析依次检查两个字符的格式、天干名、地支名及阴阳配对，错误由 `ParseError` 区分。解析不分配堆内存；`Display` 可写入 `core::fmt::Write`，上例的 `to_string()` 是应用侧的字符串分配。

中文文本不替代机器编码：Serde 仍使用 `"Jia"`、`"Zi"` 和 `{"stem":"Jia","branch":"Zi"}` 等原有表示。其他领域类型暂不提供这组文本接口。

## 运行完整示例

[examples/ganzhi.rs](examples/ganzhi.rs) 串起一条完整调用流程：严格解析“甲子”，查询阴阳五行、五合、六合、六冲、六害及采用表六破配对、完整三合与三会组，循环步进，查询所属旬与旬内缺位地支，再输出纳音身份及其五行。示例也演示格式、名称和阴阳配对错误的分类处理。

在 Core 仓库根目录运行：

```sh
mise exec -- cargo run --example ganzhi --locked
mise exec -- cargo run --example ganzhi --no-default-features --locked
```

示例使用宿主程序的标准输出；第二条命令验证关闭库默认特性时的接入，不是裸机程序。输出展示固定属性与关系，不作吉凶判断；具体应用条件见[规则边界](#规则边界)。

## 本地接入

下面通过本地路径依赖运行示例。先安装 Rust：最低支持版本见 [Cargo.toml](Cargo.toml) 的 `rust-version`，开发工具链版本见 [mise.toml](mise.toml)。

### 1. 准备项目

获取 Core 源码，在同一父目录创建示例应用：

```sh
git clone https://github.com/matharts/core.git
cargo new core-demo
cd core-demo
```

### 2. 添加依赖

在应用的 `Cargo.toml` 中找到 `[dependencies]`，添加：

```toml
matharts-core = { path = "../core", default-features = false }
```

### 3. 运行示例

将[甲子循环示例](#一个甲子的循环)复制到应用的 `src/main.rs`，在 `core-demo` 目录运行：

```sh
cargo run
```

全部断言通过后，程序以退出码 `0` 结束。

## 接口约定

根据输入来源和运行环境选择接口：

| 需求   | 用法与行为                                            |
| ------ | ----------------------------------------------------- |
| 索引   | `TryFrom<u8>` 拒绝越界值                              |
| 循环   | `from_index` 按周期回绕；`offset` 向前或向后步进      |
| 中文文本 | 三类干支值使用 `parse()` 严格解析、`Display` 显示；错误为 `ParseError` |
| 序列化 | 添加 `features = ["serde"]`；反序列化也校验干支配对   |
| 环境   | 库始终使用 `no_std`，默认无特性；不提供 `std` feature |

全库采用当前契约，不保留旧 API 或旧数据兼容层。可读 Serde 枚举仅接受代码字符串，记录仅接受具名对象，并拒绝未知、重复与缺失字段。藏干第三槽字段为 `tertiary`，空槽位显式编码为 `null`。变更与当前样本见[API 与编码说明](docs/migration.md)，实际验证见[清理记录](docs/breaking-cleanup.md)。

## 规则边界

Core 面向跨数术共享的基础模型。日期、节气、换日和各体系的应用条件由上层领域库处理。

现有十神、长生、藏干、刑合等查询采用固定映射。使用前请查阅[规则定义与证据边界](docs/rules.md)，确认采用的定义和核验状态；它们在不同体系中的适用性仍需逐项核验。固定关系查询不判断实际成局、成化或吉凶。

## 开发验证

仓库通过 [rust-toolchain.toml](rust-toolchain.toml) 固定 Rust 1.99.0，与 mise 和 CI 保持一致。在仓库内直接运行 `cargo` 或由编辑器调用 rustup 时，会选择该版本并准备 rustfmt、Clippy 和裸机目标，不依赖全局默认版本。

安装 mise 后，在 Core 仓库准备 [mise.toml](mise.toml) 指定的工具链，并安装本仓库的 Git hooks：

```sh
mise install
mise exec -- rustup component add rustfmt clippy
mise exec -- hk install --mise
```

[hk 配置](hk.pkl)在提交前检查暂存内容的 Rust 格式和 Clippy 警告，不自动修改或暂存文件；推送前运行 Serde 关闭／开启两组测试。修改 Rust 源码、Cargo 文件或相关工具配置时触发检查；`tests/fixtures/` 的冻结样本变更也会触发测试，纯文档改动跳过。

也可以手动检查整个工作区，或主动修复格式：

```sh
mise exec -- hk check --all
mise exec -- hk fix --all
mise exec -- hk check --all --slow
```

`--slow` 额外执行推送前的两组特性测试；不带该选项时只检查格式和 Clippy。

测试应全部通过，Clippy 应无警告。完整的 release 特性矩阵、裸机 `no_std` 编译和文档检查见[持续集成工作流](.github/workflows/ci.yml)；根据改动补跑对应检查，并记录执行环境与结果。hk 的安装方式和命令说明见[官方文档](https://hk.jdx.dev/getting_started.html)。

查找公开类型、方法和示例时，在 Core 仓库生成并打开接口文档：

```sh
mise exec -- cargo doc --no-deps --all-features --open
```

## 许可证

本项目采用 [MIT 许可证](LICENSE)。
