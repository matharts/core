<picture>
  <source media="(max-width: 600px)" srcset="assets/readme/hero-mobile.svg" />
  <img src="assets/readme/hero.svg" width="100%" alt="MathArts Core：阴阳、五行、干支的 Rust 基础库。十天干与十二地支同步步进，六十步回到甲子。" />
</picture>

# MathArts Core

数术项目共用的 Rust 基础库：表示阴阳、五行、干支与爻卦结构，提供合法值校验、周期步进和固定关系查询。

库始终使用 `no_std`，禁止 `unsafe`，默认无特性、无生产依赖；按需启用 `serde`。

[快速开始](#快速开始) · [当前能力](#当前能力) · [接口约定](#接口约定) · [规则边界](#规则边界) · [开发验证](#开发验证)

## 一个甲子的循环

解析“甲子”，向前走到“乙丑”，六十步回到起点；同一个值还能查询所属旬和纳音：

```rust
use matharts_core::{Branch, CyclicRing, SexagenaryCycle};

fn main() {
    let value: SexagenaryCycle = "甲子".parse().unwrap();

    println!("{value} → {}", value.offset(1)); // 甲子 → 乙丑
    assert_eq!(value.offset(60), value);
    assert_eq!(value.xun().void_branches(), (Branch::Xu, Branch::Hai));
    assert_eq!(value.nayin().name(), "海中金");
    assert!("甲丑".parse::<SexagenaryCycle>().is_err());
}
```

干支构造、中文解析和反序列化均校验天干与地支的阴阳配对。“甲丑”不会被接受为合法值。

## 快速开始

### 本地接入

安装 Rust，版本要求见 [Cargo.toml](Cargo.toml) 的 `rust-version`。将 Core 和应用放在同一父目录：

```sh
git clone https://github.com/matharts/core.git
cargo new core-demo
cd core-demo
```

在应用的 `Cargo.toml` 的 `[dependencies]` 下添加：

```toml
matharts_core = { path = "../core" }
```

将[上面的示例](#一个甲子的循环)复制到 `src/main.rs`，运行 `cargo run`。程序输出 `甲子 → 乙丑`，并验证周期回绕、旬内缺位地支、纳音与非法输入。

需要序列化时，将依赖改为：

```toml
matharts_core = { path = "../core", features = ["serde"] }
```

### 运行完整示例

在 Core 仓库根目录运行：

```sh
mise install
mise exec -- cargo run --example ganzhi --locked
```

[examples/ganzhi.rs](examples/ganzhi.rs) 展示中文解析、阴阳五行、关系查询、循环步进、旬与纳音，以及解析错误的分类处理。示例使用宿主程序的标准输出；库本身保持 `no_std`。

## 当前能力

| 领域       | 操作与查询                                                                   |
| ---------- | ---------------------------------------------------------------------------- |
| 阴阳、五行 | 阴阳反转、五行生克与关系查询                                                 |
| 干支       | 严格构造、周期步进、中文解析与显示                                           |
| 旬、纳音   | 身份、成员、旬内缺位地支与纳音五行                                           |
| 固定关系组 | 五合、六合、六冲、六害、采用表六破、三合与三会；查询归属、成员与完整集合识别 |
| 具名规则   | 十神、长生、藏干及刑的采用表查询                                             |
| 干推导     | `derive_month_stem`、`derive_hour_stem` 推导月干、时干                       |
| 爻卦结构   | 三爻、六爻构造，指定爻操作、阴阳反转与爻序倒置                               |

公共类型可从 crate 根导入；完整签名、采用规则与示例见生成的 rustdoc。

`Xun` 表示六十甲子的六旬；`Nayin` 表示三十类纳音身份。两者支持严格索引构造，`Nayin` 不提供周期操作。旬内缺位地支的查询不附加命盘中的空亡效应。

### 关系查询

先取得关系组身份，再查询成员、伙伴或固定对应。也可以用一组输入识别完整集合：

```rust
use matharts_core::{Branch, Element, FiveCombination, Stem, ThreeCombination};

let five = Stem::Jia.five_combination();
assert_eq!(five, FiveCombination::JiaJi);
assert_eq!(five.partner_of(Stem::Jia), Some(Stem::Ji));
assert_eq!(five.partner_of(Stem::Yi), None);
assert_eq!(five.element(), Element::Earth);

let three = Branch::Shen.three_combination();
assert_eq!(three.members(), [Branch::Zi, Branch::Chen, Branch::Shen]);
assert_eq!(
    ThreeCombination::from_branches([Branch::Shen, Branch::Chen, Branch::Zi]),
    Some(three),
);
assert_eq!(ThreeCombination::from_branches([Branch::Zi; 3]), None);
```

成员数组按领域索引升序；完整集合识别忽略输入排列，拒绝重复与混组。调用方需自行保留三传等有业务含义的顺序。六合不提供合化五行接口；同一输入可同时具有多种关系。

### 爻卦结构

六爻构造参数为下卦在前、上卦在后；爻数组从下到上，最低位代表初爻，阳为 `1`、阴为 `0`：

```rust
use matharts_core::{Hexagram, HexagramPosition, Trigram};

let tai = Hexagram::from_trigrams(Trigram::Qian, Trigram::Kun);
assert_eq!(tai.bits(), 7);
assert_eq!(tai.reverse_lines().bits(), 56);

let qian = Hexagram::from_trigrams(Trigram::Qian, Trigram::Qian);
let gou = qian.toggle_line(HexagramPosition::First);
assert_eq!((gou.lower(), gou.upper()), (Trigram::Xun, Trigram::Qian));
```

`complement()` 逐爻反转阴阳；`reverse_lines()` 倒置完整爻序。`try_from_bits()` 拒绝越界编码，爻位不周期回绕。结构编码不代表传统卦序，当前接口不提供六十四卦名称、文王序、动爻选择或占断规则。

## 接口约定

- **索引与周期**：`TryFrom<u8>` 严格拒绝越界值；`CyclicRing::from_index()` 按周期回绕，`offset()` 支持全部 `i32` 正负位移。
- **中文文本**：`Stem`、`Branch`、`SexagenaryCycle` 的 `FromStr` 仅接受精确名称，`Display` 输出中文。`ParseError` 区分格式、名称和非法配对；不裁剪空白或接受别名。
- **关系方向**：十神、生克、刑与循环距离保留参数方向。
- **运行环境**：默认无特性，唯一可选生产特性为 `serde`，启用后仍支持 `no_std`。

中文解析不分配堆内存。`Display` 可写入 `core::fmt::Write`；调用 `to_string()` 的分配发生在应用侧。

<details>
<summary>Serde 编码与严格解码</summary>

中文显示与机器编码分开维护。可读格式使用固定代码字符串和具名对象，例如：

```json
{ "stem": "Jia", "branch": "Zi" }
```

枚举接受代码字符串；记录拒绝数组、未知、重复和缺失字段，干支仍需通过阴阳配对校验。

| 记录              | 必需字段                                                |
| ----------------- | ------------------------------------------------------- |
| `SexagenaryCycle` | `stem`、`branch`                                        |
| `HiddenStems`     | `primary`、`secondary`、`tertiary`；空槽显式为 `null`   |
| `Hexagram`        | `lower`、`upper`，例如 `{"lower":"Qian","upper":"Kun"}` |

`HiddenStems` 是可自定义的记录容器，允许重复成员和第三槽独立存在。紧凑格式使用原生 unit variant／struct 模型，不承诺任意第三方格式的字节兼容。独立编码样本维护在 [tests/fixtures/](tests/fixtures/)，编码、解码和非法输入分别验证。

</details>

## 规则边界

Core 承载跨数术共享的基础值、周期操作和显式定义的固定关系。历法日期、节气、换日、排盘、起卦与流派策略由上层领域库处理。

关系查询返回固定对应，实际成局、成化、空亡效应和吉凶由调用方结合应用条件判断。十神、长生、藏干及刑合等采用表在不同体系中的适用性需逐项核验，使用前查看相关类型或方法的 rustdoc。

天干冲、五合、六合、三合、三会、长生和十神的 rustdoc 列出采用版本、消费场景、同输入对照及已知差异；转录校勘、实际消费方或跨体系证据尚未完成的部分明确标记为待补。固定表测试通过不代表这些证据已完备。六破采用《六壬大全》表，跨体系复用证据待补。

## 开发验证

[rust-toolchain.toml](rust-toolchain.toml) 固定 Rust `1.99.0`，与 [mise.toml](mise.toml) 和 CI 保持一致。在 Core 仓库安装工具链、Git hooks，并执行本地检查：

```sh
mise install
mise exec -- rustup component add rustfmt clippy
mise exec -- hk install --mise
mise exec -- hk check --all --slow
```

[hk.pkl](hk.pkl) 配置格式、严格 Clippy 及 Serde 关闭／开启两组 debug 测试；不带 `--slow` 时只检查格式与 Clippy。提交前检查暂存的 Rust 改动，推送前运行测试，纯文档改动跳过。

完整的 release 特性矩阵、裸机 `no_std` 构建、示例运行和严格文档检查见 [CI 工作流](.github/workflows/ci.yml)。按本次改动选择相应检查，执行要求见 [AGENTS.md](AGENTS.md)。

生成并打开公共接口文档：

```sh
mise exec -- cargo doc --no-deps --all-features --locked --open
```

## 文档

| 内容                         | 维护入口                                                                                                                    |
| ---------------------------- | --------------------------------------------------------------------------------------------------------------------------- |
| 接入、当前能力与接口约定     | 本 README；完整 API 见生成的 rustdoc                                                                                        |
| 项目职责、文件组织与验证要求 | [AGENTS.md](AGENTS.md)                                                                                                      |
| 统一领域术语                 | [CONTEXT.md](CONTEXT.md)                                                                                                    |
| 领域结构、固定关系与编码契约 | 所属类型的 rustdoc、源码与 [tests/](tests/)                                                                                 |
| 工程技能配置                 | [Issue 操作](docs/agents/issue-tracker.md)、[分诊标签](docs/agents/triage-labels.md)、[领域文档规则](docs/agents/domain.md) |

## 许可证

[MIT](LICENSE)
